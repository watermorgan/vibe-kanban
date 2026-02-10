# 04-test（测试说明）

## 1. 测试目标

- 验证 Control Room 相关 executor/variant 渲染与后端 SoT 一致。
- 验证上下文切换后不会复用旧会话的本地选择状态。

## 2. 测试范围

- `SessionChatBoxContainer` 的 executor/variant 选择与回退逻辑
- `useExecutorSelection` 的优先级与重置行为
- `useVariant` 的 `scopeKey` 重置行为
- `TaskFollowUpSection` 与 `RetryEditorInline` 的调用链行为

## 3. 已执行检查

### 3.1 后端编译检查

- 命令：`cargo check`
- 结果：通过

### 3.2 前端类型检查

- 命令：`pnpm run frontend:check`（未在当前环境执行成功）
- 阻塞原因：当前环境 Node.js 版本为 `v14.18.2`，项目要求 `>=18`

## 4. 建议补充验证（Node 18+ 环境）

1. 执行静态检查
   - `pnpm run frontend:check`
   - `pnpm run frontend:lint`

2. 人工回归（关键路径）
   - 进入已有会话：确认 executor/variant 来源于该会话 SoT，不被其他会话污染
   - 切换会话：确认 variant 会随 `scopeKey` 重置为当前 SoT
   - 审批态切换：确认输入区 executor/variant 不串线
   - 重试编辑器（Retry）：切换不同 process，variant 应按 process SoT 切换

3. 边界场景
   - 无 scratch、有 process
   - 有 scratch、无 process
   - 无 scratch、无 process、仅 `session.executor`

## 5. 验收标准

- executor/variant 的展示和发送参数与当前上下文 SoT 一致
- 不存在跨 session / 跨 process 的旧选择泄漏
- 前端类型检查与 lint 全绿
