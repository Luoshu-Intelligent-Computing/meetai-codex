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
- Local Rust build unavailable on this workstation (`cargo`/`rustc` absent).
