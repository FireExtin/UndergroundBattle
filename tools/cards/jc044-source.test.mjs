import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const read=p=>readFileSync(new URL('../../'+p,import.meta.url));
const json=p=>JSON.parse(read(p));
const hash=b=>createHash('sha256').update(b).digest('hex');
const current=json('rust-game/data/cards.json'),data={...current,cards:current.cards.filter(c=>!['JC066','JC041','XQ23'].includes(c.id))},spec=json('docs/factions/card-specifications.json').cards.JC044;
test('JC044 uses every printed gameplay field from the pinned original without upgrading archival acceptance',()=>{
 const c=data.cards.at(-1),s=spec.sourceVerification;assert.equal(c.id,'JC044');assert.equal(s.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(s.blockingFields,[]);
 assert.equal(s.imageSha256,'e250b0edab5c9ba2dd3a3d3cd15d0708bb72420ca6a059bcb6b116321c411800');
 assert.equal(hash(read(s.imagePath)),s.imageSha256);assert.deepEqual(read(s.imagePath),read('web/public/cards/JC044.jpg'));
 assert.deepEqual(json('web/public/card-scans.json').JC044,{url:'/cards/JC044.jpg',source:s.imagePath,sha256:s.imageSha256});
 assert.match(read('web/src/game/cardScans.ts').toString(),/"JC044": "\/cards\/JC044\.jpg"/);
 assert.equal(c.name,spec.fields.name);assert.equal(c.cost,3);assert.deepEqual(c.loyalty,['红色']);assert.equal(c.kind,'character');assert.equal(c.color,'红');assert.equal(c.magic,'死亡');
 assert.deepEqual(c.subtypes,['人类','法师','邪教徒']);assert.equal(c.defense,1);assert.equal(c.unique,false);assert.deepEqual(c.keywords,['时间标志1']);
 assert.deepEqual(c.icons,{permanent:spec.fields.permanentIcons,temporary:spec.fields.temporaryIcons});assert.equal(c.text,spec.fields.printedRuleTextLines.join('\n'));
 assert.equal(spec.implementationDesignStatus,'readyForBoundedDesign');assert.deepEqual(spec.openQuestionIds,[]);assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('JC044 adds exactly one red card, protects all previous148 definitions/presets/world and all152 original scans',()=>{
 assert.equal(data.cards.length,149);assert.equal(data.cards.at(-1).id,'JC044');
 for(const[key,value,expected]of [
  ['cards',data.cards.slice(0,-1),'37673e04c7c77fa7df27372ba06a11b42355365a2f33f7f425296df8c8ee83f8'],
  ['decks',data.decks,'3b51ce8f6ae0c78445e9660c4dc0e0c0bdc07bc35c7c8b30ae5ae107097e1987'],
  ['world',data.world,'c7026487b412b58385ddd94e073b224e65e730ee636a2834586901c25b1252a5'],
 ])assert.equal(hash(JSON.stringify(value)),expected,key+' fixed reviewed84 baseline');
 assert.equal(data.engineVersion,'rust-v0.2.88-xq23-asset-return-candidate');assert.equal(data.cardPoolVersion,'limited-v2.80-xq23-asset-return-candidate');
 const result=validateAdmittedScans();assert.equal(result.checkedCards,152);assert(result.checkedIds.includes('JC044'));
});
