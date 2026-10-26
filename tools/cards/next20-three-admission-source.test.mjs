import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const pool=JSON.parse(readFileSync(new URL('../../rust-game/data/cards.json',import.meta.url),'utf8'));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
test('pool104 preserves three admitted cards, all legacy277 records and one closed BQ001',()=>{
 assert.equal(pool.cardPoolVersion,'limited-v2.104-bq001-anchor-source-wip');assert.equal(pool.cards.length,278);assert.equal(pool.cards.filter(c=>c.supported!==false).length,236);
 assert.deepEqual(pool.cards.slice(277).map(c=>c.id),['BQ001']);
 const previous=structuredClone(pool.cards.slice(0,277));
 for(const id of ['BQ034','CQ25','XG25']){const c=previous.find(c=>c.id===id);assert.equal(c.supported,true);c.supported=false;}
 for(const id of ['BQ001','BQ009','BQ033'])assert.equal(pool.cards.find(c=>c.id===id).supported,false);
 assert.equal(createHash('sha256').update(JSON.stringify(canonical(previous))).digest('hex'),'0876590922afda7f522d4594a5e0d689266b0e325acdf191ef1ce915307e6c91');
 const c=pool.cards.find(c=>c.id==='BQ034');assert.equal(c.unique,true);assert.deepEqual(c.keywords,[]);
});
