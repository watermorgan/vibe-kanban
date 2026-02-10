# handoff

## 本次交付

- **任务**: `V2-I1-P0-03 Active task hygiene`
- **提交**: `6be66f29`
- **范围**: 后端 Starbus active task 规则统一与自愈；新增单测覆盖核心场景。

## 已完成事项

- 实现 active task 选择函数，确保 resume 路径确定性。
- 在 Starbus 状态读取与关键写接口中加入 active task 归一化与必要写回。
- 增加 5 个单测覆盖：
  - 有效 active 保留
  - active 缺失修复
  - active 指向终态修复
  - 优先级与更新时间排序
  - 全终态返回 `None`
- 执行 `cargo test -p server` 并通过。

## 关键文件

- `crates/server/src/routes/starbus.rs`
- `03-dev.md`
- `04-test.md`
- `06-audit.md`

## 运行与验证

```bash
cargo test -p server
```

可选联调验证：

1. 调用 `GET /starbus/state`，确认无效 active 会被修复。
2. 调用 `POST /starbus/state/transition` 将 active 任务推进到终态，确认 active 自动切换到下一候选或 `None`。

## 待办（建议）

1. 增加 Starbus API 集成测试，覆盖真实 DB 读写链路。
2. 若后续新增状态，更新优先级映射与测试期望。
3. 如需前端展示 deterministic resume 依据，可在 UI 增加 active 来源说明。

