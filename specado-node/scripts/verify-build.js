#!/usr/bin/env node
/**
 * Specado Node.js Build Verification Script
 * Tests npm package building, installation, and basic functionality
 */

const fs = require('fs');
const path = require('path');
const { spawn, execSync } = require('child_process');
const os = require('os');

// Get the directory of this script
const SCRIPT_DIR = __dirname;
const PROJECT_DIR = path.dirname(SCRIPT_DIR);
const TEMP_DIR = path.join(os.tmpdir(), `specado-verify-${Date.now()}`);

// Colors for output
const colors = {
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  reset: '\x1b[0m'
};

function log(level, message) {
  const color = colors[level] || colors.reset;
  const prefix = {
    info: '[INFO]',
    warn: '[WARN]',
    error: '[ERROR]',
    success: '[SUCCESS]'
  }[level] || '[LOG]';
  
  console.log(`${color}${prefix}${colors.reset} ${message}`);
}

function cleanup() {
  if (fs.existsSync(TEMP_DIR)) {
    log('info', 'Cleaning up temporary environment...');
    fs.rmSync(TEMP_DIR, { recursive: true, force: true });
  }
}

// Cleanup on exit
process.on('exit', cleanup);
process.on('SIGINT', cleanup);

// Check dependencies
function checkDependencies() {
  log('info', 'Checking dependencies...');
  
  try {
    const nodeVersion = process.version;
    log('info', `Node.js version: ${nodeVersion}`);
    
    // Check if major version is >= 16
    const majorVersion = parseInt(nodeVersion.slice(1).split('.')[0]);
    if (majorVersion < 16) {
      log('error', 'Node.js 16.0.0 or higher is required');
      process.exit(1);
    }
    
    // Check npm
    try {
      const npmVersion = execSync('npm --version', { encoding: 'utf8' }).trim();
      log('info', `npm version: ${npmVersion}`);
    } catch (e) {
      log('error', 'npm is required but not available');
      process.exit(1);
    }
    
    // Check if we can build with napi
    try {
      execSync('npx napi --help', { encoding: 'utf8', cwd: PROJECT_DIR, stdio: 'pipe' });
      log('info', 'NAPI-RS CLI is available');
    } catch (e) {
      log('error', '@napi-rs/cli is required but not available');
      log('info', 'Install with: npm install @napi-rs/cli');
      process.exit(1);
    }
    
  } catch (error) {
    log('error', `Dependency check failed: ${error.message}`);
    process.exit(1);
  }
}

// Build native module
function buildNativeModule() {
  log('info', 'Building native module...');
  
  try {
    // Clean previous builds
    const artifactsDir = path.join(PROJECT_DIR, 'artifacts');
    const distDir = path.join(PROJECT_DIR, 'dist');
    
    if (fs.existsSync(artifactsDir)) {
      fs.rmSync(artifactsDir, { recursive: true, force: true });
    }
    if (fs.existsSync(distDir)) {
      fs.rmSync(distDir, { recursive: true, force: true });
    }
    
    // Build the native module
    log('info', 'Running: npm run build');
    execSync('npm run build', { 
      cwd: PROJECT_DIR, 
      stdio: 'pipe',
      encoding: 'utf8'
    });
    
    // Check if .node file was created
    const nodeFiles = fs.readdirSync(PROJECT_DIR).filter(f => f.endsWith('.node'));
    if (nodeFiles.length === 0) {
      log('error', 'No .node file found after build');
      process.exit(1);
    }
    
    log('success', `Built native module: ${nodeFiles[0]}`);
    
  } catch (error) {
    log('error', `Build failed: ${error.message}`);
    process.exit(1);
  }
}

// Create npm package
function createNpmPackage() {
  log('info', 'Creating npm package...');
  
  try {
    // Run npm pack to create tarball
    const packOutput = execSync('npm pack', { 
      cwd: PROJECT_DIR, 
      encoding: 'utf8'
    }).trim();
    
    const tarballName = packOutput.split('\n').pop();
    const tarballPath = path.join(PROJECT_DIR, tarballName);
    
    if (!fs.existsSync(tarballPath)) {
      log('error', `Tarball not found: ${tarballPath}`);
      process.exit(1);
    }
    
    log('success', `Created package: ${tarballName}`);
    return { tarballPath, tarballName };
    
  } catch (error) {
    log('error', `Package creation failed: ${error.message}`);
    process.exit(1);
  }
}

// Test installation in clean environment
function testInstallation(tarballPath) {
  log('info', 'Testing installation in clean environment...');
  
  try {
    // Create temporary directory
    fs.mkdirSync(TEMP_DIR, { recursive: true });
    
    // Create a minimal package.json for testing
    const testPackageJson = {
      name: 'specado-test',
      version: '1.0.0',
      type: 'module'
    };
    
    fs.writeFileSync(
      path.join(TEMP_DIR, 'package.json'), 
      JSON.stringify(testPackageJson, null, 2)
    );
    
    // Install the package
    log('info', `Installing package from ${tarballPath}...`);
    execSync(`npm install "${tarballPath}"`, { 
      cwd: TEMP_DIR, 
      stdio: 'inherit',
      encoding: 'utf8'
    });
    
    log('success', 'Installation completed');
    
    // Verify the package is installed
    const nodeModulesPath = path.join(TEMP_DIR, 'node_modules', '@specado', 'core');
    if (!fs.existsSync(nodeModulesPath)) {
      log('error', 'Package not found in node_modules');
      process.exit(1);
    }
    
    log('success', 'Package correctly installed in node_modules');
    
  } catch (error) {
    log('error', `Installation test failed: ${error.message}`);
    process.exit(1);
  }
}

// Test basic functionality
function testFunctionality() {
  log('info', 'Testing basic functionality...');
  
  try {
    // Create test script
    const testScript = `
import { Client, createMessage, version } from 'specado';

console.log('🧪 Testing specado functionality...');

// Test version function
try {
  const ver = version();
  console.log('✅ Version function works:', ver);
  if (typeof ver !== 'string' || ver.length === 0) {
    throw new Error('Version should return a non-empty string');
  }
} catch (error) {
  console.error('❌ Version function failed:', error.message);
  process.exit(1);
}

// Test message creation
try {
  const msg = createMessage('user', 'test message');
  console.log('✅ Message creation works:', msg);
  if (msg.role !== 'user' || msg.content !== 'test message') {
    throw new Error('Message not created correctly');
  }
} catch (error) {
  console.error('❌ Message creation failed:', error.message);
  process.exit(1);
}

// Test Client creation
try {
  const client = new Client();
  console.log('✅ Client creation works');
  
  // Test configuration access
  const keys = client.configKeys();
  console.log('✅ Config keys:', keys);
  
  const primary = client.getConfig('primary_provider');
  const fallback = client.getConfig('fallback_provider');
  console.log('✅ Configuration access works:', { primary, fallback });
  
  // Test API structure
  const chat = client.getChat();
  const completions = chat.getCompletions();
  console.log('✅ API structure is correct');
  
} catch (error) {
  console.error('❌ Client functionality failed:', error.message);
  process.exit(1);
}

console.log('🎉 All functionality tests passed!');
`;
    
    const testPath = path.join(TEMP_DIR, 'test.mjs');
    fs.writeFileSync(testPath, testScript);
    
    // Run the test
    execSync(`node ${testPath}`, { 
      cwd: TEMP_DIR, 
      stdio: 'inherit',
      encoding: 'utf8'
    });
    
    log('success', 'Functionality tests passed');
    
  } catch (error) {
    log('error', `Functionality test failed: ${error.message}`);
    process.exit(1);
  }
}

// Test package metadata
function testPackageMetadata(tarballPath) {
  log('info', 'Testing package metadata...');
  
  try {
    // Extract and examine package.json from tarball
    const extractDir = path.join(TEMP_DIR, 'extract');
    fs.mkdirSync(extractDir, { recursive: true });
    
    execSync(`tar -xzf "${tarballPath}" -C "${extractDir}"`, { encoding: 'utf8' });
    
    const packageDir = fs.readdirSync(extractDir)[0];
    const packageJsonPath = path.join(extractDir, packageDir, 'package.json');
    
    if (!fs.existsSync(packageJsonPath)) {
      log('error', 'package.json not found in tarball');
      process.exit(1);
    }
    
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    
    // Check required fields
    const requiredFields = ['name', 'version', 'description', 'main', 'types', 'license'];
    for (const field of requiredFields) {
      if (!packageJson[field]) {
        log('error', `Missing required field: ${field}`);
        process.exit(1);
      }
      log('info', `✅ ${field}: ${packageJson[field]}`);
    }
    
    // Check files
    const expectedFiles = ['index.js', 'index.d.ts', 'README.md', 'CHANGELOG.md'];
    for (const file of expectedFiles) {
      const filePath = path.join(extractDir, packageDir, file);
      if (!fs.existsSync(filePath)) {
        log('warn', `Expected file missing: ${file}`);
      } else {
        log('info', `✅ File included: ${file}`);
      }
    }
    
    log('success', 'Package metadata validation passed');
    
  } catch (error) {
    log('error', `Metadata test failed: ${error.message}`);
    process.exit(1);
  }
}

// Main execution
async function main() {
  console.log('🔧 Specado Node.js Build Verification');
  console.log('=====================================');
  
  try {
    checkDependencies();
    buildNativeModule();
    const { tarballPath, tarballName } = createNpmPackage();
    testPackageMetadata(tarballPath);
    testInstallation(tarballPath);
    testFunctionality();
    
    log('success', '🎉 Build verification completed successfully!');
    log('info', 'Your npm package is ready for distribution:');
    console.log(path.join(PROJECT_DIR, tarballName));
    
  } catch (error) {
    log('error', `Verification failed: ${error.message}`);
    process.exit(1);
  }
}

// Run main function
main().catch(error => {
  log('error', `Unexpected error: ${error.message}`);
  process.exit(1);
});