# jenkins_brute

Hydra-style brute-forcer for Jenkins form login. It POSTs to your login path (default `j_spring_security_check`) and treats a successful login from the HTTP `Location` header; POSTs do not follow redirects so that logic stays reliable. On startup it sends a **GET** to the base URL with redirects enabled, so if something on port 80 only forwards to Jenkins on another port, the tool uses the resolved origin for all attempts.

## Prebuilt binaries

Downloads are on [GitHub Releases](https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases) (Linux, Windows, macOS)

Or from Linux Terminal:

```bash
curl -fsSL -o jenkins_brute \
  "https://github.com/abdulsaabir/Jenkins-login-brute-forcer/releases/latest/download/jenkins_brute-linux-x86_64"
chmod +x jenkins_brute
./jenkins_brute --url 'http://jenkins.example:8080' -l admin -P passwords.txt
```

On Windows, use the `.exe` from the release. On Apple Silicon Macs, use the `macos-aarch64` binary.

## Build from source

Install [Rust](https://rustup.rs/) (stable), then:

```bash
cargo build --release
```

Binary: `target/release/jenkins_brute` (or `jenkins_brute.exe` on Windows).

## Usage

```text
jenkins_brute --url <URL> [OPTIONS]
```

`--url` is the Jenkins base URL (no trailing slash required), for example `http://jenkins.example.com:8080`.

You need at least one username (`-l` or `-L`) and at least one password (`-p` or `-P`).

### Options

| Option | Description |
|--------|-------------|
| `-u`, `--url <URL>` | Jenkins base URL (required). |
| `-e`, `--endpoint <PATH>` | Login POST path. Default: `j_spring_security_check`. Older installs may use `j_acegi_security_check`. |
| `-l`, `--user <USERNAME>` | Single username. |
| `-L`, `--user-file <USERFILE>` | File of usernames (one per line). |
| `-p`, `--password <PASSWORD>` | Single password. |
| `-P`, `--pass-file <PASSFILE>` | File of passwords (one per line). |
| `-t`, `--threads <N>` | Worker threads. Default: CPU×8, between 32 and 512. |
| `-h`, `--help` | Help. |
| `-V`, `--version` | Version. |

You can combine `-l` with `-L` (and `-p` with `-P`); values are merged and deduplicated.

Wordlists are read line by line with lossy UTF-8 so files like rockyou still load if a line has invalid bytes.

Progress prints on stderr as `[current:total]`, updated in place.

## Examples

Single user and password file:

```bash
./target/release/jenkins_brute --url 'http://jenkins.target:8080' -l admin -P passwords.txt
```

User list and password list:

```bash
./target/release/jenkins_brute --url 'http://jenkins.target:8080' -L users.txt -P passwords.txt
```

Older Jenkins login path:

```bash
./target/release/jenkins_brute --url 'http://jenkins.target:8080' -e j_acegi_security_check -l admin -P rockyou.txt
```

More threads for large lists:

```bash
./target/release/jenkins_brute --url 'http://jenkins.target:8080' -L users.txt -P rockyou.txt -t 256
```


## Disclaimer

For authorized security testing only. Misuse can break laws and rules. You are responsible for compliance.
