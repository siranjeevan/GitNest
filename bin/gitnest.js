#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const platform = process.platform;
const binaryName = platform === 'win32' ? 'gitnest.exe' : 'gitnest';
const binaryPath = path.join(__dirname, '..', 'vendor', binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`[gitnest] Binary not found at ${binaryPath}. Re-running postinstall...`);
  require('./install');
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env,
});

child.on('exit', (code, signal) => {
  if (code !== null) {
    process.exit(code);
  } else if (signal) {
    process.kill(process.pid, signal);
  }
});
