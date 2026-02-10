# 03-dev

## 任务

- **任务编号**: `V2-I1-P0-03`
- **任务名称**: Active task hygiene
- **目标**: 保证全局仅有一个可恢复的 active task，并提供确定性的 resume 路径。

## 实现摘要

- 在 `crates/server/src/routes/starbus.rs` 增加 active task 选择与自愈逻辑。
- 新增状态判断与优先级规则：
  - 终态：`DONE` / `FAILED` / `CANCELLED`
  - 优先级：`EXECUTING` > `VERIFYING` > `AUDITING` > `DESIGNING` > `QUEUED` > `BLOCKED_HUMAN` > 其他
- 在以下 Starbus 入口统一执行 active task 归一化：
  - `GET /starbus/state`
  - `POST /starbus/intake/create`
  - `POST /starbus/state/next_action`
  - `POST /starbus/state/transition`
  - `POST /starbus/state/decision/resolve`
- 当 `active_task_id` 缺失、指向不存在任务或指向终态任务时，自动回填为确定性候选；若无可用候选，则回填 `None`。

## 确定性规则

1. 如果请求/现有 `active_task_id` 指向“存在且非终态”任务，保留该值。
2. 否则在所有非终态任务中排序选择：
   - 先按状态优先级；
   - 再按 `updated_at` 降序；
   - 再按 `created_at` 降序；
   - 最后按 `task_id` 升序作为最终 tie-breaker。
3. 没有非终态任务时返回 `None`。

## 兼容性与影响

- 数据模型未变更，无需 DB migration。
- API 结构未变更，行为变更为“读取/写入时自动修正 active task”。
- 对既有脏数据具备向前兼容能力（读取即修复）。

## 相关提交

- `6be66f29`

