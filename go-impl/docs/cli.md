---
title: CLI Reference
lang: en
description: Every bpan subcommand, every flag, with examples.
---

# CLI Reference

## Global flags

These flags apply to every subcommand and must come **before** the
subcommand name:

```
bpan [global flags] <command> [command flags] [args...]

--config <path>   config file (default ~/.config/bdpan/config.json)
--json            machine-readable JSON output
--verbose         verbose logging
--no-color        disable ANSI color
```

## Environment

| Var                    | Purpose                            |
|------------------------|------------------------------------|
| `BDPAN_CLIENT_ID`      | Baidu Open Platform AppKey         |
| `BDPAN_CLIENT_SECRET`  | Baidu Open Platform SecretKey      |
| `BDPAN_CONFIG`         | override `--config` path           |

## `bpan login`

OAuth device-code flow against Baidu.

```
$ BDPAN_CLIENT_ID=xxx BDPAN_CLIENT_SECRET=yyy bpan login

To authorize bpan, open this URL in your browser:

    https://openauth.baidu.com/device

And enter this code:

    ABCD-EFGH

Waiting for authorization (expires at 2026-08-29T13:00:00+08:00)...
✓ Authorization successful — credentials saved.
```

## `bpan whoami`

```
$ bpan whoami
Baidu name: alice
Netdisk:    alice's netdisk
UK:         1234567890
VIP type:   svip
```

## `bpan ls [path]`

Flags: `--limit N`, `--order time|name|size`, `--desc`.

```
$ bpan ls /documents --limit 5 --order time --desc
d   4 KB      2026-08-29 12:00  work
-   1.2 MB    2026-08-29 11:30  report.pdf
d   256 B     2026-08-28 09:00  archive
total: 3 entries
```

## `bpan upload <local> <remote>`

Flags: `--overwrite`, `--chunk-size MB`.

```
$ bpan upload ./photo.jpg /backup/2026/photo.jpg
✓ uploaded ./photo.jpg → /backup/2026/photo.jpg
```

## `bpan download <remote> <local>`

```
$ bpan download /backup/2026/photo.jpg ./photo.jpg
✓ downloaded /backup/2026/photo.jpg → ./photo.jpg
```

## `bpan search <query>`

Flags: `--dir PATH`, `--type file|dir|all`.

```
$ bpan search "毕业照" --dir /photos --type file
DSC_0001.jpg
DSC_0002.jpg
```

## `bpan quota`

```
$ bpan quota
Quota:     12.4 GB / 2.0 TB
Free:      1.99 TB
```

## `bpan mkdir`, `rm`, `mv`, `cp`, `rename`

Standard file management. Paths are Netdisk paths, always absolute
(start with `/`). Lexical normalization prevents `..` from escaping root.

```
$ bpan mkdir /backup/2026
✓ mkdir /backup/2026

$ bpan mv /backup/2026/photo.jpg /backup/2026/old-photo.jpg
✓ mv /backup/2026/photo.jpg → /backup/2026/old-photo.jpg
```

## Exit codes

| Code | Meaning                  |
|------|--------------------------|
| 0    | success                  |
| 1    | general error            |
| 2    | argument / usage error   |
| 3    | authentication error     |
| 4    | permission denied        |
| 5    | resource not found       |
| 6    | quota exceeded           |