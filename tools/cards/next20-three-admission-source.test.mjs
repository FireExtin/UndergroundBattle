import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const pool=JSON.parse(readFileSync(new URL('../../rust-game/data/cards.json',import.meta.url),'utf8'));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
test('pool103 admits exactly BQ034 CQ25 XG25 and preserves all prior277 printed records',()=>{
 assert.equal(pool.cardPoolVersion,'limited-v2.103-next20-three-admitted');assert.equal(pool.cards.length,277);assert.equal(pool.cards.filter(c=>c.supported!==false).length,236);
 const previous=structuredClone(pool.cards);
 for(const id of ['BQ034','CQ25','XG25']){const c=previous.find(c=>c.id===id);assert.equal(c.supported,true);c.supported=false;}
 for(const id of ['BQ009','BQ033'])assert.equal(pool.cards.find(c=>c.id===id).supported,false);
 assert.equal(createHash('sha256').update(JSON.stringify(canonical(previous))).digest('hex'),'0876590922afda7f522d4594a5e0d689266b0e325acdf191ef1ce915307e6c91');
 const c=pool.cards.find(c=>c.id==='BQ034');assert.equal(c.unique,true);assert.deepEqual(c.keywords,[]);
});
