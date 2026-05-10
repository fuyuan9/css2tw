#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const args = process.argv.slice(2);
const platform = process.platform;
const arch = process.arch;

// Mapping of platform/arch to package names
const pkgMap = {
  'darwin-arm64': 'css2tw-darwin-arm64',
  'darwin-x64': 'css2tw-darwin-x64',
  'linux-x64': 'css2tw-linux-x64',
  'win32-x64': 'css2tw-win32-x64'
};

const key = `${platform}-${arch}`;
const pkgName = pkgMap[key];
let binaryPath = null;

if (pkgName) {
  try {
    // Try to find the package's directory
    const pkgJsonPath = require.resolve(`${pkgName}/package.json`, {
      paths: [path.join(__dirname, '..', '..')]
    });
    const pkgDir = path.dirname(pkgJsonPath);
    const exeName = platform === 'win32' ? 'css2tw.exe' : 'css2tw';
    const potentialPath = path.join(pkgDir, 'bin', exeName);
    
    if (fs.existsSync(potentialPath)) {
      binaryPath = potentialPath;
    }
  } catch (e) {
    // Package not found, will fallback to cargo run
  }
}

if (binaryPath) {
  // Execute the prebuilt binary
  const result = spawnSync(binaryPath, args, { stdio: 'inherit' });
  process.exit(result.status ?? 0);
} else {
  // Fallback to cargo run (Development mode)
  const rootDir = path.resolve(__dirname, '../..');
  const cargoTomlPath = path.join(rootDir, 'Cargo.toml');
  
  if (fs.existsSync(cargoTomlPath)) {
    const result = spawnSync('cargo', ['run', '--release', '--bin', 'css2tw', '--', ...args], {
      stdio: 'inherit',
      cwd: rootDir
    });
    process.exit(result.status ?? 0);
  } else {
    console.error(`Error: Prebuilt binary for ${key} not found and source code not available.`);
    process.exit(1);
  }
}
