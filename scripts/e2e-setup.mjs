#!/usr/bin/env node
/**
 * E2E 测试环境设置脚本
 *
 * 功能:
 *   1. 检查 Rust 工具链
 *   2. 构建前端 (npm run build)
 *   3. 构建 Rust (cargo build)
 *   4. 检查 Playwright 浏览器
 *   5. 以 CDP 模式启动 Tauri app
 *
 * 使用: node scripts/e2e-setup.mjs [--start-only] [--build-only]
 */

import { spawn, execSync } from 'child_process';
import { existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { createInterface } from 'readline';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const TAURI_DIR = resolve(ROOT, 'src-tauri');
const CDP_PORT = process.env.WEBKIT_DEBUG_PORT || '9222';

const args = process.argv.slice(2);
const buildOnly = args.includes('--build-only');
const startOnly = args.includes('--start-only');

// ============================================================================
// 辅助
// ============================================================================

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    console.log(`  → ${cmd} ${args.join(' ')}`);
    const child = spawn(cmd, args, {
      stdio: 'inherit',
      shell: true,
      cwd: ROOT,
      ...opts,
    });
    child.on('close', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} exited with code ${code}`));
    });
    child.on('error', reject);
  });
}

async function waitForCdp(timeoutSec = 60) {
  const start = Date.now();
  while (Date.now() - start < timeoutSec * 1000) {
    try {
      const res = await fetch(`http://localhost:${CDP_PORT}/json`);
      if (res.ok) {
        const pages = await res.json();
        if (pages.some((p) => p.type === 'page')) {
          console.log(`  ✅ CDP 就绪 (${CDP_PORT})`);
          return true;
        }
      }
    } catch {
      // 尚未就绪
    }
    await new Promise((r) => setTimeout(r, 2000));
  }
  throw new Error(`CDP 端口 ${CDP_PORT} 在 ${timeoutSec}s 后未就绪`);
}

// ============================================================================
// 步骤
// ============================================================================

async function buildFrontend() {
  console.log('\n📦 构建前端...');
  await run('npx', ['vite', 'build']);
}

async function buildRust() {
  console.log('\n🦀 构建 Rust...');
  // 使用 debug 构建 (更快, CDP 需要)
  await run('cargo', ['build', '--manifest-path', `${TAURI_DIR}/Cargo.toml`]);
}

async function checkPlaywright() {
  console.log('\n🎭 检查 Playwright 浏览器...');
  try {
    execSync('npx playwright install --with-deps chromium', {
      stdio: 'inherit',
      cwd: ROOT,
    });
  } catch {
    console.log('  ⚠️  Playwright 安装跳过 (可能已安装)');
  }
}

async function startApp() {
  console.log(`\n🚀 启动 Tauri app (CDP: ${CDP_PORT})...`);

  const exePath = resolve(TAURI_DIR, 'target/debug/deep-student.exe');
  if (!existsSync(exePath)) {
    throw new Error(`二进制文件不存在: ${exePath}\n请先运行: node scripts/e2e-setup.mjs --build-only`);
  }

  const appProcess = spawn(exePath, ['--test-mode'], {
    env: {
      ...process.env,
      WEBKIT_DEBUG_PORT: CDP_PORT,
    },
    detached: true,
    stdio: 'ignore',
  });

  appProcess.unref();

  // 保存 PID
  const pidFile = resolve(ROOT, 'tauri-app.pid');
  const fs = await import('fs');
  fs.writeFileSync(pidFile, String(appProcess.pid));
  console.log(`  PID: ${appProcess.pid} (保存到 ${pidFile})`);

  await waitForCdp(60);
  console.log('  ✅ App 已启动并支持 CDP');
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  console.log('🧪 DeepStudent E2E 测试环境设置');
  console.log('='.repeat(50));

  try {
    if (!startOnly) {
      await buildFrontend();
      await buildRust();
    }

    if (!buildOnly) {
      await checkPlaywright();
      await startApp();

      console.log('\n' + '='.repeat(50));
      console.log('✅ 环境就绪 — 运行测试:');
      console.log(`   npx playwright test --config=tests/playwright.e2e.config.ts`);
      console.log(`   npx playwright test --ui --config=tests/playwright.e2e.config.ts`);
      console.log('\n停止 App:');
      console.log(`   taskkill /F /PID $(cat tauri-app.pid)  # Windows`);
      console.log(`   kill $(cat tauri-app.pid)              # Linux/macOS`);
    } else {
      console.log('\n✅ 构建完成');
    }
  } catch (err) {
    console.error('\n❌ 设置失败:', err.message);
    process.exit(1);
  }
}

main();
