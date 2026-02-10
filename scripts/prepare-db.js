#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const checkMode = process.argv.includes('--check');

console.log(checkMode ? 'Checking SQLx prepared queries...' : 'Preparing database for SQLx...');

// Change to backend directory
const backendDir = path.join(__dirname, '..', 'crates/db');
process.chdir(backendDir);

// Create temporary database file
const dbFile = path.join(backendDir, 'prepare_db.sqlite');
fs.writeFileSync(dbFile, '');

try {
  // Get absolute path (cross-platform)
  const dbPath = path.resolve(dbFile);
  const databaseUrl = `sqlite:${dbPath}`;

  console.log(`Using database: ${databaseUrl}`);

  // Run migrations
  console.log('Running migrations...');
  execSync('cargo sqlx migrate run', {
    stdio: 'inherit',
    env: { ...process.env, DATABASE_URL: databaseUrl }
  });

  // Prepare queries with workspace-specific optimizations
  const sqlxBaseCommand = checkMode
    ? 'cargo sqlx prepare --check --profile sqlx-prepare'
    : 'cargo sqlx prepare --profile sqlx-prepare';

  // Use optimized build settings for sqlx prepare
  // SQLX_OFFLINE=false ensures we actually prepare (not just check cache)
  console.log(checkMode ? 'Checking prepared queries...' : 'Preparing queries...');

  const prepareEnv = {
    ...process.env,
    DATABASE_URL: databaseUrl,
    SQLX_OFFLINE: 'false',
  };

  execSync(sqlxBaseCommand, {
    stdio: 'inherit',
    env: prepareEnv
  });

  console.log(checkMode ? 'SQLx check complete!' : 'Database preparation complete!');

} finally {
  // Clean up temporary file
  if (fs.existsSync(dbFile)) {
    fs.unlinkSync(dbFile);
  }
}