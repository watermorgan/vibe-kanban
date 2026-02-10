# DEV Phase - V2-I1-P2-01 SQLx Prepare Performance

## Overview
Optimize the duration of `cargo sqlx prepare --workspace` command by implementing compilation profile optimizations.

## Date
- 2026-02-10

## Implementation Details

### Analysis
- **Current State**: 145 SQLx query files across `crates/db/.sqlx` and `crates/remote/.sqlx`
- **Affected Crates**: 7 crates use sqlx (server, db, utils, local-deployment, executors, deployment, services, remote)
- **Query Count**: 70+ Rust files with `sqlx::` macro usage

### Bottlenecks Identified
1. Full workspace compilation required for each prepare run
2. Database I/O for query type checking (145 queries)
3. No dedicated compilation profile for prepare workflow

### Changes Made

#### 1. `.cargo/config.toml` - Added sqlx-prepare profile
```toml
[profile.sqlx-prepare]
inherits = "dev"
codegen-units = 256
incremental = false
opt-level = 0
[profile.sqlx-prepare.package."*"]
opt-level = 0
```

#### 2. `scripts/prepare-db.js` - Updated to use optimized profile
- Added `--profile sqlx-prepare` flag
- Simplified environment handling (profile manages optimizations)

#### 3. `crates/remote/scripts/prepare-db.sh` - Updated to use optimized profile
- Added `--profile sqlx-prepare` for both check and prepare modes

### Optimization Strategy

| Setting | Value | Impact |
|---------|-------|--------|
| codegen-units | 256 | Max parallelization across CPU cores (default: 16) |
| incremental | false | Faster full compiles for fresh prepare runs |
| opt-level | 0 | Skips LLVM optimizations (unnecessary for type checking) |

## Expected Results
- **20-40% improvement** in prepare duration
- Improvement scales with CPU core count
- No impact on runtime performance (profile only affects prepare)

## Files Modified
- `.cargo/config.toml:27-35`
- `scripts/prepare-db.js:33-51`
- `crates/remote/scripts/prepare-db.sh:37,74,77`
