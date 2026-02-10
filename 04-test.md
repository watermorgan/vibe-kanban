# V2-I1-P0-05 测试说明（decision_resolved -> auto_resume）

## 测试范围
本次以链路验证为主，覆盖从审批决议到执行器恢复的关键节点。

## 用例设计
1. `approved` 路径：审批通过后 waiter 返回 `Approved`，执行器回传允许并继续执行。
2. `denied` 路径：审批拒绝后 waiter 返回 `Denied`，执行器回传 reject（携带理由），流程继续到可处理状态。
3. `timeout` 路径：审批超时后 waiter 返回 `TimedOut`，执行器回传 reject（超时提示）。
4. `pending` 异常路径：若 waiter 结束在 `Pending`，桥接层返回错误，避免不确定状态继续。

## 证据定位
- 审批提交：`frontend/src/hooks/useApprovalMutation.ts:15`
- 审批响应 API：`frontend/src/lib/api.ts:1173`
- 路由处理：`crates/server/src/routes/approvals.rs:23`
- waiter 唤醒：`crates/services/src/services/approvals.rs:149`
- 超时 watcher：`crates/services/src/services/approvals.rs:197`
- bridge 等待与取消：`crates/services/src/services/approvals/executor_approvals.rs:76`
- OpenCode reply 分支：`crates/executors/src/executors/opencode/sdk.rs:1355`

## 结果
- 静态代码链路检查：通过。
- 自动化/端到端执行：本轮未新增测试代码，未执行集成回放。

## 建议补测
1. 增加后端集成测试：覆盖 `POST /api/approvals/{id}/respond` 到 waiter 返回。
2. 增加执行器侧测试：覆盖 `Approved/Denied/TimedOut/Pending` 四分支。
