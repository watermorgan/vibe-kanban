#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const http = require('http');
const https = require('https');

const API_BASE = process.env.STARBUS_API_BASE || 'http://127.0.0.1:3007';
const PROJECT_ID =
  process.env.STARBUS_PROJECT_ID || '46419dc8-19dc-4d10-9930-58bd0ab3be8f';
const ROOT = path.resolve(__dirname, '..');
const RUNS_ROOT = path.join(ROOT, 'docs', 'starbus', 'runs');
const RUN_ALIAS = process.env.STARBUS_RUN_ALIAS || 'TASK-010';
const REPORT_PATH = path.join(
  ROOT,
  'artifacts',
  'TASK-010-STARBUS-VIBE-INTEGRATION-MVP',
  'inreview-integrity-report.json',
);

const REQUIRED = [
  'task.md',
  'context.md',
  'playbook.md',
  '03-dev.md',
  '04-test.md',
  '06-audit.md',
  'handoff.md',
];

async function fetchJson(url) {
  return new Promise((resolve, reject) => {
    const mod = url.startsWith('https:') ? https : http;
    const req = mod.get(url, (res) => {
      let body = '';
      res.setEncoding('utf8');
      res.on('data', (chunk) => {
        body += chunk;
      });
      res.on('end', () => {
        if (res.statusCode < 200 || res.statusCode >= 300) {
          reject(new Error(`HTTP ${res.statusCode} for ${url}`));
          return;
        }
        try {
          resolve(JSON.parse(body));
        } catch (err) {
          reject(new Error(`Invalid JSON from ${url}: ${err.message}`));
        }
      });
    });
    req.on('error', (err) => reject(err));
  });
}

function readRunArtifacts(taskId, title) {
  const canonicalDir = path.join(RUNS_ROOT, taskId);
  let dir = canonicalDir;
  let aliasUsed = false;
  // Fallback for V2 migration period where outputs are still under TASK-010 alias.
  if (!fs.existsSync(canonicalDir) && /^V2-I1|^TASK-010/i.test(title || '')) {
    const aliasDir = path.join(RUNS_ROOT, RUN_ALIAS);
    if (fs.existsSync(aliasDir)) {
      dir = aliasDir;
      aliasUsed = true;
    }
  }
  const exists = fs.existsSync(dir);
  const files = {};
  for (const file of REQUIRED) {
    files[file] = exists && fs.existsSync(path.join(dir, file));
  }
  return { dir, exists, files, aliasUsed };
}

function mainStatus(starbusTask) {
  if (!starbusTask) return 'MISSING_IN_STARBUS';
  return starbusTask.status;
}

async function main() {
  const [tasksRes, stateRes] = await Promise.all([
    fetchJson(`${API_BASE}/api/tasks?project_id=${PROJECT_ID}`),
    fetchJson(`${API_BASE}/api/starbus/state`),
  ]);

  const inReview = (tasksRes.data || []).filter((t) => t.status === 'inreview');
  const starbusMap = new Map((stateRes.data.tasks || []).map((t) => [t.task_id, t]));

  const report = {
    generated_at: new Date().toISOString(),
    api_base: API_BASE,
    project_id: PROJECT_ID,
    total_inreview: inReview.length,
    pass_count: 0,
    fail_count: 0,
    items: [],
  };

  for (const task of inReview) {
    const artifacts = readRunArtifacts(task.id, task.title);
    const missing = REQUIRED.filter((f) => !artifacts.files[f]);
    const starbusTask = starbusMap.get(task.id);
    const status = mainStatus(starbusTask);
    const pass = missing.length === 0;

    if (pass) {
      report.pass_count += 1;
    } else {
      report.fail_count += 1;
    }

    report.items.push({
      task_id: task.id,
      title: task.title,
      kanban_status: task.status,
      starbus_status: status,
      run_dir_exists: artifacts.exists,
      run_dir: artifacts.dir,
      alias_used: artifacts.aliasUsed,
      missing_artifacts: missing,
      all_required_present: pass,
    });
  }

  fs.mkdirSync(path.dirname(REPORT_PATH), { recursive: true });
  fs.writeFileSync(REPORT_PATH, JSON.stringify(report, null, 2));

  console.log(`Report written: ${REPORT_PATH}`);
  console.log(
    `inreview=${report.total_inreview} pass=${report.pass_count} fail=${report.fail_count}`,
  );

  if (report.fail_count > 0) {
    console.log('Failed task IDs:');
    for (const item of report.items.filter((i) => !i.all_required_present)) {
      console.log(`- ${item.task_id} (${item.title})`);
    }
    process.exitCode = 2;
  }
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
