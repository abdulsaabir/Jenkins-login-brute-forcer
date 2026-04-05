# jenkins_brute

Hydra-style brute-forcer for Jenkins form login. Sends POST requests to the configured login endpoint (default: `j_spring_security_check`) and treats a successful login based on the HTTP `Location` header (redirect handling disabled).



## Run without installing Rust (prebuilt binary)

The repo does not require visitors to install Rust if you publish **prebuilt binaries** on [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).

1. Push your code to GitHub.
2. Tag a version and push the tag (this project includes a workflow that builds Linux, Windows, and macOS binaries and attaches them to the release):

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. After the **Release** workflow finishes, open **Releases** on your repo and download the file for your OS (e.g. `jenkins_brute-linux-x86_64`).

**Linux example** (replace `OWNER`, `REPO`, and `v0.1.0` with yours):

```bash
curl -fsSL -o jenkins_brute \
  "https://github.com/OWNER/REPO/releases/download/v0.1.0/jenkins_brute-linux-x86_64"
chmod +x jenkins_brute
./jenkins_brute 'http://jenkins.example:8080' -l admin -P passwords.txt
```

Windows users download `jenkins_brute-windows-x86_64.exe` and run it from a terminal. macOS (Apple Silicon) uses `jenkins_brute-macos-aarch64`.

That is the usual “one download, one command” experience; you only maintain **releases**, not Rust on every user’s machine.

## Build from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
cargo build --release
```

Binary: `target/release/jenkins_brute` (or `jenkins_brute.exe` on Windows).

## Usage

```text
jenkins_brute [OPTIONS] <URL>
```

The first argument is the Jenkins base URL (no trailing slash required), for example `http://jenkins.example.com:8080`.

You must supply at least one username (`-l` or `-L`) and at least one password (`-p` or `-P`).

### Options

| Option | Description |
|--------|-------------|
| `<URL>` | Target Jenkins base URL (positional, required). |
| `-e`, `--endpoint <PATH>` | Login form POST path. Default: `j_spring_security_check`. Older installs may use `j_acegi_security_check`. |
| `-l`, `--user <USERNAME>` | Single username. |
| `-L`, `--user-file <USERFILE>` | File of usernames (one per line). |
| `-p`, `--password <PASSWORD>` | Single password. |
| `-P`, `--pass-file <PASSFILE>` | File of passwords (one per line). |
| `-t`, `--threads <N>` | Concurrent workers. Default: `CPU × 8`, clamped between **32** and **512**. Omit to use the default; raise for large wordlists (e.g. rockyou). |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

You can combine `-l` with `-L` (and `-p` with `-P`); entries are merged and deduplicated.

Wordlists are read line-by-line with **lossy UTF-8** so files like rockyou (with occasional invalid bytes) still load.

While running, progress appears on stderr as `[current:total]` (similar in spirit to ffuf), updating in place.

## Examples

Single user and password file:

```bash
./target/release/jenkins_brute 'http://jenkins.target:8080' -l admin -P passwords.txt
```

User list and password list:

```bash
./target/release/jenkins_brute 'http://jenkins.target:8080' -L users.txt -P passwords.txt
```

Older Jenkins login path:

```bash
./target/release/jenkins_brute 'http://jenkins.target:8080' -e j_acegi_security_check -l admin -P rockyou.txt
```

More concurrent workers (e.g. huge lists):

```bash
./target/release/jenkins_brute 'http://jenkins.target:8080' -L users.txt -P rockyou.txt -t 256
```

## Notes

- If `--url` has no port, **`http://` is normalized to port 80** and **`https://` to port 443** (shown explicitly in `[*] Target:`). Jenkins on another port (e.g. 8080) must still be given as `http://host:8080`.
- Requests use a 10s connect and overall timeout; redirects are not followed (login success is inferred from headers).
- Default thread count is tuned for **I/O-bound** HTTP work, not CPU core count only.

## Disclaimer

This tool is for **authorized security testing** only. Misuse may violate computer crime laws and program rules. You are responsible for complying with applicable laws and any bug bounty or contract terms.
