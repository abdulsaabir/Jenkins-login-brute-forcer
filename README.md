# Jenkins-login-brute-forcer

Command-line tool to brute-force Jenkins logins. The program is **`jenkins_brute`**.

Use it **only on systems you are allowed to test**.

---

## Install: use a release

**Prefer a [GitHub Release](https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases)** — download the ready-made binary for your OS (you do **not** need Rust).

| OS | Download |
|----|----------|
| Linux x86_64 | `jenkins_brute-linux-x86_64` |
| Windows x86_64 | `jenkins_brute-windows-x86_64.exe` |
| macOS (Apple Silicon) | `jenkins_brute-macos-aarch64` |

**Linux** (latest release, one-liner):

```bash
curl -fsSL -o jenkins_brute \
  "https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases/latest/download/jenkins_brute-linux-x86_64"
chmod +x jenkins_brute
```

Then run (example):

```bash
./jenkins_brute --url 'http://YOUR_JENKINS:PORT' -l admin -P passwords.txt
```

On Windows, download the `.exe` from the same Releases page and run it from a terminal.

---

## How to use

```text
jenkins_brute --url <URL> [options]
```

| | |
|--|--|
| `--url` / `-u` | Jenkins base URL (**required**), e.g. `http://host:8080` |
| `-l` | One username |
| `-L` | File with usernames (one per line) |
| `-p` | One password |
| `-P` | File with passwords (one per line) |
| `-e` | Login path (optional; default suits most Jenkins installs) |
| `-t` | Number of parallel workers (optional) |

Examples:

```bash
jenkins_brute --url 'http://jenkins.example:8080' -l admin -P passwords.txt
jenkins_brute --url 'http://jenkins.example:8080' -L users.txt -P passwords.txt -t 256
```

All options: **`jenkins_brute --help`**

---

## Build from source (optional)

```bash
cargo build --release
```

Output: `target/release/jenkins_brute`
