import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {catalog} from '../../rust-game-wasm/tests/current83-kernel.mjs';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../../',moduleUrl));
const read=p=>readFileSync(root+p),json=p=>JSON.parse(read(p));
const hash=b=>createHash('sha256').update(b).digest('hex');
const specs=json('docs/factions/card-specifications.json').cards;
test('MSJC04/MSJC05 use their complete pinned originals and two additional immutable scans',()=>{
  const scans=json('web/public/card-scans.json');
  for(const [id,name,color,subtype,sha256]of [
    ['MSJC04','鸣钟教派','红','秘密教派','a6ba66c7fb73acdbe4cab001e6330d0f339c46e12b6d3e714307196e9fd23cf3'],
    ['MSJC05','国家机构','灰','机构','d71e7c3df37a7c857cfb74c9988261ad722c5f3012ca49c75151fd407dab59f3'],
  ]) {
    const spec=specs[id],source=spec.sourceVerification,society=catalog.societies.find(c=>c.id===id);
    assert.equal(source.status,'completeGameplayFieldsForPinnedImage');assert.deepEqual(source.blockingFields,[]);
    // Keep the archival rules question intact. This candidate proposes the
    // same project interpretation already used by the admitted color societies;
    // independent review must explicitly approve its scope for MSJC04/MSJC05.
    assert.deepEqual(spec.openQuestionIds,['conditionalActivation']);
    assert.equal(hash(read(source.imagePath)),sha256);assert.deepEqual(read(source.imagePath),read(`web/public/cards/${id}.jpg`));
    assert.deepEqual(scans[id],{url:`/cards/${id}.jpg`,source:source.imagePath,sha256});
    assert.equal(society.name,name);assert.equal(society.color,color);assert.deepEqual(society.subtypes,[subtype]);
    assert.equal(society.unique,true);assert.equal(society.printedCost,null);assert.equal(society.startingHand,6);
    assert.deepEqual(society.deckConstraints,[{kind:'minimumColor',color,count:25}]);assert.deepEqual(society.unresolvedAbilities,{});
    assert.equal(society.abilities.length,2);
  }
  const result=validateAdmittedScans();assert.equal(result.checkedCards,147);assert.equal(result.manifestEntries,160);
});
test('the new batch changes no ordinary card, preset, world or other source metadata',()=>{
  const previous=JSON.parse(execFileSync('git',['show','398e220721f9e14c4f523530183c6288b7749096:rust-game/data/cards.json'],{cwd:root}));
  const current=json('rust-game/data/cards.json');
  const {engineVersion,cardPoolVersion,...body}=current;
  assert.deepEqual({...body,engineVersion:previous.engineVersion,cardPoolVersion:previous.cardPoolVersion},previous);
  assert.equal(engineVersion,'rust-v0.2.83-msjc04-msjc05-societies-candidate');
  assert.equal(cardPoolVersion,'limited-v2.75-msjc04-msjc05-societies-candidate');
  assert.deepEqual(catalog.cards.filter(c=>c.color==='红'&&c.unique).map(c=>c.id),['JC052']);
  assert.deepEqual(catalog.cards.filter(c=>c.color==='灰'&&c.unique),[]);
});
