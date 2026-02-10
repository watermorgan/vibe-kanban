# V2-I1-P0-05 审计记录（decision_resolved -> auto_resume）

## 审计结论
`decision_resolved -> auto_resume` 证据链完整，可追溯到前端触发、后端决议处理、waiter 唤醒、执行器恢复四层。

## Evidence Chain
1. 决议产生（用户审批）：
   - `frontend/src/hooks/useApprovalMutation.ts:15`
   - `frontend/src/lib/api.ts:1173`
2. 决议入站（后端 API）：
   - `crates/server/src/routes/approvals.rs:16`
   - `crates/server/src/routes/approvals.rs:23`
3. 决议落地并唤醒：
   - `crates/services/src/services/approvals.rs:147`
   - `crates/services/src/services/approvals.rs:149`
4. 自动恢复（waiter 返回后继续）：
   - `crates/services/src/services/approvals/executor_approvals.rs:76`
   - `crates/executors/src/executors/opencode/sdk.rs:1321`
   - `crates/executors/src/executors/opencode/sdk.rs:1393`

## 关键审计点
- 审批处理不是“仅写状态”，而是显式 `send` 到 waiter 通道，具备恢复触发因果性。
- 自动恢复由执行器等待逻辑消费 `ApprovalStatus` 驱动，非前端二次触发。
- OpenCode 路径具备明确 reply 映射：
  - `Approved -> once`
  - `Denied/TimedOut/Pending -> reject + message`

## 风险与缺口
1. 缺少统一命名的审计事件（例如 `decision_resolved`、`auto_resume`）持久化记录。
2. 当前证据依赖代码路径与运行日志，若需合规审计建议新增结构化事件存储。
