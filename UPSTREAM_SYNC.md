# Upstream synchronization audit

This branch (`sync/upstream-823ea83`) records the first synchronization pass
against OpenAI Codex commit `823ea830c0fd418b09ff02d36cad9a1fff66465b`.

## Result

已补齐双方历史，共同祖先为
`1cc7e2361237ce7244430ee1d581c77f95c57ac8`。合并在
`sync/upstream-823ea83` 分支进行，旧基线保留于
`baseline/verified-20261005`。

合并仅发生两处文本冲突，均保留双方行为：`Session` 同时提供 MeetAI
应用上下文读取和上游 dynamic tools 读取；`rmcp-client` 同时导出 MeetAI
进度回调与上游 EMA OAuth 凭证接口。其余 MeetAI 补丁自动合并。

2026-10-05：合并后的 `cargo check --locked --bin codex-app-server -j 2`
通过（6 分 48 秒），`just fmt` 和 `just fmt-check` 通过；PPT 适配器与配置
26 项模拟服务测试通过。`just test` 运行的 14 项定向回归测试及 1 项 MCP
HTTP 进度通知集成测试全部通过。release 构建完成（41 分 56 秒）；原始与 stripped 二进制均通过真实 PPT
适配器握手/线程创建以及 Agent 请求字段握手/线程创建/线程列表检查。
已为本机 `/usr/bin/bwrap` 加载专用 AppArmor userns 许可；全局限制保持开启。
真实 `command/exec` 工作区内写入/读取通过，工作区外写入被拒绝，沙箱警告
消失；运行时镜像静态合同通过。本地 rootless Podman 候选镜像已构建，
镜像内握手、沙箱写入和外部写入拒绝通过（需取消 `/proc/*` 的容器屏蔽，
具体运行参数见 `MEETAI_BUILD_LOCK.md`）。完整模型任务仍待验证。

测试编译发现上游 `build_responses_request` 新增 `include_internal` 参数，已
适配本地 reasoning disabled 测试（生产源码已通过编译，无需改动调用路径）。

`just fix -p codex-core -p codex-rmcp-client -j 2` 检查完成（9 分 06 秒）。
Clippy 提出的冗余 clone 和上游测试 import 清理未纳入本轮同步；原有
dead-code 警告保留。未执行整个工作区测试套件。

相对上游，`app-server-protocol` 的本地差异仅涉及 MCP 进度事件不持久化到
线程历史及其测试。消费者使用的 thread/turn 方法、workspace/environment
字段仍存在，但完整模型任务兼容性需消费者端到端验证。

当前仍保留 Qwen 本地模型元数据、reasoning disabled、library scope、MCP
进度转发、线程搜索与 web search 补丁；模型配置和镜像职责向 `llm-service`
迁移应单独实施，避免把功能迁移混入上游同步。

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

同步合并提交：`43689d6cd36b46ee3bf78bc52128801851aa1044`，已推送同步分支。
新制品来源与哈希见 `MEETAI_BUILD_LOCK.md`，生产镜像锁未更新。
