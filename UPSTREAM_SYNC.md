# Upstream synchronization audit

This branch (`sync/upstream-823ea83`) records the first synchronization pass
against OpenAI Codex commit `823ea830c0fd418b09ff02d36cad9a1fff66465b`.

## Result

The upstream commit is fetched and pinned, but is intentionally not merged
yet. The local clone was shallow and did not contain the parent history needed
for a meaningful merge base. A direct snapshot diff also shows broad app-server
protocol evolution, including changes to `ClientRequest`, `ServerNotification`,
thread resume/list responses, attachments, MCP OAuth, model capabilities, and
turn lifecycle events. Treating this as a routine fast-forward would risk
breaking `meetai-agent` and `ppt-service` consumers.

## Integration gates

1. Deepen both fork and upstream history and record the merge base.
2. Review protocol changes against the consumer adapter and event trajectory
   importer; preserve wire compatibility where MeetAI depends on it.
3. Keep model-specific behavior in `llm-service`/deployment. Retain a Codex
   change only when it is required for request serialization or app-server
   protocol compatibility, with a focused regression test.
4. Build `codex-app-server` with `cargo build --release --locked --bin
   codex-app-server` on a Rust-enabled builder.
5. Run the app-server protocol smoke plus `meetai-agent` and `ppt-service`
   consumer smokes before publishing a new runtime image.
6. Stage the binary with its fork commit and SHA256, then update the runtime
   image reference atomically. Until then, retain the current image lock.

## Current compatibility boundaries

- `meetai-codex`: upstream source lock, app-server protocol, and reproducible
  binary build.
- `meetai-agent` / `ppt-service`: Codex adapter, trajectory capture, and
  consumer-level behavior.
- `llm-service`: Qwen serving, image input adaptation, model capabilities,
  runtime images, and NVIDIA/Ascend deployment concerns.
