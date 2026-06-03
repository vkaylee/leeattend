import { spawnSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Helper to read and parse .env files
function parseEnv(filePath) {
  try {
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, 'utf-8');
      const env = {};
      content.split(/\r?\n/).forEach(line => {
        // Ignore comments and empty lines
        if (line.trim().startsWith('#') || !line.trim()) return;
        const match = line.match(/^\s*([\w.-]+)\s*=\s*(.*)\s*$/);
        if (match) {
          const key = match[1];
          let value = match[2].trim();
          // Remove wrapping quotes if any
          if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
            value = value.slice(1, -1);
          }
          env[key] = value;
        }
      });
      return env;
    }
  } catch (e) {
    // Ignore error
  }
  return {};
}

// Read .env from webdashboard folder and root folder
const localEnv = parseEnv(path.join(__dirname, '../.env'));
const rootEnv = parseEnv(path.join(__dirname, '../../.env'));

const env = { ...process.env, ...rootEnv, ...localEnv };

const args = process.argv.slice(2);

// Check if ESLINT_CONCURRENCY is defined in environment
const concurrency = env.ESLINT_CONCURRENCY;

if (concurrency) {
  // Check if --concurrency is already passed in args to avoid duplication
  if (!args.some(arg => arg.startsWith('--concurrency'))) {
    args.push(`--concurrency=${concurrency}`);
  }
}

// Run eslint
console.log(`[ESLint Wrapper] Running: bunx eslint ${args.join(' ')}`);
const result = spawnSync('bunx', ['eslint', ...args], { stdio: 'inherit', shell: true });
process.exit(result.status ?? 0);

