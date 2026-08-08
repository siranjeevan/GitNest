#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = '1.0.0';
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
  https.get(url, (response) => {
    if (response.statusCode === 302 || response.statusCode === 301) {
      return download(response.headers.location, dest, cb);
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

download(downloadUrl, targetFile, () => {
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
