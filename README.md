# bpan

**AI-agent-first CLI + Rust SDK for Baidu Netdisk personal Open Platform.**

bpan is designed for LLM/AI agents that need to interact with Baidu
Netdisk programmatically — not for humans typing commands in a terminal.

- **Structured by default**: every command outputs stable JSON unless `--human`
- **Zero interaction**: no menus, no prompts (use OAuth device-code)
- **Predictable exit codes**: BSD sysexits.h style (0/1/2/3/4/5/6)
- **Rust SDK + CLI in one crate**: `use bpan::Client;` for embedders
- **12 native platforms**: Linux / macOS / Windows / FreeBSD / OpenBSD / NetBSD

---

## What's new in v0.2.0

- **Complete Rust rewrite** (Go-based v0.1.0 lives at [`go-impl/`](go-impl/) as historical reference)
- **Default JSON output** for agent consumption
- **Rust SDK** published to crates.io (`bpan = "0.2"`)
- **OAuth device-code flow** built in (no loopback, no port conflicts)
- **No vendored SDK dependency**: 100% self-written, based on Baidu's published Open Platform docs
- **Single GitHub Actions runner** cross-compiles all 12 platforms via `cargo-zigbuild`

## Installation

```bash
# Linux/amd64 example (12 platforms available, see Releases)
curl -L -o bpan.tar.xz https://github.com/ljh-sh/bpan/releases/download/v0.2.0/bpan-linux-amd64.tar.xz
tar -xJf bpan.tar.xz
install -m 0755 bpan-linux-amd64-pkg/bpan ~/.local/bin/bpan

# Or via Cargo (Rust users):
cargo install bpan
```

## Quick start

```bash
# Apply for AppKey at https://pan.baidu.com/union/ first
export BDPAN_CLIENT_ID=...
export BDPAN_CLIENT_SECRET=...

bpan login    # OAuth device-code flow → prints URL + user_code
bpan whoami   # show account (JSON by default)
bpan ls /     # list Netdisk root
bpan quota    # show storage usage
```

## JSON output (default)

```bash
$ bpan --json ls /
{
  "ok": true,
  "data": {
    "path": "/",
    "entries": [
      {"fs_id": 123, "filename": "report.pdf", "is_dir": false, "size": 12345, "mtime": 1693219200},
      ...
    ],
    "total": 12
  }
}
```

## Rust SDK usage

```toml
# Cargo.toml
[dependencies]
bpan = "0.2"
tokio = { version = "1", features = ["full"] }
```

```rust
use bpan::{Client, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Config::from_env()?);
    let token = client.login_device_code().await?;
    client.save_token(&token).await?;

    let user = client.user_info().await?;
    println!("hello, {}", user.baidu_name);

    let files = client.list_dir("/", None).await?;
    for f in files {
        println!("  {} ({} bytes)", f.filename, f.size);
    }
    Ok(())
}
```

## Commands

| Command                   | Description                              |
|---------------------------|------------------------------------------|
| `bpan login`              | OAuth device-code login                  |
| `bpan logout`             | delete saved credentials                 |
| `bpan whoami`             | show logged-in account                   |
| `bpan ls [path]`          | list a Netdisk directory                 |
| `bpan upload <l> <r>`     | upload a local file                      |
| `bpan download <r> <l>`   | download a Netdisk file                  |
| `bpan search <query>`     | semantic search                          |
| `bpan quota`              | show storage usage                       |
| `bpan mkdir/rm/mv/cp/rename` | file management                        |
| `bpan install`            | install to `~/.local/bin/`               |
| `bpan version`            | print version and exit                   |

## Compliance

bpan uses **only Baidu's published Open Platform API** (`/xpan/*`) and the
**official Go SDK patterns** (now self-written in Rust). Authentication is
OAuth device-code flow only. Every user supplies their own
`BDPAN_CLIENT_ID` / `BDPAN_CLIENT_SECRET` — bpan ships no bundled credentials.

This implementation does NOT depend on Baidu's official Go SDK; all HTTP calls
are made directly against the documented Open Platform endpoints at
<https://pan.baidu.com/union/doc/基础网盘服务>.

Powered by Baidu Netdisk Open Platform.

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).

The Go-based v0.1.0 implementation is preserved at [`go-impl/`](go-impl/)
under the same license, for reference and for users who prefer it.