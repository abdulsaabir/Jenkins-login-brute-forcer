# Jenkins-login-brute-forcer

Hydra-style brute-forcer for Jenkins form login (binary name: `jenkins_brute`). It POSTs to the login endpoint (default: `j_spring_security_check`) and infers success from the HTTP `Location` header (redirects are not followed).

**Use only on targets you are allowed to test** (your lab, in-scope bug bounty, CTF, etc.).

---

## Use a GitHub Release (recommended)

**You do not need Rust or Cargo.** Download a **prebuilt binary** from this repo’s [Releases](https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases) page.

| Your system | Download this asset |
|-------------|---------------------|
| Linux x86_64 | `jenkins_brute-linux-x86_64` |
| Windows x86_64 | `jenkins_brute-windows-x86_64.exe` |
| macOS Apple Silicon | `jenkins_brute-macos-aarch64` |

1. Open **[Releases](https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases)**.
2. Choose the **latest** release (see version note below).
3. Under **Assets**, download the file for your OS.
4. **Linux / macOS:** `chmod +x jenkins_brute-linux-x86_64` (or the macOS name) after download.
5. Run the tool with **`--url`** (see [Usage](#usage)). Example:

```bash
chmod +x jenkins_brute-linux-x86_64
./jenkins_brute-linux-x86_64 --url 'http://jenkins.example:8080' -l admin -P passwords.txt
```

### Download without a browser (Linux)

This always fetches the **latest** release asset:

```bash
curl -fsSL -o jenkins_brute \
  "https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases/latest/download/jenkins_brute-linux-x86_64"
chmod +x jenkins_brute
./jenkins_brute --url 'http://jenkins.example:8080' -l admin -P passwords.txt
```

Replace the filename in the URL with `jenkins_brute-windows-x86_64.exe` or `jenkins_brute-macos-aarch64` if needed.

### Version note (v1.1+)

- **Use release v1.1.0 or newer** from [Releases](https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases). The CLI requires **`--url` / `-u`** for the Jenkins base URL.
- Very old unofficial builds that used a **positional** URL (`jenkins_brute <URL> ...`) are obsolete; current releases use **`--url`** only.

---

## Build from source (optional)

Only needed if you change the code or do not want a release binary.

Requires [Rust](https://rustup.rs/) (stable).

```bash
cargo build --release
```

Output: `target/release/jenkins_brute` (or `jenkins_brute.exe` on Windows).

---

## Publishing a new release (maintainers)

The [Release workflow](.github/workflows/release.yml) builds Linux, Windows, and macOS binaries using **Node 24–compatible** GitHub Actions (`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`, current `actions/checkout`, `upload-artifact`, `download-artifact`).

1. Commit and push to `main`.
2. Create and push a version tag:

```bash
git tag v1.1.0
git push origin v1.1.0
```

3. On GitHub: **Actions** → wait for **Release** to finish → **Releases** should show the new version with assets.

---

## Usage

```text
jenkins_brute --url <URL> [OPTIONS]
```

`--url` (short: `-u`) is **required** and must start with **`http://`** or **`https://`**.

You need at least one username (`-l` or `-L`) and at least one password (`-p` or `-P`). Options can be in any order.

### CLI tips

- Do **not** append `--help` to a real run; that only prints help and exits.
- You usually do **not** need `--` before `-L` / `-P`.

### Validation and errors

The binary prints clear errors if `--url`, credentials, or wordlist paths are wrong (missing flags, bad paths, empty lists, etc.).

### Options

| Option | Description |
|--------|-------------|
| `-u`, `--url <URL>` | Target Jenkins base URL (**required**). Example: `http://jenkins.example.com:8080` |
| `-e`, `--endpoint <PATH>` | Login POST path. Default: `j_spring_security_check`. Older Jenkins may use `j_acegi_security_check`. |
| `-l`, `--user <USERNAME>` | Single username. |
| `-L`, `--user-file <USERFILE>` | Username wordlist (one per line). |
| `-p`, `--password <PASSWORD>` | Single password. |
| `-P`, `--pass-file <PASSFILE>` | Password wordlist (one per line). |
| `-t`, `--threads <N>` | Workers. Default: `CPU × 8`, clamped **32–512**. |
| `-h`, `--help` | Help. |
| `-V`, `--version` | Version. |

Wordlists use **lossy UTF-8** per line (e.g. rockyou). Progress on stderr: `[current:total]`.

### Examples

```bash
./jenkins_brute --url 'http://jenkins.target:8080' -l admin -P passwords.txt
./jenkins_brute -L users.txt -P passwords.txt --url 'http://jenkins.target:8080'
./jenkins_brute --url 'http://jenkins.target:8080' -e j_acegi_security_check -l admin -P rockyou.txt
./jenkins_brute --url 'http://jenkins.target:8080' -L users.txt -P rockyou.txt -t 256
```

## Notes

- 10s connect and request timeout; redirects are not followed.
- Default thread count is aimed at **I/O-bound** HTTP work.

## Disclaimer

For **authorized security testing** only. You are responsible for legal and program rules compliance.
