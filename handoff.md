# handoff（交接说明）

## 1. 当前状态

- 任务：`V2-I1-P0-04 Control room state consistency`
- 代码状态：已完成核心改造并提交
- 提交号：`c27d04b3`
- 提交信息：`V2-I1-P0-04: align control room state with backend SoT`

## 2. 本次落地内容

- 对齐 FE 渲染与后端 SoT（executor/variant）
- 修复会话 fallback executor 逻辑
- 增加 `scopeKey` 机制，避免跨上下文状态污染
- 改动文件：
  - `frontend/src/hooks/useExecutorSelection.ts`
  - `frontend/src/hooks/useVariant.ts`
  - `frontend/src/components/ui-new/containers/SessionChatBoxContainer.tsx`
  - `frontend/src/components/tasks/TaskFollowUpSection.tsx`
  - `frontend/src/components/NormalizedConversation/RetryEditorInline.tsx`

## 3. 文档清单

- `03-dev.md`：开发实现说明
- `04-test.md`：测试说明
- `06-audit.md`：审计说明
- `handoff.md`：本文件

## 4. 验证状态

- 已执行：`cargo check`（通过）
- 未执行：`pnpm run frontend:check`、`pnpm run frontend:lint`
  - 原因：当前环境 Node 版本 `v14.18.2`，项目要求 Node `>=18`

## 5. 接手后建议执行

1. 切换 Node 到 18+。
2. 安装依赖：`pnpm i`
3. 执行静态检查：
   - `pnpm run frontend:check`
   - `pnpm run frontend:lint`
4. 关键回归：
   - 会话切换（existing/new）
   - 审批态切换
   - retry process 切换

## 6. 风险与关注点

- 若仍出现渲染错配，重点排查：
  - `scopeKey` 是否按会话/审批/process 正确变化
  - `scratchProfileId` 是否正确加载
  - `latestProfileId` 是否异常退化

## 7. 回滚策略

- 代码回滚：回退提交 `c27d04b3`
- 风险隔离：优先回退 `useExecutorSelection` 与 `useVariant` 两个 hook 的行为改动
