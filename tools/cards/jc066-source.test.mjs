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
const data=catalogBeforeAbi89(json('rust-game/data/cards.json')),spec=json('docs/factions/card-specifications.json').cards.JC066;
test('JC066 includes the complete pinned printed card and retains archival acceptance boundaries',()=>{
 const c=data.cards.find(c=>c.id==='JC066'),s=spec.sourceVerification,f=spec.fields;
 assert.equal(c.id,'JC066');assert.equal(s.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(s.blockingFields,[]);
 assert.equal(s.imageSha256,'c4cc8ccb0add2ad04e5a4a328949f24a982011c48d19e2c69073bd9cb05ea4a2');
 assert.equal(hash(read(s.imagePath)),s.imageSha256);assert.deepEqual(read(s.imagePath),read('web/public/cards/JC066.jpg'));
 assert.deepEqual(json('web/public/card-scans.json').JC066,{url:'/cards/JC066.jpg',source:s.imagePath,sha256:s.imageSha256});
 assert.match(read('web/src/game/cardScans.ts').toString(),/"JC066": "\/cards\/JC066\.jpg"/);
 assert.equal(c.name,f.name);assert.equal(c.subtitle,'样本B7021');assert.equal(c.cost,5);assert.deepEqual(c.loyalty,['灰色','灰色','灰色']);
 assert.equal(c.kind,'character');assert.equal(c.color,'灰');assert.equal(c.magic,'心灵');assert.deepEqual(c.subtypes,['人类','超能力者']);assert.equal(c.defense,2);assert.equal(c.unique,true);assert.deepEqual(c.keywords,[]);
 assert.deepEqual(c.icons,{permanent:f.permanentIcons,temporary:f.temporaryIcons});assert.equal(c.text,f.printedRuleTextLines.join('\n'));
 assert.deepEqual(spec.openQuestionIds,[]);assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('JC066 prior149 definitions remain protected after the separate JC041 blue addition',()=>{
 assert.equal(data.cards.length,152);assert.equal(data.cards.at(-1).id,'XQ23');
 const prior=data.cards.filter(c=>!['JC041','XQ23'].includes(c.id));assert.equal(prior.length,150);assert.equal(prior.at(-1).id,'JC066');
 for(const[key,value,expected]of [
  ['cards',prior.slice(0,-1),'97e288a031ac6fba1504db69bd903143bfde56c0692028748c51e54e08c0914c'],
  ['decks',data.decks,'3b51ce8f6ae0c78445e9660c4dc0e0c0bdc07bc35c7c8b30ae5ae107097e1987'],
  ['world',data.world,'c7026487b412b58385ddd94e073b224e65e730ee636a2834586901c25b1252a5'],
 ])assert.equal(hash(JSON.stringify(value)),expected,key+' reviewed v67 baseline');
 assert.equal(data.engineVersion,'rust-v0.2.89-jc035-preparation-candidate');assert.equal(data.cardPoolVersion,'limited-v2.81-jc035-preparation-candidate');
 const result=validateAdmittedScans();assert.equal(result.checkedCards,153);assert(result.checkedIds.includes('JC066'));
});
