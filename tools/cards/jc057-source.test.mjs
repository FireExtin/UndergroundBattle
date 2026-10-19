import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const read=p=>readFileSync(new URL('../../'+p,import.meta.url));
const json=p=>JSON.parse(read(p));
const hash=b=>createHash('sha256').update(b).digest('hex');
const current=json('rust-game/data/cards.json'),data={...current,cards:current.cards.filter(c=>!['JC044','JC066','JC041','XQ23'].includes(c.id))},spec=json('docs/factions/card-specifications.json').cards.JC057;
test('JC057 is the complete pinned original, without upgrading archival acceptance',()=>{
  const c=data.cards.find(c=>c.id==='JC057'),s=spec.sourceVerification;
  assert.equal(s.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(s.blockingFields,[]);
  assert.equal(s.imageSha256,'acb8e6d6c421a37862b81e5f194af50a974888e74e252b0a594b7123a6d01678');
  assert.equal(hash(read(s.imagePath)),s.imageSha256);assert.deepEqual(read(s.imagePath),read('web/public/cards/JC057.jpg'));
  assert.deepEqual(json('web/public/card-scans.json').JC057,{url:'/cards/JC057.jpg',source:s.imagePath,sha256:s.imageSha256});
  assert.match(read('web/src/game/cardScans.ts').toString(),/['"]?JC057['"]?:\s*['"]\/cards\/JC057\.jpg['"]/);
  assert.equal(c.name,spec.fields.name);assert.equal(c.cost,spec.fields.printedCost);
  assert.equal(c.kind,'character');assert.equal(c.color,spec.fields.colorKey);assert.equal(c.magic,'');
  assert.deepEqual(c.subtypes,spec.fields.subtypes);assert.deepEqual(c.loyalty,['灰色']);
  assert.deepEqual(c.icons,{permanent:spec.fields.permanentIcons,temporary:spec.fields.temporaryIcons});
  assert.equal(c.defense,spec.fields.defense);assert.equal(c.unique,false);assert.equal(c.society,'');
  assert.equal(c.text,spec.fields.printedRuleTextLines.join('\n'));assert.deepEqual(c.keywords,[]);
  assert.equal(spec.implementationDesignStatus,'readyForBoundedDesign');assert.deepEqual(spec.openQuestionIds,[]);
  assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('JC057 adds exactly one gray card and leaves the original 147 cards, presets and world untouched',()=>{
  assert.equal(data.cards.length,148);assert.equal(data.cards.at(-1).id,'JC057');
  for(const [key,value,sha]of [
    ['cards',data.cards.slice(0,-1),'429f9097a748309da8bf3d29c2f70fac03c727c47e8fd36c1f98086a1da7e4f2'],
    ['decks',data.decks,'3b51ce8f6ae0c78445e9660c4dc0e0c0bdc07bc35c7c8b30ae5ae107097e1987'],
    ['world',data.world,'c7026487b412b58385ddd94e073b224e65e730ee636a2834586901c25b1252a5'],
  ])assert.equal(hash(JSON.stringify(value)),sha,key+' fixed baseline 14758d9');
  assert.equal(data.engineVersion,'rust-v0.2.88-xq23-asset-return-candidate');
  assert.equal(data.cardPoolVersion,'limited-v2.80-xq23-asset-return-candidate');
});
