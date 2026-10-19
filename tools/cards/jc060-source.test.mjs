import {catalogBeforeAbi84} from './historical-catalog.mjs';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC060,pool=catalogBeforeAbi84(json('rust-game/data/cards.json'));
test('JC060 preserves the complete accepted 140-card baseline, decks and metadata',()=>{
 const {cards,engineVersion,cardPoolVersion,...metadata}=pool;
 assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
 assert.equal(digest(cards.filter(bq035 => !["BQ035", "JZ46", "WM062", "JC054"].includes(bq035.id)).filter(c=>!['JC060','JC095','JC046'].includes(c.id))),'fb7b098df40bf6db04df91ef6480e037af535acf37c44c25daa50bcd33345a75');
 assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
 assert.equal(engineVersion,'rust-v0.2.88-xq23-asset-return-candidate');assert.equal(cardPoolVersion,'limited-v2.80-xq23-asset-return-candidate');
});
test('JC060 complete printed fields and continuous text match pinned060/135 original',()=>{
 assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'fbe71295b4a88c02f5b08db8076ca72c4a737fd0a51005e44e03e59a20f4daed');
 assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
 const f=spec.fields,s=spec.sourceVerification;
 assert.deepEqual(pool.cards.find(c=>c.id==='JC060'),{id:'JC060',name:f.name,kind:'character',subtypes:['人类','学者'],cost:3,
 loyalty:['灰色'],loyaltyText:'灰色',color:'灰',magic:'心灵',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:1,
 text:f.printedRuleTextLines.join('\n'),keywords:[],source:{file:'docs/factions/card-specifications.json',record:'JC060',image:s.imagePath,sha256:s.imageSha256}});
});
test('JC060 public scan is the byte-identical original and existing scans remain valid',()=>{
 const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
 assert.deepEqual(read('web/public/cards/JC060.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,152);assert.equal(r.manifestEntries,165);
});
