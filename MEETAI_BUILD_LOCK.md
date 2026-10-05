# MeetAI Codex Build Lock

This fork is the source of the `codex-app-server` binary consumed by
`meetai-agent` and `ppt-service`. Keep the source lock separate from the
runtime image lock: a source update must be built and smoke-tested before its
image tag is promoted.

## Current source baseline

| Field | Value |
|---|---|
| MeetAI fork | `git@github.com:Luoshu-Intelligent-Computing/meetai-codex.git` |
| MeetAI commit before sync | `e223405ddcb7c53a51fe19c814f5cb78c4a3473b` |
| MeetAI source merge | `43689d6cd36b46ee3bf78bc52128801851aa1044` |
| Upstream | `https://github.com/openai/codex.git` |
| Upstream reference checked | `823ea830c0fd418b09ff02d36cad9a1fff66465b` |
| Sync branch | `sync/upstream-823ea83` |
| Binary target | `codex-app-server` from `codex-rs/app-server` |
| Runtime image retained | `registry.cn-hangzhou.aliyuncs.com/meetai/codex-runtime:607e72f29-amd64` |

The upstream reference was fetched through the local proxy at `127.0.0.1:1081`.
合并已在上述同步分支进行；旧源码与已验证制品保留为回退基线。
共同祖先、冲突处理及同步验证见 `UPSTREAM_SYNC.md`。

## Build contract

From a complete checkout with a pinned Rust toolchain:

```bash
cd codex-rs
cargo build --release --locked --bin codex-app-server
```

The expected source artifact is `codex-rs/target/release/codex-app-server`.
Before publishing a runtime image, run the app-server protocol smoke and the
consumer repository smoke tests. Stage the binary with
`meetai-agent/scripts/stage-codex-app-server.sh`, set `MEETAI_CODEX_COMMIT` to
the fork commit, and record the resulting SHA256 beside the runtime image tag.

## Validation status

- Source and upstream refs recorded.
- Upstream fetch passed through `127.0.0.1:1081`.
- 双方历史已补齐，上游合并后的 app-server 编译检查通过。
- 本机已安装 stable Rust 和仓库锁定的 Rust 1.95.0；构建使用后者。
- 2026-10-05：`cargo build --release --locked --bin codex-app-server -j 2`
  成功，耗时 46 分 34 秒，源码检出提交为
  `289d5fc811e8487590da890af85e9cdfdfbb4c07`（源码基线仍为上述 MeetAI commit）。
- CLI `--help`、隔离 `CODEX_HOME` 下的 `initialize` / `initialized` /
  `thread/list` 检查通过。
- 原始 ELF 产物保留调试符号；staging 副本位于
  `codex-rs/target/staged/codex-app-server`，x86_64，213680648 字节。
- staging SHA256：
  `a29c19a8fc534d4b828720c6571a7495044fad4bea9709049efe4bd252d53a4d`。
- 原始产物 SHA256：
  `7620e84bce4a7dd87837283249bdf9b184a64ae7ded63e3adc61116b9a7d3fa5`。
- 构建存在 unused/dead-code 警告；本轮未修改源码消除警告。
- 上述 SHA256 对应旧 MeetAI 基线，不代表上游合并后的制品。
- 合并后 `just fmt` / `just fmt-check`、PPT 适配器与配置 26 项测试、
  MCP HTTP 进度通知集成测试通过。
- 上游合并后的 release 构建完成（41 分 56 秒），15 项 Rust 定向测试通过；尚未发布新镜像。
- 完整模型任务和全栈消费者端到端验证尚未执行。

## 上游合并后制品（2026-10-05）

- 源码：`43689d6cd36b46ee3bf78bc52128801851aa1044`。
- 构建：Rust 1.95.0，`cargo build --release --locked --bin codex-app-server -j 2`。
- staging：`codex-rs/target/staged-upstream-823ea83/codex-app-server`，
  x86_64，218255720 字节；相邻 JSON 记录来源。
- staging SHA256：`9ea1af845ad55065eac539a814b7725b697ef4304b944fd3768172bb8db0febd`。
- 原始 SHA256：`b71251125596582985eee7dcfb179b8f8be25a66087c4ac141012d5a9ce21450`。
- 原始和 stripped 二进制均通过 CLI help、真实 PPT 适配器 initialize/thread-start、
  Agent 请求字段 initialize/thread-start/thread-list 检查；未发起 turn 或模型推理。
- 已通过 sudo 添加 `/etc/apparmor.d/bwrap` 专用 AppArmor profile：
  `/usr/bin/bwrap` 使用 `flags=(unconfined)` 和 `userns,`，并用
  `sudo apparmor_parser -r /etc/apparmor.d/bwrap` 加载。保留全局
  `kernel.apparmor_restrict_unprivileged_userns=1`；一般程序的 `unshare`
  仍可能受限，不代表 bubblewrap 失败。
- bubblewrap 的 user/network namespace 检查通过，app-server 沙箱警告消失。
  标准 `command/exec`（workspaceWrite）实际执行验证：工作区内写入/读取成功，
  工作区外写入返回只读文件系统错误。CODEX_HOME 使用用户缓存目录，避免
  `/tmp` 中的 helper alias 被拒绝。此次未调用模型或执行 Agent turn。
- `scripts/test-codex-runtime-image-contract.sh` 静态合同通过。
- 已安装 Podman 4.9.3 并确认 rootless 模式，候选镜像
  `localhost/meetai-codex-runtime:43689d6-amd64` 已本地构建，约 505 MiB。
  image ID：`6e611e33e56cb1a3fdf13125c8bc5501380d561067e3737c05aa94a751debfd8`；
  labels 中的源码提交及二进制 SHA256 与上述锁定值一致。
- 镜像内 initialize、真实 command/exec 工作区写入和工作区外写入拒绝均通过。
  本机 rootless Podman 需 `--security-opt 'unmask=/proc/*'`，否则 bubblewrap
  返回 `Can't mount proc on /proc`；同时使用
  `--userns=keep-id:uid=999,gid=999` 映射挂载目录的当前用户到镜像 agentuser。
  验证使用 `--network none`，未增加 privileged 或修改默认 seccomp。
  这些结论针对本机 Podman，不代替其他部署环境的验证。
- 候选镜像未发布，生产镜像锁未修改；完整 Agent turn 与模型推理仍待验证。
- 如需撤销本机许可：`sudo apparmor_parser -R /etc/apparmor.d/bwrap`，然后
  删除该 profile 文件；这不涉及仓库源码或模型配置。
