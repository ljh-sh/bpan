---
title: Home
lang: en
description: bpan — community-maintained Go CLI for Baidu Netdisk.
---

# bpan

A CLI for the **Baidu Netdisk personal Open Platform**, written in Go.
The official Baidu Netdisk SDK is vendored via `git subtree` so the build
is fully self-contained.

- **Repository**: <https://github.com/ljh-sh/bpan>
- **License**: Apache-2.0
- **Vendored SDK**: [`baidu-netdisk/baidu-drive-sdk-go`](https://github.com/baidu-netdisk/baidu-drive-sdk-go)
- **Upstream API docs**: <https://pan.baidu.com/union/doc/基础网盘服务>

## What bpan is

bpan is a **community-maintained** Go CLI for the Baidu Netdisk personal
Open Platform. It builds on the official Baidu SDK (Apache-2.0) and exposes
file-management, search, quota, and OAuth commands over a stable CLI surface.

Why a community CLI when `bdpan-storage` already exists?

- **Auditable end-to-end.** bpan is open-source Go, not a shell wrapper
  around a CDN-downloaded prebuilt binary.
- **12 native platforms.** Linux / macOS / Windows / FreeBSD / OpenBSD /
  NetBSD, each on amd64 and arm64.
- **Stable for downstream.** Vendoring via `git subtree` keeps the build
  reproducible and lets the community patch and release.

## Quick start

```bash
# Install
curl -L https://github.com/ljh-sh/bpan/releases/latest/download/bpan-installer.sh | sh

# Login (apply for AppKey at https://pan.baidu.com/union/ first)
export BDPAN_CLIENT_ID=...
export BDPAN_CLIENT_SECRET=...
bpan login

# Use
bpan whoami
bpan ls /
bpan upload file.txt /backup/file.txt
bpan quota
```

## Commands at a glance

| Command                   | Description                       |
|---------------------------|-----------------------------------|
| `bpan login`              | OAuth device-code login           |
| `bpan logout`             | delete saved credentials          |
| `bpan whoami`             | show logged-in account            |
| `bpan ls [path]`          | list a Netdisk directory          |
| `bpan upload <l> <r>`     | upload a file                     |
| `bpan download <r> <l>`   | download a file                   |
| `bpan search <q>`         | semantic search                   |
| `bpan quota`              | show storage usage                |
| `bpan mkdir/rm/mv/cp/rename` | file management                 |
| `bpan install`            | install to `~/.local/bin/`        |

See [CLI reference](cli/) for flags and details.