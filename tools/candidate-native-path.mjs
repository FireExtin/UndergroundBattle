// Explicit test-evidence location. This never falls back to a preserved fixture.
import assert from 'node:assert/strict';
import path from 'node:path';

export function candidateNativePath(relative, repository, candidate = process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES) {
  assert(typeof relative === 'string' && relative.length > 0 && !path.isAbsolute(relative));
  assert(!relative.split(/[\\/]/).some(part => part === '..' || part === '.' || part === ''));
  const source = path.resolve(repository);
  if (candidate === undefined) return path.join(source, relative);
  assert(typeof candidate === 'string' && candidate.trim() && path.isAbsolute(candidate), 'Candidate Native evidence root must be an explicit absolute directory');
  const target = path.resolve(candidate);
  const outside = (parent, child) => path.relative(parent, child).startsWith('..' + path.sep);
  assert(outside(source, target) && outside(target, source), 'Candidate Native evidence must be separate from the source checkout');
  return path.join(target, relative);
}
