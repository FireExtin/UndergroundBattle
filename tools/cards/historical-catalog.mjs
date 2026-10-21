// Preserve the reviewed pre-ABI84 card projection inside the current candidate.
// The original 147 records and ordering stay pinned; later admissions are explicit.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

export function catalogBeforeAbi84(current) {
  assert.equal(current.cards.length,153,'Current catalog is checked separately from historical projections');
  assert.deepEqual(current.cards.slice(147).map(c=>c.id),['JC057','JC044','JC066','JC041','XQ23','JC035']);
  assert.equal(new Set(current.cards.map(c=>c.id)).size,153);
  const cards=current.cards.slice(0,147);
  assert.equal(createHash('sha256').update(JSON.stringify(cards)).digest('hex'),
    '429f9097a748309da8bf3d29c2f70fac03c727c47e8fd36c1f98086a1da7e4f2',
    'The entire reviewed 147-card array must remain byte-value equivalent');
  return {...current,cards};
}

// Exact published v70 card prefix, with candidate identity checked separately.
export function catalogBeforeAbi89(current) {
  assert.equal(current.cards.length,153);
  assert.deepEqual(current.cards.slice(152).map(c=>c.id),['JC035']);
  const cards=current.cards.slice(0,152);
  assert.equal(createHash('sha256').update(JSON.stringify(cards)).digest('hex'),
    '69f2401c40f0488fe94a15a992dc0688924f101af83888992b4696cd49136f25',
    'The entire reviewed152-card prefix must remain byte-value equivalent');
  return {...current,cards};
}

// Semantic JSON hashing for fixed historical catalog pins, independent of object key order.
export function catalogDigest(value) {
  const canonical = v => Array.isArray(v) ? v.map(canonical)
    : v && typeof v === 'object' ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])])) : v;
  return createHash('sha256').update(JSON.stringify(canonical(value))).digest('hex');
}
