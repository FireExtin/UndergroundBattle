import {catalogBeforeAbi89} from './historical-catalog.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const moduleUrl=import.meta.url;
const read=p=>readFileSync(new URL('../../'+p,moduleUrl));
const json=p=>JSON.parse(read(p));
const hash=b=>createHash('sha256').update(b).digest('hex');
const data=catalogBeforeAbi89(json('rust-game/data/cards.json')),spec=json('docs/factions/card-specifications.json').cards.JC041;
test('JC041 admits its complete pinned blue print with both abilities and exact original scan',()=>{
 const c=data.cards.find(c=>c.id==='JC041'),s=spec.sourceVerification,f=spec.fields;
 assert.equal(c.id,'JC041');assert.equal(s.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(s.blockingFields,[]);
 assert.equal(s.imageSha256,'6ec41c75ea9709b9043ab4c842f425087ac615b2e9e5cc65879c0e16970e3f89');
 assert.equal(hash(read(s.imagePath)),s.imageSha256);assert.deepEqual(read(s.imagePath),read('web/public/cards/JC041.jpg'));
 assert.deepEqual(json('web/public/card-scans.json').JC041,{url:'/cards/JC041.jpg',source:s.imagePath,sha256:s.imageSha256});
 assert.match(read('web/src/game/cardScans.ts').toString(),/"JC041": "\/cards\/JC041\.jpg"/);
 assert.equal(c.name,f.name);assert.equal(c.kind,'character');assert.equal(c.color,'蓝');assert.equal(c.cost,4);
 assert.deepEqual(c.loyalty,[]);assert.equal(c.magic,'死亡');assert.equal(c.defense,1);assert.equal(c.unique,false);
 assert.deepEqual(c.subtypes,['不死生物','尸鬼']);assert.deepEqual(c.keywords,[]);
 assert.deepEqual(c.icons,{permanent:f.permanentIcons,temporary:f.temporaryIcons});assert.equal(c.text,f.printedRuleTextLines.join('\n'));
 assert.deepEqual(spec.openQuestionIds,[]);assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('JC041 adds one blue card while preserving all150 prior definitions, decks, world and archival ownership',()=>{
 assert.equal(data.cards.length,152);assert.equal(data.cards.at(-1).id,'XQ23');
 for(const[key,value,expected]of [
  ['cards',data.cards.filter(c=>!['JC041','XQ23'].includes(c.id)),'0914aa5f5ef9b55b5c24dafbe4c38a654a03376a7c989a7cd36a945a96653c2c'],
  ['decks',data.decks,'3b51ce8f6ae0c78445e9660c4dc0e0c0bdc07bc35c7c8b30ae5ae107097e1987'],
  ['world',data.world,'c7026487b412b58385ddd94e073b224e65e730ee636a2834586901c25b1252a5'],
 ])assert.equal(hash(JSON.stringify(value)),expected,key+' reviewed v68 baseline');
 assert.equal(data.engineVersion,'rust-v0.2.89-jc035-preparation-candidate');assert.equal(data.cardPoolVersion,'limited-v2.81-jc035-preparation-candidate');
 const result=validateAdmittedScans();assert.equal(result.checkedCards,153);assert(result.checkedIds.includes('JC041'));
});
