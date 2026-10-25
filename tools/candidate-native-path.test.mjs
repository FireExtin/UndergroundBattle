import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {candidateNativePath} from './candidate-native-path.mjs';

test('candidate evidence is opt-in, isolated and never repaired from a prior fixture', () => {
  const prior = process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES;
  delete process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES;
  const base = mkdtempSync(path.join(tmpdir(), 'candidate-native-path-'));
  try {
    const source = path.join(base, 'source'), candidate = path.join(base, 'fresh');
    const relative = 'rust-game-wasm/tests/current112/example.json.gz';
    mkdirSync(path.join(source, path.dirname(relative)), {recursive:true});
    const original = path.join(source, relative);
    writeFileSync(original, 'prior-evidence');
    assert.equal(candidateNativePath(relative, source, undefined), original);
    const fresh = candidateNativePath(relative, source, candidate);
    assert.throws(() => readFileSync(fresh), {code:'ENOENT'});
    assert.equal(readFileSync(original, 'utf8'), 'prior-evidence');
    mkdirSync(path.dirname(fresh), {recursive:true}); writeFileSync(fresh, 'fresh-evidence');
    assert.equal(readFileSync(candidateNativePath(relative, source, candidate), 'utf8'), 'fresh-evidence');
    for (const bad of ['', 'relative', source, path.dirname(source), path.join(source, 'fixtures')])
      assert.throws(() => candidateNativePath(relative, source, bad));
    for (const bad of ['../old.json.gz', '/old.json.gz', 'x/../old.json.gz', 'x//old.json.gz'])
      assert.throws(() => candidateNativePath(bad, source, candidate));
  } finally {
    rmSync(base, {recursive:true, force:true});
    if (prior === undefined) delete process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES;
    else process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES = prior;
  }
});
