# bpan

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Go Reference](https://pkg.go.dev/badge/github.com/ljh-sh/bpan.svg)](https://pkg.go.dev/github.com/ljh-sh/bpan)
[![Build](https://img.shields.io/badge/build-multi--platform-blueviolet)](#download)

A CLI for the **Baidu Netdisk personal Open Platform**, written in Go. The
official Baidu Netdisk SDK is vendored via `git subtree` so the build is
fully self-contained.

- **Repository**: <https://github.com/ljh-sh/bpan>
- **Pages**: <https://ljh-sh.github.io/bpan/>
- **License**: Apache-2.0
- **Vendored SDK**: [`baidu-netdisk/baidu-drive-sdk-go`](https://github.com/baidu-netdisk/baidu-drive-sdk-go)
  (Apache-2.0)
- **Upstream API docs**: <https://pan.baidu.com/union/doc/基础网盘服务>

## What bpan is

bpan is a **community-maintained** Go CLI for the Baidu Netdisk personal
Open Platform. It builds on the official Baidu SDK (Apache-2.0) and exposes
file-management, search, quota, and OAuth commands over a stable CLI surface.

Why a community CLI when `bdpan-storage` already exists?

- **Auditable end-to-end.** bpan is open-source Go, not a shell wrapper
  around a CDN-downloaded prebuilt binary.
- **12 native platforms.** Linux / macOS / Windows / FreeBSD / OpenBSD /
  NetBSD, each on amd64 and arm64 — built from a single Go toolchain in CI.
- **Stable for downstream.** Vendoring the SDK via `git subtree` keeps the
  build reproducible and lets the community patch and release without
  waiting on any single vendor.

### Compliance scope

bpan is built strictly on top of Baidu's published Open Platform API and
the official `baidu-netdisk/baidu-drive-sdk-go` SDK. All authentication
goes through OAuth device-code flow; no cookie/BDUSS injection, no web
scraping, no `pcs.baidu.com` rate-limit bypass.

Every user supplies their own `BDPAN_CLIENT_ID` / `BDPAN_CLIENT_SECRET`
obtained at <https://pan.baidu.com/union/>; bpan ships **no bundled
credentials**. See [`SECURITY.md`](SECURITY.md) for the full list of
features we deliberately do not implement.

See [`upstream/BAIDU-DRIVE-SDK-README.md`](upstream/BAIDU-DRIVE-SDK-README.md)
for how the vendoring works and how to upgrade the SDK copy.

## Quick start

### Install

```bash
# Linux/macOS (amd64 or arm64 auto-detected)
curl -L https://github.com/ljh-sh/bpan/releases/latest/download/bpan-installer.sh | sh

# Or via the Go toolchain
go install github.com/ljh-sh/bpan/cmd/bpan@latest
```

### Login

You need a Baidu Open Platform AppKey (a.k.a. client_id) and SecretKey
(a.k.a. client_secret). Apply at <https://pan.baidu.com/union/>.

```bash
export BDPAN_CLIENT_ID=your-appkey
export BDPAN_CLIENT_SECRET=your-secretkey
bpan login
# → opens URL, you enter the short user_code, tokens are saved.
```

### Use

```bash
bpan whoami            # show your Baidu account info
bpan ls /              # list Netdisk root
bpan ls /documents     # list a subdirectory
bpan upload file.txt /backup/file.txt
bpan download /backup/file.txt file.txt
bpan quota             # show used / total storage
```

## Commands

| Command                  | Description                                  |
|--------------------------|----------------------------------------------|
| `bpan login`             | OAuth device-code login                      |
| `bpan logout`            | delete saved credentials                     |
| `bpan whoami`            | show the logged-in account                   |
| `bpan ls [path]`         | list a Netdisk directory                     |
| `bpan upload <l> <r>`    | upload a local file                          |
| `bpan download <r> <l>`  | download a Netdisk file                      |
| `bpan search <query>`    | semantic search                              |
| `bpan quota`             | show storage usage                           |
| `bpan mkdir <path>`      | create a directory                           |
| `bpan rm <path>`         | delete a file or directory                   |
| `bpan mv <src> <dst>`    | move or rename                               |
| `bpan cp <src> <dst>`    | copy                                         |
| `bpan rename <p> <name>` | rename in place                              |
| `bpan install`           | install bpan to `~/.local/bin/`              |
| `bpan version`           | print version and exit                       |

## Global flags

```
--config <path>   config file path (default ~/.config/bdpan/config.json)
--json            output JSON for machine-readable responses
--verbose         verbose logging
--no-color        disable ANSI color output
```

## Building from source

```bash
git clone https://github.com/ljh-sh/bpan
cd bpan
GOTOOLCHAIN=auto go build -trimpath -o bpan ./cmd/bpan
```

The vendored SDK lives at `./upstream/baidu-drive-sdk-go/` and is referenced
via a `replace` directive in `go.mod`, so the build never reaches the
network for the SDK itself.

## Architecture

```
cmd/bpan/main.go                # CLI entrypoint and subcommand dispatch
internal/config                 # ~/.config/bdpan/config.json (0600)
internal/auth                   # OAuth device-code flow + token refresh
internal/client                 # SDK wrapper with refresh-aware construction
internal/sandbox                # lexical normalization of Netdisk paths
internal/version                # build-time version metadata (ldflags)
upstream/baidu-drive-sdk-go/    # git-subtree-vendored Baidu SDK
```

The CLI uses the SDK's recommended `scene` layer (`scene.Scene`),
which provides auto-retry, multi-API composition, and pre-checks. The
underlying `api.Client` is used for endpoints the scene layer doesn't
expose yet (e.g. `quota`).

## Roadmap

- v0.1.0 (this release): file management + login + quota + ls/upload/download
- v0.2.0: `transfer` / `share` once the SDK exposes them
- v0.2.0: `bpan update` for in-place self-update
- v0.3.0: optional OS-keychain credential storage

## Contributing

Issues and pull requests are welcome at
<https://github.com/ljh-sh/bpan>. The project follows the ljh-sh
dist-regime conventions (see `/.github/workflows/` for the CI matrix).

To upgrade the vendored SDK:

```bash
git subtree pull --prefix=upstream/baidu-drive-sdk-go \
    https://github.com/baidu-netdisk/baidu-drive-sdk-go.git \
    main --squash
```

See [`upstream/BAIDU-DRIVE-SDK-README.md`](upstream/BAIDU-DRIVE-SDK-README.md)
for the full procedure.

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).

The vendored SDK is also Apache-2.0; see
[`upstream/baidu-drive-sdk-go/LICENSE`](upstream/baidu-drive-sdk-go/LICENSE).

## See also

- Chinese README: [README.cn.md](README.cn.md)
- Pages site: <https://ljh-sh.github.io/bpan/>
- Reference SDK: <https://github.com/baidu-netdisk/baidu-drive-sdk-go>
- Skill packaging: <https://github.com/baidu-netdisk/bdpan-storage>
- Upstream API docs: <https://pan.baidu.com/union/doc/基础网盘服务>