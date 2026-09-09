#!/usr/bin/env node
const { spawn } = require('node:child_process');
const path = require('node:path');
const binary = path.join(__dirname, process.platform === 'win32' ? 'agentman-bin.exe' : 'agentman-bin');
const child = spawn(binary, process.argv.slice(2), { stdio: 'inherit' });
child.on('error', error => { console.error(`agentman binary unavailable: ${error.message}`); process.exit(1); });
child.on('exit', (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
