# Security Policy

## Supported versions

| Version | Supported          |
|---------|--------------------|
| latest  | ✅ actively patched |
| < latest | ❌ please upgrade |

bpan follows a `latest-only` support model. We do not backport security
fixes to older releases.

## Reporting a vulnerability

Please **do not file a public issue** for security vulnerabilities.

Email: **ljh@x-cmd.com** (PGP key on request)

Include in your report:

1. A clear description of the vulnerability and its impact.
2. Steps to reproduce, ideally with a minimal `bpan` invocation.
3. The version of `bpan` (`bpan version` output).
4. Any known workarounds.

We will:

- Acknowledge your report within 3 days.
- Triage and respond with an impact assessment within 7 days.
- Coordinate disclosure timing with you — typically a fix-and-release within
  30 days for high-severity findings.

## Vendored SDK security

bpan vendors `baidu-netdisk/baidu-drive-sdk-go` via `git subtree`. If you find
a vulnerability in that SDK, please report it upstream at
https://github.com/baidu-netdisk/baidu-drive-sdk-go/security — but also
notify us so we can backport a fix to our vendored copy.

## Threat model

bpan is a personal Netdisk CLI that uses only Baidu's published Open
Platform API and the official `baidu-netdisk/baidu-drive-sdk-go` SDK. All
authentication goes through OAuth device-code flow; all calls go to
endpoints documented at <https://pan.baidu.com/union/doc/基础网盘服务>.

Threats we consider in scope:

- Token exfiltration from `~/.config/bdpan/config.json` (mitigated: 0600).
- Path traversal of `remote` arguments against Netdisk (mitigated:
  `internal/sandbox` lexical normalization).
- Malicious Baidu API responses that escape the SDK's own validation
  (mitigated: SDK tests; we surface errors verbatim).

Threats we consider out of scope:

- Adversaries with read access to your local filesystem (keychain / disk
  encryption is the user's responsibility).
- Baidu-side account compromise (rotate your Baidu password).