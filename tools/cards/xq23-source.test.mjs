import {catalogBeforeAbi89} from './historical-catalog.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const read=p=>readFileSync(new URL('../../'+p,import.meta.url));
const json=p=>JSON.parse(read(p)),hash=b=>createHash('sha256').update(b).digest('hex');
const data=catalogBeforeAbi89(json('rust-game/data/cards.json')),spec=json('docs/factions/card-specifications.json').cards.XQ23;
test('XQ23 whole gray print and source artwork remain pinned without changing archival acceptance',()=>{
 const c=data.cards.at(-1),f=spec.fields,s=spec.sourceVerification;
 assert.equal(c.id,'XQ23');assert.equal(c.name,f.name);assert.equal(c.color,'灰');assert.equal(c.kind,'character');
 assert.equal(c.cost,1);assert.deepEqual(c.loyalty,['灰色']);assert.deepEqual(c.subtypes,['人类','文员']);assert.equal(c.magic,'');
 assert.equal(c.unique,false);assert.equal(c.defense,1);assert.deepEqual(c.icons,{permanent:f.permanentIcons,temporary:f.temporaryIcons});
 assert.deepEqual(c.keywords,['公开','撤回']);assert.equal(c.text,f.printedRuleTextLines.join('\n'));
 assert.equal(s.imageSha256,'7370d343be9e467716a3d7ada1d95abc012f5901ed15629e2f8d18065ea0769e');
 assert.equal(hash(read(s.imagePath)),s.imageSha256);assert.deepEqual(read(s.imagePath),read('web/public/cards/XQ23.jpg'));
 assert.deepEqual(json('web/public/card-scans.json').XQ23,{url:'/cards/XQ23.jpg',source:s.imagePath,sha256:s.imageSha256});
 assert.match(read('web/src/game/cardScans.ts').toString(),/"XQ23": "\/cards\/XQ23\.jpg"/);
 assert.equal(s.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(s.blockingFields,[]);assert.deepEqual(spec.openQuestionIds,[]);
 assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('XQ23 preserves the reviewed151 definitions, all decks and world; exact published87 remains a dependency',async()=>{
 assert.equal(data.cards.length,152);assert.equal(data.engineVersion,'rust-v0.2.90-audit-correctness-candidate');assert.equal(data.cardPoolVersion,'limited-v2.81-jc035-preparation-candidate');
 for(const[key,value,expected]of [
  ['cards',data.cards.filter(c=>c.id!=='XQ23'),'baa6c537d1309d2e0d88ec6f5053cd0debe816390ba19cdfb245ab6c04c56b8c'],
  ['decks',data.decks,'3b51ce8f6ae0c78445e9660c4dc0e0c0bdc07bc35c7c8b30ae5ae107097e1987'],
  ['world',data.world,'c7026487b412b58385ddd94e073b224e65e730ee636a2834586901c25b1252a5'],
 ])assert.equal(hash(JSON.stringify(value)),expected,key+' reviewed v69 baseline');
 const checked=validateAdmittedScans();assert.equal(checked.checkedCards,153);assert(checked.checkedIds.includes('XQ23'));
 const old=await import('../../rust-game-wasm/tests/current87-kernel.mjs');
 assert.equal(old.catalog.cards.length,151);assert.equal(old.catalog.cards.some(c=>c.id==='XQ23'),false);
 assert.equal(hash(old.moduleBytes),'f2c1c2f38bf7eec1103c32afac955500d29ab875f4c9039e1a73a787dc6568ed');
 assert.equal(old.catalog.engineVersion,'rust-v0.2.87-jc041-water-hidden-candidate');
});
