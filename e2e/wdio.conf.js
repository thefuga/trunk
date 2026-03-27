// Adapted from https://github.com/tauri-apps/webdriver-example/blob/main/v2/webdriver/webdriverio/wdio.conf.js
import os from 'os';
import path from 'path';
import { existsSync } from 'fs';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

let tauriDriver;
let exit = false;

const binaryPath = path.resolve(
  __dirname,
  '../src-tauri/target/debug/trunk',
);

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./specs/**/*.e2e.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: binaryPath,
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // Build the debug binary before any test runs (skip if E2E_SKIP_BUILD is set)
  onPrepare: () => {
    if (!process.env.E2E_SKIP_BUILD) {
      console.log('Building debug binary...');
      const result = spawnSync(
        'bun',
        ['run', 'tauri', 'build', '--', '--debug', '--no-bundle'],
        {
          cwd: path.resolve(__dirname, '..'),
          stdio: 'inherit',
          shell: true,
        },
      );
      if (result.status !== 0) {
        throw new Error(`Build failed with exit code ${result.status}`);
      }
    } else {
      console.log('Skipping build (E2E_SKIP_BUILD is set)');
    }

    if (!existsSync(binaryPath)) {
      throw new Error(`Binary not found at ${binaryPath}`);
    }
  },

  // Start tauri-driver before each session
  beforeSession: () => {
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] },
    );

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });

    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    closeTauriDriver();
  },
};

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };
  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
});
