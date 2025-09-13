// @ts-nocheck
import os from 'os';
import path from 'path';
import fs from 'fs';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

// keep track of the `tauri-driver` child process
let tauriDriver;
let exit = false;

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./specs/**/*.js'],
  maxInstances: 1,
  logLevel: 'debug',
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        // Point to NAMDRunner release binary (built by onPrepare)
        application: path.resolve(__dirname, '../../src-tauri/target/release/namdrunner'),
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // ensure the rust project is built since we expect this binary to exist for the webdriver sessions
  onPrepare: () => {
    const binaryPath = path.resolve(__dirname, '../../src-tauri/target/release/namdrunner');
    console.log('🔍 Checking for existing binary at:', binaryPath);
    
    if (fs.existsSync(binaryPath)) {
      const stats = fs.statSync(binaryPath);
      const ageMinutes = (Date.now() - stats.mtime.getTime()) / (1000 * 60);
      console.log(`✅ Existing binary is ${ageMinutes.toFixed(1)} minutes old`);
    }
    
    console.log('🏗️ Building new NAMDRunner binary for E2E testing...');
    spawnSync('npm', ['run', 'tauri', 'build'], {
      cwd: path.resolve(__dirname, '../..'),
      stdio: 'inherit',
      shell: true,
    });
    console.log('✅ Build complete');
  },

  // ensure we are running `tauri-driver` before the session starts so that we can proxy the webdriver requests
  beforeSession: async () => {
    console.log('🚀 Starting tauri-driver...');
    
    // Check if the binary exists before launching
    const binaryPath = path.resolve(__dirname, '../../src-tauri/target/release/namdrunner');
    console.log('📍 Expected binary path:', binaryPath);
    
    if (!fs.existsSync(binaryPath)) {
      console.error('❌ Binary not found at:', binaryPath);
      console.error('💡 Run "npm run tauri build" first');
      process.exit(1);
    }
    console.log('✅ Binary exists and is ready');
    
    // Launch tauri-driver with verbose output
    const driverPath = path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver');
    console.log('🔧 Launching tauri-driver from:', driverPath);
    
    tauriDriver = spawn(driverPath, [
      '--native-driver', '/usr/bin/WebKitWebDriver'
    ], {
      stdio: [null, process.stdout, process.stderr]
    });
    
    console.log('🔄 tauri-driver PID:', tauriDriver.pid);
    
    // Enhanced error handling with more details
    tauriDriver.on('spawn', () => {
      console.log('✅ tauri-driver spawned successfully');
    });
    
    tauriDriver.on('error', (error) => {
      console.error('💥 tauri-driver spawn error:', error.message);
      console.error('💡 Make sure tauri-driver is installed: cargo install tauri-driver --locked');
      process.exit(1);
    });
    
    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('🔴 tauri-driver exited unexpectedly with code:', code);
        process.exit(1);
      }
    });
    
    // Give tauri-driver time to start up and bind to port 4444
    console.log('⏳ Waiting for tauri-driver to start...');
    await new Promise(resolve => setTimeout(resolve, 2000));
    console.log('✅ tauri-driver startup complete');
  },

  // clean up the `tauri-driver` process we spawned at the start of the session
  // note that afterSession might not run if the session fails to start, so we also run the cleanup on shutdown
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

// ensure tauri-driver is closed when our test process exits
onShutdown(() => {
  closeTauriDriver();
});