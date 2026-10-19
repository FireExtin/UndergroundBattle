// Preserve the reviewed pre-ABI84 card projection inside the current candidate.
// The original 147 records and ordering stay pinned; later admissions are explicit.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

export function catalogBeforeAbi84(current) {
  assert.equal(current.cards.length,152,'Current catalog is checked separately from historical projections');
  assert.deepEqual(current.cards.slice(147).map(c=>c.id),['JC057','JC044','JC066','JC041','XQ23']);
  assert.equal(new Set(current.cards.map(c=>c.id)).size,152);
  const cards=current.cards.slice(0,147);
  assert.equal(createHash('sha256').update(JSON.stringify(cards)).digest('hex'),
    '429f9097a748309da8bf3d29c2f70fac03c727c47e8fd36c1f98086a1da7e4f2',
    'The entire reviewed 147-card array must remain byte-value equivalent');
  return {...current,cards};
}
