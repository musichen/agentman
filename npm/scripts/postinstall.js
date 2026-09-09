const fs = require('node:fs');
const path = require('node:path');
const https = require('node:https');
const { pipeline } = require('node:stream/promises');

const packageRoot = path.resolve(__dirname, '..');
const binaryName = process.platform === 'win32' ? 'agentman-bin.exe' : 'agentman-bin';
const destination = path.join(packageRoot, 'bin', binaryName);
const arch = ({ darwin: { arm64: 'arm64' }, linux: { x64: 'x64', arm64: 'arm64' }, win32: { x64: 'x64' } }[process.platform] || {})[process.arch];
if (!arch) { console.error(`agentman: unsupported platform ${process.platform}/${process.arch}`); process.exit(1); }
const platform = process.platform === 'darwin' ? 'darwin' : process.platform === 'win32' ? 'windows' : 'linux';
const version = require(path.join(packageRoot, 'package.json')).version;
const asset = `agentman-v${version}-${platform}-${arch}${platform === 'windows' ? '.exe' : ''}`;
const url = `https://github.com/musichen/agentman/releases/download/v${version}/${asset}`;

function download(source) {
  https.get(source, response => {
    if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) return download(response.headers.location);
    if (response.statusCode !== 200) { console.error(`agentman: download failed (${response.statusCode})`); process.exit(1); }
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    pipeline(response, fs.createWriteStream(destination)).then(() => {
      if (process.platform !== 'win32') fs.chmodSync(destination, 0o755);
      console.log(`agentman: installed ${platform}/${arch}`);
    }).catch(error => { console.error(`agentman: ${error.message}`); process.exit(1); });
  }).on('error', error => { console.error(`agentman: ${error.message}`); process.exit(1); });
}
download(url);
