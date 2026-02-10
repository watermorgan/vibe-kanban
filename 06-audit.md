# 06-audit（变更审计）

## 1. 变更概览

- 审计对象：`V2-I1-P0-04` Control Room 状态一致性改造
- 核心方向：将 FE 的 executor/variant 推导逻辑对齐到后端 SoT
- 提交：`c27d04b3`

## 2. 审计结论

- 结论：本次为前端选择与状态同步逻辑修正，属于低风险行为一致性改造。
- 风险等级：`Low-Medium`
  - Low：未改后端接口、未改数据库
  - Medium：涉及输入区执行画像选择，若逻辑错误会影响用户发送到错误 executor/variant

## 3. 代码审计点

### 3.1 正确性

- `useExecutorSelection` 优先级明确，且支持按上下文重置本地状态。
- `SessionChatBoxContainer` fallback 修复后，不再默认误用 `sessions[0]`。
- `useVariant` 的 `scopeKey` 机制能避免跨上下文残留。

### 3.2 一致性

- SoT 与 UI 显示/发送参数同源：
  - scratch：`DRAFT_FOLLOW_UP.executor_profile_id`
  - process：`executor_action.typ.executor_profile_id`

### 3.3 回归风险

- 主要回归点：
  - 新会话模式与已有会话模式切换
  - 审批态 scratch key 切换
  - retry 场景 process 切换

## 4. 安全与合规

- 本次变更不涉及：
  - 权限模型
  - 认证授权
  - 敏感数据存储/传输
- 无新增外部依赖，无新增网络调用。

## 5. 可观测性与排障建议

- 若出现 executor/variant 错配，优先检查：
  - `scopeKey` 是否按上下文变化
  - `scratchProfileId` 是否为空或过期
  - `latestProfileId` 来源是否被 fallback 覆盖

## 6. 后续审计建议

1. 增加 `useExecutorSelection` / `useVariant` 单元测试，覆盖优先级和重置行为。
2. 增加 E2E 场景：会话切换 + 审批切换 + retry 切换的组合路径。
