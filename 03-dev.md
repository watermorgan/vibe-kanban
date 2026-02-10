# V2-I1-P0-05 开发说明（decision_resolved -> auto_resume）

## 目标
建立并确认 `decision_resolved -> auto_resume` 运行链路，确保审批决议后执行器可自动恢复，无需二次人工触发。

## 实现链路
1. 前端提交决议（approve/deny）。
2. 后端审批路由接收并转发到审批服务。
3. 审批服务写回状态并通过 `oneshot` 唤醒等待方。
4. 执行器桥接层收到 waiter 结果，返回 `ApprovalStatus`。
5. 执行器向底层会话回传 permission reply，流程自动继续。

## 关键实现点
- 前端审批提交：`frontend/src/hooks/useApprovalMutation.ts:13`
- 审批 API 封装：`frontend/src/lib/api.ts:1167`
- 审批路由入口：`crates/server/src/routes/approvals.rs:16`
- 审批状态写回与 waiter 唤醒：`crates/services/src/services/approvals.rs:141`
- Executor 审批等待接口：`crates/executors/src/approvals.rs:31`
- Executor bridge 等待并返回状态：`crates/services/src/services/approvals/executor_approvals.rs:38`
- OpenCode 执行器处理审批回复并继续：`crates/executors/src/executors/opencode/sdk.rs:1321`
- OpenCode 审批响应事件定义：`crates/executors/src/executors/opencode/types.rs:8`

## 结论
当前代码路径已具备从审批决议到自动恢复执行的闭环；实现形态是“审批状态驱动 + waiter 唤醒”，而不是显式命名事件 `auto_resume`。
