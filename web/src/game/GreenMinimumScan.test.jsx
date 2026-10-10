import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import { kernel } from './testKernel';
import {CardContent,visibleCard} from './CardTile';
import {ReadModal} from './ReadModal';
import {validateDeckDraft} from './deckLibrary';
const catalog=JSON.parse(kernel.catalog());
// Recorded native four-seat views for presentation/privacy checks. This file
// does not restore game states through WASM; current rules have separate tests.
const fixtures=JSON.parse(readFileSync(resolve('src/game/greenMinimumTest.fixture.json'),'utf8')).fixtures;
const green=['JC014','JC016','JZ08','BQ022','JC020','XQ07','LC30','JC018','JC015'];
function draft(n=27,total=50){let left=n;const cards=green.flatMap(cardId=>{const count=Math.min(3,left);left-=count;return count?[{cardId,count}]:[];});cards.push({cardId:'JC125',count:total-n});return{id:'green-reader',name:'绿色最小50',description:'',societyId:'MSJC02',cards,rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''};}

it('admits only the three reviewed green roles and exact MSJC02 fields and whole original scans',()=>{

 const lc=catalog.cards.find(c=>c.id==='LC30'),mind=catalog.cards.find(c=>c.id==='JC018'),exorcist=catalog.cards.find(c=>c.id==='JC015'),society=catalog.societies.find(c=>c.id==='MSJC02');
 expect(lc).toMatchObject({name:'J·罗伯茨，“海雕”',subtitle:'“猛禽”战术小组',kind:'character',color:'绿',unique:true,cost:4,loyalty:['绿色'],magic:'',subtypes:['人类','猎手'],defense:2,keywords:['护卫2'],permanentIcons:{investigation:0,combat:1,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0}});
 expect(mind).toMatchObject({name:'魔刃传人',kind:'character',color:'绿',unique:false,cost:4,loyalty:['绿色','绿色'],magic:'心灵',subtypes:['人类','猎手','超能力者'],defense:2,keywords:['威名'],permanentIcons:{investigation:0,combat:1,influence:0},temporaryIcons:{investigation:0,combat:1,influence:1}});
 expect(exorcist).toMatchObject({name:'驱魔人',kind:'character',color:'绿',unique:false,cost:3,loyalty:['绿色'],magic:'神圣',subtypes:['人类','僧侣'],defense:1,permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:1,combat:1,influence:0}});
 expect(exorcist.abilities.map(a=>a.key)).toEqual(['exorcise']);expect(lc.abilities).toEqual([]);expect(mind.abilities).toEqual([]);
 expect(society).toMatchObject({name:'猎魔人',subtitle:'猎杀异种战团',kind:'society',color:'绿',subtypes:['群体'],unique:true,startingHand:6,printedCost:null,deckConstraints:[{kind:'minimumColor',color:'绿',count:25}]});expect(society.abilities.map(a=>a.key)).toEqual(['drawWithInitiative','search-green-unique']);
 expect(catalog.societies.some(c=>c.id==='MSJC03')).toBe(true);expect(catalog.cards.some(c=>c.id==='XQ11')).toBe(false);
 const scans=JSON.parse(readFileSync(resolve('src/game/cardScansV034.fixture.json'),'utf8'));
 expect(Object.keys(scans)).toHaveLength(106);
 for(const [id,hash]of Object.entries({LC30:'31ce6bd49725e5db643adb13daaef3529b8fb8440691214a00dd9fe666c5041d',JC018:'a764f6ad543c1aa25e287e42a508ecce2368277e64377c2969b2aff079b63af8',JC015:'84ab4fb4fdf5e44f35b8cb8d9a40531d57bb0edb64d29a9c15f80d6e1cd37e1e',MSJC02:'915a723573028050980cb665d0de85f5e5fd5a6de1b54e3d7664e69882edc807'})){
  expect(scans[id].sha256).toBe(hash);expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(hash);
 }
});

it('matches the 24/25/27 printed-green boundary with ordinary copy and total limits',()=>{
 for(const n of [24,25,27])expect(validateDeckDraft(draft(n),catalog).valid).toBe(n>=25);
 expect(validateDeckDraft(draft(25,49),catalog).valid).toBe(false);
 const four=draft();four.cards[0].count=4;four.cards.at(-1).count--;expect(validateDeckDraft(four,catalog).valid).toBe(false);
});

it('checks recorded four-seat views and current-controller privacy for all three borrowed backs',()=>{
 for(const row of fixtures.filter(f=>f.kind!=='dynamic'))for(let seat=0;seat<4;seat++){
  const view=row.views[seat];
  for(const card of view.regions[0].characters){expect(!!card.cardId).toBe(seat===row.controller);expect(card.owner).toBe(`p${row.owner}`);expect(card.controller).toBe(`p${row.controller}`);}
  const stale=row.views[row.controller].regions[0].characters;
  for(const card of stale)expect(!!visibleCard(card,`p${seat}`).cardId).toBe(seat===row.controller);
 }
 const row=fixtures.find(f=>f.controller===0);
 for(let seat=0;seat<4;seat++){
  const card=row.views[seat].regions[0].characters[0];
  const r=render(<ReadModal card={card} definition={catalog.cards.find(c=>c.id==='LC30')} viewerId={`p${seat}`} onClose={vi.fn()}/>);
  expect(!!screen.queryByRole('button',{name:'原始牌面'})).toBe(seat===0);
  if(seat!==0)expect(screen.getByRole('dialog')).not.toHaveTextContent('J·罗伯茨');r.unmount();
 }
 const ownerView=row.views[row.owner];
 for(const card of ownerView.hand){const d=catalog.cards.find(c=>c.id===card.cardId);const r=render(<ReadModal card={card} definition={d} viewerId={`p${row.owner}`} onClose={vi.fn()}/>);fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${d.name}原始牌面`})).toHaveAttribute('src',`/cards/${d.id}.jpg`);r.unmount();}
 const society=ownerView.societyZones.find(z=>z.owner===`p${row.owner}`)?.card || ownerView.societyZones[row.owner].card;
 const r=render(<ReadModal card={society} definition={catalog.societies.find(c=>c.id==='MSJC02')} viewerId={`p${row.owner}`} onClose={vi.fn()}/>);expect(screen.getByRole('dialog')).toHaveTextContent('起手 6 张');fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'猎魔人原始牌面'})).toHaveAttribute('src','/cards/MSJC02.jpg');r.unmount();
});

it('shows native current green stats separately from unchanged printed black icons and defense',()=>{
 const row=fixtures.find(f=>f.kind==='dynamic'),view=row.views[0];
 for(const [id,combat,defense]of [['LC30',5,5],['JC018',3,2]]){
  const card=view.regions[0].characters.find(c=>c.cardId===id),definition=catalog.cards.find(c=>c.id===id);
  expect(card.icons.combat).toBe(combat);expect(card.defense).toBe(defense);expect(definition.permanentIcons.combat).toBe(1);expect(definition.defense).toBe(2);
  const r=render(<CardContent card={card} definition={definition} viewerId="p0"/>);expect(screen.getByLabelText(`当前有效图标：调查0，战斗${combat}，势力${id==='LC30'?1:0}`)).toBeInTheDocument();expect(screen.getByText(`当前防御 ${defense} · 印刷防御 2`)).toBeInTheDocument();r.unmount();
 }
});
