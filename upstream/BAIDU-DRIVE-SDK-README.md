# Vendored SDK: baidu-netdisk/baidu-drive-sdk-go

This directory contains a **git-subtree-vendored** copy of the official Baidu Netdisk Go SDK.

## Origin

- **Upstream URL**: https://github.com/baidu-netdisk/baidu-drive-sdk-go
- **Upstream default branch**: `main`
- **Upstream version at vendoring**: v0.1.0
- **Upstream commit SHA at vendoring**: `e975a2c2b0bd123b5a7c9aca4f59fe7a6e02de4e`
- **Subtree commit SHA (squashed into ljh-sh/bpan)**: `fab9266`
- **License**: Apache-2.0 (see `./LICENSE` for full text)

## Vendoring mechanism

We use **`git subtree` (squash mode)** rather than submodules or a plain tarball extract:

| Mode | Build works when upstream deleted? | Upgrade path |
|------|-------------------------------------|--------------|
| git subtree (us) | ✅ yes | `git subtree pull --prefix=upstream/baidu-drive-sdk-go https://github.com/baidu-netdisk/baidu-drive-sdk-go.git main --squash` |
| git submodule   | ❌ points to upstream SHA | `git submodule update --remote` |
| tarball         | ✅ yes but loses history  | manual re-download |

The `replace` directive in `/go.mod` ensures Go's toolchain reads from `./upstream/baidu-drive-sdk-go/`, not the network. Builds are reproducible from this repository alone — no upstream fetch is required at build time.

## First sync

2026-08-29 — initial subtree add from main @ `e975a2c`.

## Upgrade procedure

```bash
# From repo root:
git subtree pull --prefix=upstream/baidu-drive-sdk-go \
    https://github.com/baidu-netdisk/baidu-drive-sdk-go.git \
    main --squash

# Bump version pin in /go.mod
# require github.com/baidu-netdisk/baidu-drive-sdk-go v0.2.0

GOPROXY=https://goproxy.cn,direct GOTOOLCHAIN=auto go mod tidy
GOPROXY=https://goproxy.cn,direct GOTOOLCHAIN=auto go build ./...
```

## Contributing back upstream

```bash
git subtree push --prefix=upstream/baidu-drive-sdk-go \
    https://github.com/baidu-netdisk/baidu-drive-sdk-go.git \
    <branch-name>
```

## Rationale

We chose `git subtree` (squash mode) over `git submodule` or a plain tarball
extract because:

- **Reproducible builds.** A `git clone` of this repo is enough to compile —
  no submodule init, no upstream fetch at build time.
- **Full upgrade history.** Each `git subtree pull` produces a single
  squashed commit on top of `main`, plus the upstream commit chain is
  preserved in `.git` for archaeology.
- **Community-contributable.** Anyone with commit access can `git subtree
  push` a fix back upstream without learning submodule mechanics.