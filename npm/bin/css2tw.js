#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');

// Ideally, this script detects the OS and architecture, then executes the correct prebuilt binary.
// For development purposes, it proxies to `cargo run`.
const args = process.argv.slice(2);

const result = spawnSync('cargo', ['run', '--bin', 'css2tw', '--', ...args], {
  stdio: 'inherit',
  cwd: path.resolve(__dirname, '../..')
});

process.exit(result.status);
