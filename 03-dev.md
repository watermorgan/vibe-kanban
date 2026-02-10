# 03-dev（开发实现说明）

## 1. 任务背景

- 任务编号：`V2-I1-P0-04`
- 目标：对齐 Control Room 的状态/执行者/角色渲染，统一以后端 SoT（Source of Truth）为准。

## 2. SoT 定义

- 后端会话执行画像：`ExecutionProcess.executor_action.typ.executor_profile_id`
- 前端草稿画像：`DRAFT_FOLLOW_UP.executor_profile_id`
- 回退来源：当前会话 `session.executor`（无 variant 时）

## 3. 关键实现

### 3.1 Executor/Variant 选择逻辑重构

- 文件：`frontend/src/hooks/useExecutorSelection.ts`
- 变更点：
  - 入参由 `scratchVariant` 升级为 `scratchProfileId`
  - Executor 优先级调整为：`用户选择(可选)` > `scratch` > `latest process` > `config` > `first available`
  - 新增：
    - `allowUserSelection`：控制是否允许 UI 覆盖 executor
    - `scopeKey`：切换上下文时重置本地选择，避免跨会话污染

### 3.2 Variant 状态隔离

- 文件：`frontend/src/hooks/useVariant.ts`
- 变更点：
  - 新增 `scopeKey`
  - `scopeKey` 变化时重置 `hasUserSelectionRef`，恢复 SoT 驱动
  - 解决会话切换/审批切换/重试切换后的 variant 残留问题

### 3.3 SessionChatBoxContainer 对齐 SoT

- 文件：`frontend/src/components/ui-new/containers/SessionChatBoxContainer.tsx`
- 变更点：
  - `useExecutorSelection` 改为传入 `scratchProfileId`
  - 使用 `allowUserSelection: needsExecutorSelection`
  - 使用 `scopeKey: scratchId`
  - 修复 fallback executor：优先 `session.executor`，再退化到 `sessions[0].executor`

### 3.4 相关调用点补齐 scopeKey

- 文件：`frontend/src/components/tasks/TaskFollowUpSection.tsx`
  - `useVariant` 增加 `scopeKey: sessionId`
- 文件：`frontend/src/components/NormalizedConversation/RetryEditorInline.tsx`
  - `useVariant` 增加 `scopeKey: executionProcessId`

## 4. 影响范围

- 影响模块：
  - Workspaces Chat 输入区的 executor/variant 渲染与发送配置
  - Task Follow-up 区的 variant 选择
  - Retry Inline 编辑器的 variant 选择
- 不影响：
  - 后端 API 协议与数据结构
  - 数据库存储

## 5. 关联提交

- Commit: `c27d04b3`
- Message: `V2-I1-P0-04: align control room state with backend SoT`
