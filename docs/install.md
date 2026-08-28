---
title: Install
lang: en
description: How to install bpan on Linux, macOS, Windows, FreeBSD, OpenBSD, NetBSD.
---

# Install

## Pre-built binary (recommended)

Each release page lists 12 assets:

```
bpan-linux-amd64.tar.xz
bpan-linux-arm64.tar.xz
bpan-darwin-amd64.tar.xz
bpan-darwin-arm64.tar.xz
bpan-windows-amd64.zip
bpan-windows-arm64.zip
bpan-freebsd-amd64.tar.xz
bpan-freebsd-arm64.tar.xz
bpan-openbsd-amd64.tar.xz
bpan-openbsd-arm64.tar.xz
bpan-netbsd-amd64.tar.xz
bpan-netbsd-arm64.tar.xz
```

### Linux / macOS / \*BSD

```bash
# Pick the right asset for your OS / arch, e.g. on macOS Apple Silicon:
curl -L -o bpan.tar.xz \
  https://github.com/ljh-sh/bpan/releases/latest/download/bpan-darwin-arm64.tar.xz

# Verify (recommended) — check the SHA256SUMS file from the same release page.
curl -L -o SHA256SUMS \
  https://github.com/ljh-sh/bpan/releases/latest/download/SHA256SUMS
sha256sum -c --ignore-missing < SHA256SUMS

# Extract and install
tar -xJf bpan.tar.xz
install -m 0755 bpan-pkg/bpan ~/.local/bin/bpan
```

### Windows

```powershell
# From PowerShell
Invoke-WebRequest -OutFile bpan.zip `
  https://github.com/ljh-sh/bpan/releases/latest/download/bpan-windows-amd64.zip
Expand-Archive bpan.zip
Move-Item bpan-pkg\bpan.exe $env:USERPROFILE\bin\bpan.exe
```

## Go toolchain

```bash
go install github.com/ljh-sh/bpan/cmd/bpan@latest
```

This uses the public Go module proxy. If you need offline / air-gapped
builds, see **Build from source** below.

## Build from source

```bash
git clone https://github.com/ljh-sh/bpan
cd bpan
GOTOOLCHAIN=auto go build -trimpath -ldflags="-s -w" -o bpan ./cmd/bpan
```

The vendored SDK lives at `./upstream/baidu-drive-sdk-go/` and is
referenced via a `replace` directive in `go.mod`, so the build never
reaches the network for the SDK itself.

## Requirements

- **Login**: a Baidu Open Platform AppKey and SecretKey. Apply at
  <https://pan.baidu.com/union/> (free for personal use).
- **Network**: HTTPS access to `pan.baidu.com` (API) and
  `openauth.baidu.com` (OAuth).
- **Disk**: ~10 MB for the binary, plus your own Netdisk quota.

## See also

- [CLI reference](cli/)
- Upstream API docs: <https://pan.baidu.com/union/doc/基础网盘服务>
- Source SDK: [`baidu-netdisk/baidu-drive-sdk-go`](https://github.com/baidu-netdisk/baidu-drive-sdk-go)