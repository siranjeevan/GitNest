#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = '1.0.4';
const REPO = 'siranjeevan/GitNest';

const platform = process.platform;
const arch = process.arch;

let artifactName = '';

if (platform === 'darwin') {
  artifactName = arch === 'arm64' ? `gitnest-v${VERSION}-macos-arm64.tar.gz` : `gitnest-v${VERSION}-macos-x86_64.tar.gz`;
} else if (platform === 'linux') {
  if (arch === 'x64') {
    artifactName = `gitnest-v${VERSION}-linux-x86_64.tar.gz`;
  } else {
    console.error(`[gitnest] Unsupported Linux architecture: ${arch}`);
    process.exit(1);
  }
} else if (platform === 'win32') {
  artifactName = `gitnest-v${VERSION}-windows-x86_64.zip`;
} else {
  console.error(`[gitnest] Unsupported platform: ${platform}`);
  process.exit(1);
}

const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${artifactName}`;
const binDir = path.join(__dirname, '..', 'vendor');
const targetFile = path.join(binDir, artifactName);

if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

console.log(`[gitnest] Downloading binary for ${platform}-${arch} from GitHub Releases...`);

function download(url, dest, cb) {
  const file = fs.createWriteStream(dest);
  const options = {
    headers: {
      'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) GitNest/1.0'
    }
  };
  https.get(url, options, (response) => {
    if (response.statusCode === 302 || response.statusCode === 301) {
      return download(response.headers.location, dest, cb);
    }
    if (response.statusCode !== 200) {
      fs.unlinkSync(dest);
      return cb(new Error(`HTTP ${response.statusCode}`));
    }
    response.pipe(file);
    file.on('finish', () => {
      file.close(cb);
    });
  }).on('error', (err) => {
    fs.unlinkSync(dest);
    console.error(`[gitnest] Download failed: ${err.message}`);
    process.exit(1);
  });
}

download(downloadUrl, targetFile, (err) => {
  if (err) {
    console.log(`[gitnest] Pre-built release download unavailable (${err.message}). Fallback: Compiling from source...`);
    try {
      execSync(`cargo build --release`, { cwd: path.join(__dirname, '..'), stdio: 'inherit' });
      const srcBin = platform === 'win32'
        ? path.join(__dirname, '..', 'target', 'release', 'gitnest.exe')
        : path.join(__dirname, '..', 'target', 'release', 'gitnest');
      const targetBin = platform === 'win32'
        ? path.join(binDir, 'gitnest.exe')
        : path.join(binDir, 'gitnest');
      fs.copyFileSync(srcBin, targetBin);
      if (platform !== 'win32') {
        fs.chmodSync(targetBin, 0o755);
      }
      console.log(`[gitnest] Source build & installation successful!`);
    } catch (buildErr) {
      console.error(`[gitnest] Build failed: ${buildErr.message}`);
      process.exit(1);
    }
    return;
  }

  console.log(`[gitnest] Extracting binary...`);
  try {
    if (artifactName.endsWith('.tar.gz')) {
      execSync(`tar -xzf "${targetFile}" -C "${binDir}"`);
    } else if (artifactName.endsWith('.zip')) {
      execSync(`powershell -Command "Expand-Archive -Path '${targetFile}' -DestinationPath '${binDir}' -Force"`);
    }
    fs.unlinkSync(targetFile);

    const binaryName = platform === 'win32' ? 'gitnest.exe' : 'gitnest';
    const binaryPath = path.join(binDir, binaryName);
    if (platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }
    console.log(`[gitnest] Installation successful!`);
  } catch (e) {
    console.error(`[gitnest] Extraction failed: ${e.message}`);
    process.exit(1);
  }
});
