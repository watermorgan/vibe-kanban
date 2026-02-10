# 04-test

## 测试范围

- 目标模块：`crates/server/src/routes/starbus.rs`
- 目标能力：
  - active task 保留逻辑
  - active task 自愈逻辑
  - deterministic 选择逻辑
  - 全终态回退逻辑

## 新增单测

在 `routes::starbus::tests` 新增：

- `keeps_desired_active_when_present_and_non_terminal`
- `heals_missing_active_to_best_candidate`
- `heals_terminal_active_to_best_candidate`
- `chooses_priority_then_most_recent_update`
- `returns_none_when_all_tasks_terminal`

## 执行命令

```bash
cargo test -p server
```

## 测试结果

- `server` 包测试通过。
- 关键新增用例全部通过（5/5）。
- 未观察到与本任务相关的回归失败。

## 回归关注点

- 终态集合变更时需同步更新 `is_terminal_status` 与对应测试。
- 若新增 Starbus 状态，需同步更新 `status_priority` 与排序期望。

