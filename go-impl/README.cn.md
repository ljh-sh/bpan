# bpan

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-multi--platform-blueviolet)](#下载)

百度网盘个人版开放平台命令行工具 (CLI)。Go 语言实现, 通过 `git subtree`
固化官方 SDK, 构建完全自包含。

- **仓库**: <https://github.com/ljh-sh/bpan>
- **Pages 站**: <https://ljh-sh.github.io/bpan/>
- **协议**: Apache-2.0
- **固化的 SDK**: [`baidu-netdisk/baidu-drive-sdk-go`](https://github.com/baidu-netdisk/baidu-drive-sdk-go)
  (Apache-2.0)
- **上游 API 文档**: <https://pan.baidu.com/union/doc/基础网盘服务>

## bpan 是什么

bpan 是百度网盘个人版开放平台的**社区维护** Go CLI, 基于官方 Baidu SDK
(Apache-2.0), 提供文件管理、搜索、配额与 OAuth 等命令的稳定 CLI 接口。

既然已有 `bdpan-storage`, 为什么要做社区 CLI?
- **端到端可审计**: bpan 是开源 Go, 不是围绕 CDN 预编译二进制的 shell 包装。
- **12 个原生平台**: Linux / macOS / Windows / FreeBSD / OpenBSD / NetBSD,
  每个平台覆盖 amd64 与 arm64, 由 Go 工具链一次构建。
- **下游依赖稳定**: 通过 `git subtree` 固化 SDK, 构建可复现, 社区可直接
  patch 并发布, 不依赖任何单一供应商。

### 合规范围

bpan 完全基于百度官方开放平台 API 与 `baidu-netdisk/baidu-drive-sdk-go`
SDK 构建。所有认证走 OAuth 设备码流程; 无 cookie/BDUSS 注入, 无 web
抓取, 无 `pcs.baidu.com` 限速绕过。

每个用户需自行在 <https://pan.baidu.com/union/> 申请
`BDPAN_CLIENT_ID` / `BDPAN_CLIENT_SECRET`; bpan **不内置任何共享凭据**。
完整"刻意不实现"清单见 [`SECURITY.md`](SECURITY.md)。

vendoring 机制与升级方式见
[`upstream/BAIDU-DRIVE-SDK-README.md`](upstream/BAIDU-DRIVE-SDK-README.md)。

## 快速开始

### 安装

```bash
# Linux/macOS (自动识别 amd64/arm64)
curl -L https://github.com/ljh-sh/bpan/releases/latest/download/bpan-installer.sh | sh

# 或通过 Go 工具链
go install github.com/ljh-sh/bpan/cmd/bpan@latest
```

### 登录

需要百度开放平台的 AppKey (即 client_id) 与 SecretKey (即 client_secret)。
申请地址: <https://pan.baidu.com/union/>。

```bash
export BDPAN_CLIENT_ID=你的-AppKey
export BDPAN_CLIENT_SECRET=你的-SecretKey
bpan login
# → 打印 URL 与 user_code, 浏览器打开后输入 user_code 即可。
```

### 常用命令

```bash
bpan whoami            # 显示当前百度账号
bpan ls /              # 列网盘根目录
bpan ls /documents     # 列子目录
bpan upload file.txt /backup/file.txt
bpan download /backup/file.txt file.txt
bpan quota             # 容量信息
```

## 完整命令表

| 命令                     | 用途                                  |
|--------------------------|---------------------------------------|
| `bpan login`             | OAuth 设备码登录                       |
| `bpan logout`            | 清除已保存的凭证                       |
| `bpan whoami`            | 显示当前账号                          |
| `bpan ls [path]`         | 列目录                                |
| `bpan upload <l> <r>`    | 上传文件                              |
| `bpan download <r> <l>`  | 下载文件                              |
| `bpan search <query>`    | 语义搜索                              |
| `bpan quota`             | 容量信息                              |
| `bpan mkdir <path>`      | 建目录                                |
| `bpan rm <path>`         | 删除                                  |
| `bpan mv <src> <dst>`    | 移动/重命名                           |
| `bpan cp <src> <dst>`    | 复制                                  |
| `bpan rename <p> <name>` | 重命名 (原地)                         |
| `bpan install`           | 安装到 `~/.local/bin/`                |
| `bpan version`           | 打印版本并退出                         |

## 全局 flag

```
--config <path>   配置文件路径 (默认 ~/.config/bdpan/config.json)
--json            JSON 输出 (机器可读)
--verbose         详细日志
--no-color        禁用 ANSI 颜色
```

## 从源码编译

```bash
git clone https://github.com/ljh-sh/bpan
cd bpan
GOTOOLCHAIN=auto go build -trimpath -o bpan ./cmd/bpan
```

固化的 SDK 在 `./upstream/baidu-drive-sdk-go/`, 通过 `go.mod` 中的 `replace`
指令引用, build 过程不联网获取 SDK。

## 架构

```
cmd/bpan/main.go                # CLI 入口与子命令分发
internal/config                 # ~/.config/bdpan/config.json (0600)
internal/auth                   # OAuth 设备码流程 + token 刷新
internal/client                 # SDK 包装 (含刷新感知)
internal/sandbox                # 网盘路径词法规范化
internal/version                # build-time 版本元数据 (ldflags)
upstream/baidu-drive-sdk-go/    # git subtree 固化的百度 SDK
```

CLI 调用 SDK 推荐的 `scene` 层 (`scene.Scene`), 该层提供自动重试、
多 API 串联、前置检查。底层 `api.Client` 用于 scene 层暂未暴露的端点
(如 `quota`)。

## 路线图

- v0.1.0 (本次): 文件管理 + 登录 + 配额 + ls/upload/download
- v0.2.0: `transfer` / `share` (SDK 暴露后)
- v0.2.0: `bpan update` 自更新
- v0.3.0: 可选 OS keychain 凭证存储

## 贡献

欢迎在 <https://github.com/ljh-sh/bpan> 提 issue / PR。本项目遵循
ljh-sh dist 规约 (CI matrix 见 `/.github/workflows/`)。

升级固化的 SDK:

```bash
git subtree pull --prefix=upstream/baidu-drive-sdk-go \
    https://github.com/baidu-netdisk/baidu-drive-sdk-go.git \
    main --squash
```

详见 [`upstream/BAIDU-DRIVE-SDK-README.md`](upstream/BAIDU-DRIVE-SDK-README.md)。

## 协议

Apache-2.0。见 [LICENSE](LICENSE) 与 [NOTICE.md](NOTICE.md)。

固化 SDK 亦为 Apache-2.0, 见
[`upstream/baidu-drive-sdk-go/LICENSE`](upstream/baidu-drive-sdk-go/LICENSE)。

## 相关链接

- English README: [README.md](README.md)
- Pages 站: <https://ljh-sh.github.io/bpan/>
- 上游 SDK: <https://github.com/baidu-netdisk/baidu-drive-sdk-go>
- Skill 包: <https://github.com/baidu-netdisk/bdpan-storage>
- 上游 API 文档: <https://pan.baidu.com/union/doc/基础网盘服务>