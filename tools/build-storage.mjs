// Keep reusable build caches on disk; never fall back to tmpfs when disk is full.
import { existsSync, mkdirSync, readFileSync, realpathSync, statfsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const GiB = 1024 ** 3;
const memoryFilesystems = new Set([0x01021994, 0x858458f6]); // tmpfs, ramfs

function storageInfo(destination) {
  let existing = path.resolve(destination);
  while (!existsSync(existing)) existing = path.dirname(existing);
  const filesystem = statfsSync(realpathSync(existing), { bigint: true });
  return {
    destination,
    filesystemType: Number(BigInt.asUintN(32, filesystem.type)),
    availableBytes: Number(filesystem.bavail * filesystem.bsize),
  };
}

export function buildEnvironment(repository, additionalDestinations = []) {
  const buildRoot = path.resolve(process.env.UNDERGROUNDBATTLE_BUILD_ROOT
    || (existsSync('/workspace') ? '/workspace/undergroundbattle-build' : path.join(repository, '.build-cache')));
  const target = path.resolve(repository, process.env.CARGO_TARGET_DIR || path.join(buildRoot, 'target'));
  const temporary = path.join(buildRoot, 'tmp');
  const errors = [];
  for (const destination of [target, temporary, ...additionalDestinations]) {
    // Check both the existing output and its parent: a tool may retain a link
    // or remove it and recreate the output on the parent filesystem.
    const locations = additionalDestinations.includes(destination)
      ? [destination, path.dirname(destination)] : [destination];
    for (const location of new Set(locations)) {
      const info = storageInfo(location);
      if (memoryFilesystems.has(info.filesystemType)) {
        errors.push(`${location}: tmpfs/ramfs is not a build storage destination.`);
      } else if (info.availableBytes < GiB) {
        errors.push(`${location}: ${info.availableBytes} bytes available; require at least ${GiB} bytes on disk.`);
      }
    }
  }
  // This is a launch guard, not an estimate of the compiler's peak requirement.
  if (existsSync('/sys/fs/cgroup/memory.max')) {
    const limit = readFileSync('/sys/fs/cgroup/memory.max', 'utf8').trim();
    const current = Number(readFileSync('/sys/fs/cgroup/memory.current', 'utf8').trim());
    if (limit !== 'max' && Number(limit) - current < GiB) {
      errors.push(`cgroup: ${Number(limit) - current} bytes headroom; require at least ${GiB} bytes before launching a build.`);
    }
  }
  if (errors.length) throw new Error(`Build preflight failed:\n${errors.join('\n')}\nNo build started; no storage fallback or cleanup was performed.`);
  mkdirSync(target, { recursive: true });
  mkdirSync(temporary, { recursive: true });
  return {
    ...process.env,
    CARGO_TARGET_DIR: target,
    TMPDIR: temporary,
    CARGO_BUILD_JOBS: '1',
    CARGO_INCREMENTAL: '0',
    CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_PROFILE_TEST_DEBUG: '0',
    CARGO_PROFILE_RELEASE_DEBUG: '0',
    CARGO_PROFILE_DEV_CODEGEN_UNITS: process.env.CARGO_PROFILE_DEV_CODEGEN_UNITS || '256',
    CARGO_PROFILE_TEST_CODEGEN_UNITS: process.env.CARGO_PROFILE_TEST_CODEGEN_UNITS || '256',
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS: process.env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS || '256',
    MALLOC_ARENA_MAX: '1',
    UNDERGROUNDBATTLE_BUILD_STORAGE_READY: '1',
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = fileURLToPath(new URL('../', import.meta.url));
  try {
    const argumentsRemaining = process.argv.slice(2);
    const outputs = [];
    while (argumentsRemaining[0] === '--output-dir') {
      argumentsRemaining.shift();
      const output = argumentsRemaining.shift();
      if (!output) throw new Error('--output-dir requires a destination.');
      outputs.push(path.resolve(repository, output));
    }
    const environment = buildEnvironment(repository, outputs);
    const [separator, command, ...args] = argumentsRemaining;
    if (!separator) {
      console.log(JSON.stringify({ target: environment.CARGO_TARGET_DIR, temporary: environment.TMPDIR, jobs: 1 }));
    } else {
      if (separator !== '--' || !command) throw new Error('Usage: node tools/build-storage.mjs [--output-dir path] [-- command args...]');
      const result = spawnSync(command, args, { cwd: repository, env: environment, stdio: 'inherit' });
      if (result.error) throw result.error;
      process.exitCode = result.status ?? 1;
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 78;
  }
}
