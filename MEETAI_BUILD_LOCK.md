# MeetAI Codex Build Lock

This fork is the source of the `codex-app-server` binary consumed by
`meetai-agent` and `ppt-service`. Keep the source lock separate from the
runtime image lock: a source update must be built and smoke-tested before its
image tag is promoted.

## Current source baseline

| Field | Value |
|---|---|
| MeetAI fork | `git@github.com:Luoshu-Intelligent-Computing/meetai-codex.git` |
| MeetAI commit | `e223405ddcb7c53a51fe19c814f5cb78c4a3473b` |
| Upstream | `https://github.com/openai/codex.git` |
| Upstream reference checked | `823ea830c0fd418b09ff02d36cad9a1fff66465b` |
| Sync branch | `sync/upstream-823ea83` |
| Binary target | `codex-app-server` from `codex-rs/app-server` |
| Runtime image retained | `registry.cn-hangzhou.aliyuncs.com/meetai/codex-runtime:607e72f29-amd64` |

The upstream reference was fetched through the local proxy at `127.0.0.1:1081`.
It has not been merged into the MeetAI fork. Integration must happen on a
dedicated synchronization branch after history and protocol review.

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
- Upstream merge pending history deepening and semantic review.
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
- 完整模型任务、消费者端到端 smoke 和上游合并验证尚未执行；尚未发布新镜像。
