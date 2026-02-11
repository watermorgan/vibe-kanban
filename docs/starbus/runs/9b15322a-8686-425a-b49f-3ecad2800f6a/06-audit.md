# 06-audit

Status: APPROVE_NEXT (with iteration backlog)

## Audit summary

- REQ gate artifacts exist and are coherent:
  - `01-clarify.md`
  - `02-design.md`
- DEV implementation covers P0 route surface and core guards.
- TEST baseline passed for compile and API smoke.
- Critical path fix (repo-local run output directory) has been implemented and verified.

## Quality checks

1. Contract consistency
- PASS: starbus endpoints are exposed under `/api/starbus/*`.
- PASS: frontend adapter uses those endpoints.

2. Safety/guard rails
- PASS: invalid transitions blocked by `is_valid_transition`.
- PASS: tool actors blocked from terminal status writes.
- PASS: blocked->resume behavior requires explicit or inferred gate path.

3. Traceability
- PASS: role prompts and runbook are present under `docs/starbus/`.
- PARTIAL: automatic phase-to-file binding still requires improvement (tracked in `05-iterate.md`).

## Decision

- `APPROVE_NEXT` for V2-I1 P0 baseline.
- Continue immediately with items in `05-iterate.md` (I1-P0-01, I1-P0-02, I1-P0-03).

---

## Round 2 audit (requirements vs code vs board)

Status: NEEDS_REVISION

### Scope

- Verify current V2-I1 board state against StarBus file contract.
- Validate that task dispatch/prompt contract is deterministic per task id.
- Check requirement-code consistency for mandatory phase artifacts.

### Evidence

- Integrity report:
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`
- Runtime check command:
  - `node scripts/check-starbus-inreview.js`
- Contract implementation:
  - `crates/server/src/routes/starbus.rs`
- Prompt pack and role prompts:
  - `docs/starbus/prompts/v2-i1-role-prompt-pack.md`
  - `docs/starbus/prompts/v2-i1-req-prompt.md`
  - `docs/starbus/prompts/v2-i1-dev-prompt.md`
  - `docs/starbus/prompts/v2-i1-test-prompt.md`
  - `docs/starbus/prompts/v2-i1-accept-prompt.md`

### Findings

1. In-review artifact completeness is not yet met for the current V2-I1 set.
   - Current check result: `inreview=13 pass=1 fail=12`.
   - Multiple in-review tasks still miss one or more required files:
     - `task.md`, `context.md`, `playbook.md`, `03-dev.md`, `04-test.md`, `06-audit.md`, `handoff.md`.

2. Dispatch contract is corrected for new runs.
   - Backend dispatch prompt now binds mandatory outputs to task-scoped run path.
   - Prompt templates now use placeholders (`<task-id>`, `<task-artifact-key>`) instead of hardcoded `TASK-010`.

3. Existing in-review tasks created before the contract fix still violate the new contract and require cleanup/backfill.

### Decision

- `NEEDS_REVISION` for this review round.

### Required actions (gate to close this round)

1. Reconcile all V2-I1 in-review tasks:
   - Either move non-ready tasks out of `inreview`, or backfill all required artifacts.
2. Re-run integrity checker until:
   - `fail=0` for V2-I1 in-review tasks.
3. Append one final PASS snapshot to artifacts:
   - Updated `inreview-integrity-report.json` with `pass_count == total_inreview`.

### Round 2 closure

Status: APPROVE_NEXT

- Reconciliation applied:
  - non-ready tasks were moved out of `inreview`
  - active execution task kept in `inprogress`
- Latest integrity result:
  - `inreview=1 pass=1 fail=0`
- Evidence:
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`

---

## Round 3 audit (全面一致性审计 - role-qa-security + role-architect-auditor)

**审计时间**: 2026-02-10  
**审计范围**: V2-I1融合（Star-State-Bus + vibe-kanban）在"需求、设计、代码、运行状态、证据链"上的一致性  
**审计结论**: **NEEDS_REVISION** (需修复后进入下一迭代)

### 审计发现 - 已满足项 ✅

#### AUD-SAT-01: 编译状态和基础设施 (P0)
- **证据**: `cargo check -p server` 通过 (仅1个warning，可忽略)
- **验证**: Rust后端编译路径正常，核心API路由完整
- **评估**: 符合Gate2编译要求

#### AUD-SAT-02: API接口契约完整性 (P0)  
- **证据**: 所有关键API端点已实现并验证
  - `GET /api/starbus/state` ✅
  - `POST /api/starbus/intake/preflight` ✅  
  - `POST /api/starbus/intake/create` ✅
  - `POST /api/starbus/run-role-task` ✅
  - `POST /api/starbus/state/decision/resolve` ✅
- **验证**: 前端`starbusApi`客户端完整集成所有后端端点
- **评估**: 符合设计中的API使用要求

#### AUD-SAT-03: 状态机转换守卫 (P0)
- **证据**: `is_valid_transition`函数正确实现，包含:
  - BLOCKED_HUMAN专用转换路径
  - 工具actor终端状态写保护  
  - next_action自动推导逻辑
- **验证**: 代码分析显示转换守卫符合runbook要求
- **评估**: 状态机核心安全性满足设计要求

### 审计发现 - 待修复项 ❌

#### AUD-001: in-review完整性不一致 (P0)
- **严重级别**: P0
- **问题描述**: 当前in-review任务完整性检查失败
  - 当前状态: `inreview=13 pass=12 fail=1`  
  - 失败任务: `9b15322a-8686-425a-b49f-3ecad2800f6a` (V2-I1-P0-01 Canonical run output binding)
  - 缺失文件: `03-dev.md`, `04-test.md`, `06-audit.md`, `handoff.md`
- **证据**: 
  - `node scripts/check-starbus-inreview.js` 输出
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`
- **修复建议**: 
  1. 将任务`9b15322a-8686-425a-b49f-3ecad2800f6a`移回`inprogress`或移除`inreview`状态
  2. 或回填缺失的必需文件以符合in-review契约
- **验收标准**: `node scripts/check-starbus-inreview.js` 显示 `fail=0`

#### AUD-002: 路径绑定硬编码残留 (P1)  
- **严重级别**: P1
- **问题描述**: 虽然prompt模板已更新使用`<task-id>`占位符，但缺少task-alias机制的实现
  - 设计要求: 引入run alias (如`TASK-010`)实现规范化输出路径
  - 当前状态: 占位符已就位，但别名映射和规范化输出逻辑未实现
- **证据**:
  - `docs/starbus/prompts/v2-i1-role-prompt-pack.md` 中仍存在`<task-id>`占位符
  - `docs/starbus/runs/TASK-010/05-iterate.md` 中I1-P0-01需求未完成
- **修复建议**:
  1. 在task state metadata中增加`alias`字段
  2. 实现`<task-id>`到alias路径的映射逻辑
  3. 更新dispatch prompt以使用规范化路径
- **验收标准**: 
  - `GET /api/starbus/state`返回包含alias元数据
  - 新任务执行创建规范化的`docs/starbus/runs/TASK-010/`路径

#### AUD-003: 重复标题防护缺失 (P1)
- **严重级别**: P1  
- **问题描述**: 重复任务创建保护未实现，违反数据完整性要求
  - 设计要求: `I1-P0-02 Duplicate title protection`应在后端验证同名任务
  - 当前状态: 可以在同一项目中创建相同标题的active任务
- **证据**:
  - `04-test.md`明确标注为"Known limits kept for next iteration"
  - 代码审查显示`run-role-task`和`intake/create`缺少重复检测
- **修复建议**:
  1. 在`crates/server/src/routes/starbus.rs`中增加标题重复检测
  2. 为intentional duplicates提供`override`标志
  3. 返回清晰的4xx错误信息
- **验收标准**:
  - API测试显示重复创建被阻止
  - 带`override=true`的创建请求成功

#### AUD-004: 活跃任务卫生机制不完整 (P1)
- **严重级别**: P1
- **问题描述**: 全局活跃任务管理和清理机制未完全实现
  - 设计要求: `I1-P0-03 Active task hygiene`需要显式active task设置API
  - 当前状态: 22个任务存在于state中，缺少清理和切换机制
- **证据**:
  - `curl -s http://127.0.0.1:3007/api/starbus/state | jq '.data.tasks | length'` 返回22
  - 历史scratch任务仍在全局状态中
- **修复建议**:
  1. 增加显式设置/清除active task的API端点
  2. 实现stale-task清理端点 (soft archive或inactive标记)
  3. 提供任务切换的确定性API
- **验收标准**:
  - API支持显式active task切换
  - 操作员视图默认不显示历史scratch任务

#### AUD-005: 自动resume逻辑证据不足 (P2)
- **严重级别**: P2
- **问题描述**: auto-resume功能的端到端验证证据不完整
  - 设计要求: 决策解决后的自动恢复流程需要验证
  - 当前状态: 代码中存在`auto_resume`逻辑，但缺少端到端测试证据
- **证据**:
  - `resolve_decision`函数包含auto-resume逻辑
  - 但`04-test.md`中未包含完整的auto-resume测试场景
- **修复建议**:
  1. 增加auto-resume的集成测试用例
  2. 在`artifacts/README.md`中记录测试证据
  3. 验证所有决策解决后的状态转换
- **验收标准**:
  - 测试文档包含auto-resume场景
  - 证据链可重现auto-resume流程

### 接口契约验证结果

#### 已验证契约 ✅
1. **状态映射一致性**: `BLOCKED_HUMAN` ↔ `inreview` 正确映射
2. **决策解析契约**: `resolve_decision`接口符合规范
3. **前端集成完整性**: `starbusApi`包含所有必需方法

#### 待改进契约 ⚠️  
1. **输出路径绑定**: 缺少任务级绑定，仍依赖硬编码`TASK-010`
2. **任务别名机制**: 未实现task-alias到规范路径的映射

### 证据链完整性评估

#### 完整证据链 ✅
- Phase artifacts: `01-clarify.md` → `02-design.md` → `03-dev.md` → `04-test.md` → `06-audit.md`
- Runtime evidence: 编译检查、API smoke测试、路径验证测试
- Integrity reports: `inreview-integrity-report.json` (需要更新)

#### 缺失证据链 ❌
- Auto-resume端到端测试证据
- 重复标题防护测试证据  
- 活跃任务卫生API验证证据

### Gate级别结论

**当前Gate状态**: **Gate2 (部分达成)**

- **Gate0 (REQ)**: ✅ 完成 - 需求澄清和范围定义完整
- **Gate1 (DESIGN)**: ✅ 完成 - API契约和架构设计可执行  
- **Gate2 (IMPLEMENTATION)**: ⚠️ 部分完成 - 核心功能实现，但缺少P1防护机制
- **Gate3 (AUDIT & RELEASE)**: ❌ 未达成 - 存在P0完整性问题，需修复后重新审计

### 最终审计决策

**决策**: **NEEDS_REVISION** 

**理由**:
1. **P0阻塞**: AUD-001 in-review完整性问题必须立即解决
2. **P1债务**: AUD-002/003/004代表关键防护机制的缺失，影响系统稳定性
3. **证据不足**: 关键功能的端到端验证证据链不完整

**下一迭代优先级**:
1. **立即修复**: AUD-001 (in-review完整性) - 阻塞发布
2. **高优先级**: AUD-002 (路径绑定), AUD-003 (重复防护), AUD-004 (任务卫生)  
3. **中优先级**: AUD-005 (auto-resume证据) - 增强验证覆盖

**重新审计条件**:
- `node scripts/check-starbus-inreview.js` 显示 `fail=0`
- 所有P1项有可演示的修复证据
- 完整的端到端测试证据链可重现

---

**审计完成时间**: 2026-02-10  
**下一审计准备**: 完成上述修复后，重新运行完整审计流程
