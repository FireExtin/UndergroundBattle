import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {cleanup,render,screen} from '@testing-library/react';
import {afterEach,expect,it} from 'vitest';
import {DeckCardFace} from './DeckCardFace';
import {cardScanUrl} from './cardScans';
import {catalog as currentCatalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';

const read=path=>readFileSync(resolve('..',path));
const registry=JSON.parse(read('rust-game/data/cards.json'));
const primary=JSON.parse(read('docs/factions/card-specifications.json'));
const batch=JSON.parse(read('docs/factions/import20-rulings-2026-10-24.json')).batch20;
const newlyAdmitted=['BQ034','CQ25','XG25'];
const closedCandidates=["WM026","ZHWM026","BQ044","WM031","WM036","WM027","XQ13","JC053","JC028","JC097","JZ53","CQ27","BQ088","TK001","XG19","TK003","CQ11","TK008","BQ062","BQ060","BQ067","BQ065","BQ049", "BQ061", "BQ045", "BQ052", "XQ35", "BQ084", "CQ10", "CQ09", "BQ086", "BQ046", "BQ055", "BQ048", "BQ056", "BQ050", "BQ051", "TEST034", "BQ063"];
const scans=JSON.parse(read('web/public/card-scans.json'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'
  ?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
afterEach(cleanup);

it('preserves all241 prior source-index records and all239 prior runtime URLs',()=>{
  const prior=Object.fromEntries(Object.entries(scans).filter(([id])=>!batch.includes(id)&&!newlyAdmitted.includes(id)&&(!closedCandidates.includes(id)||id==='JZ53')&&id!=="TK010"&&!["MSBQ01","MSWM01","MSJZ04","MSBQ02","MSJC12","MSWM02","MSCQ02","MSBQ03","MSJC16"].includes(id)));
  expect(Object.keys(prior)).toHaveLength(241);
  expect(sha(JSON.stringify(canonical(prior)))).toBe('8391a760ca9b2d25d1767b53ebb2f092eb3f106f2d326b924a6eaa43e1de79b8');
  const runtime=Object.fromEntries(Object.keys(prior).filter(id=>!['BQ041','JZ38'].includes(id)).map(id=>[id,cardScanUrl(id)]));
  expect(Object.keys(runtime)).toHaveLength(239);
  expect(sha(JSON.stringify(canonical(runtime)))).toBe('b72fae2c401492c362b759fe3fbfdc36197147db936168edd048b90272b64d42');
});

it.each(batch)('%s displays its pinned original pixels through the real collection-card component',id=>{
  const card=registry.cards.find(card=>card.id===id),source=primary.cards[id].sourceVerification.imagePath;
  const expected=card.source.imageSha256;
  expect(sha(read(source))).toBe(expected);
  expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(expected);
  expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source,sha256:expected});
  expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
  render(<DeckCardFace card={card} large/>);
  expect(screen.getByRole('img',{name:`${card.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
  expect(screen.queryByText('暂无原始牌面')).not.toBeInTheDocument();
});

it.each(newlyAdmitted)('%s delivers the tracked original scan through both indexes and the card component',id=>{
  const card=registry.cards.find(card=>card.id===id);
  const {imagePath:source,imageSha256:expected}=primary.cards[id].sourceVerification;
  expect(typeof expected).toBe('string');
  expect(sha(read(source))).toBe(expected);
  expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(expected);
  expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source,sha256:expected});
  expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
  expect(card.supported).toBe(true);
  render(<DeckCardFace card={card} large/>);
  expect(screen.getByRole('img',{name:`${card.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
  expect(screen.queryByText('暂无原始牌面')).not.toBeInTheDocument();
});

it('keeps the current kernel and reviewed admission set while delivering artwork',()=>{
  expect(registry.engineVersion).toBe('rust-v0.2.113-bq001-anchor-source-wip');
  expect(currentCatalog.engineVersion).toBe('rust-v0.2.113-bq001-anchor-source-wip');
  expect(currentCatalog.cards.map(card=>[card.id,card.supported])).toEqual(registry.cards.map(card=>[card.id,card.supported!==false]));
  expect(registry.cards).toHaveLength(316);
  for(const id of ['BQ033','BQ009','BQ001',...closedCandidates])expect(registry.cards.find(card=>card.id===id).supported,id).toBe(false);
  for(const id of ['BQ034','CQ25','XG25'])expect(registry.cards.find(card=>card.id===id).supported,id).toBe(true);
  expect(registry.cards.filter(card=>card.supported!==false)).toHaveLength(236);
  expect(registry.cards.slice(213).filter(card=>card.supported).map(card=>card.id).sort()).toEqual([...batch,'BQ034','CQ25','XG25'].sort());
  expect(cardScanUrl('unknown-private-card')).toBeUndefined();
});

it.each(["MSBQ01","MSWM01","MSJZ04","MSJC12","MSWM02","MSCQ02","MSBQ03","MSJC16"])("%s preserves pinned original and remains a closed society",id=>{const source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);const card=currentCatalog.societies.find(c=>c.id===id);expect(card.supported).toBe(false);render(<DeckCardFace card={card} large/>);expect(screen.getByRole("img",{name:`${card.name}原始牌面`})).toHaveAttribute("src",`/cards/${id}.jpg`);});

it('TK010 preserves its original PNG and stays outside admission',()=>{const id='TK010',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/TK010.png'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/TK010.png',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/TK010.png');expect(card.supported).toBe(false);expect(card.subtypes).toEqual(['突变体','器官']);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:'触手原始牌面'})).toHaveAttribute('src','/cards/TK010.png');});

it('BQ052 original book artwork, loyalty and closed status stay aligned',()=>{const id='BQ052',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/BQ052.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ052.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ052.jpg');expect(card.supported).toBe(false);expect(card.loyalty).toEqual(['红色']);expect(card.unique).toBe(true);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ052.jpg');});

it('BQ045 keeps the original core cube separate from its single gray loyalty',()=>{const id='BQ045',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read('web/public/cards/BQ045.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ045.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ045.jpg');expect(card.supported).toBe(false);expect(card.loyalty).toEqual(['灰色']);expect(card.subtitle).toBe('修格斯');expect(card.cost).toBe(3);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ045.jpg');});
it('BQ065 preserves the pinned original, public image and client scan mapping without admission',()=>{const id='BQ065',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe('353de36848cdae74af860cc046822f109d88f5d57df4a9a4b940df4220d3bee4');expect(sha(read('web/public/cards/BQ065.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ065.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ065.jpg');expect(card.supported).toBe(false);expect(card.text).toContain('达成推进X时');expect(card.text).toContain('额外放置一个护盾');render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ065.jpg');});
it('BQ067 preserves its original image, both indexes and closed status',()=>{const id='BQ067',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe('841cff95c3d76d7bfc9975665c43570dc66bc1fdf9c17f91728e2ade23ff093f');expect(sha(read('web/public/cards/BQ067.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ067.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ067.jpg');expect(card.supported).toBe(false);expect(card.text).toContain('支付1名誉点');render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ067.jpg');});

it('BQ060 preserves its original image, both indexes and closed status',()=>{const id='BQ060',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe('5be1ef3265c68c9c67395331146db754291bd553f8554f898c84cd2ee9a735c1');expect(sha(read('web/public/cards/BQ060.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ060.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ060.jpg');expect(card.supported).toBe(false);expect(card.text).toContain('修格斯秘社限定');render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ060.jpg');});

it('BQ062 preserves its original image, both indexes and closed status',()=>{const id='BQ062',card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe('6853c20e6b934b8733472e8fd1a5b50c8d90cc01250dc026fcbac2010d47fcdd');expect(sha(read('web/public/cards/BQ062.jpg'))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:'/cards/BQ062.jpg',source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/BQ062.jpg');expect(card.supported).toBe(false);expect(card.text).toContain('命令牌获得+4');render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/BQ062.jpg');});

it.each(['CQ11','TK008'])('%s preserves original artwork and closed admission',id=>{const source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card.supported).toBe(false);if(id==='TK008'){expect(card.cost).toBe(0);expect(card.costAppearance).toBe('absent');expect(card.keywords).toContain('非资产');}else{expect(card.cost).toBe(2);expect(card.loyalty).toEqual(['红色']);}render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src',`/cards/${id}.jpg`);});

it.each(['XG19','TK003'])('%s retains original pixels and closed admission',id=>{const source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card.supported).toBe(false);expect(card.keywords).toContain('非资产');if(id==='TK003'){expect(card.cost).toBe(0);expect(card.costAppearance).toBe('absent');expect(card.keywords).toContain('公开');}else{expect(card.cost).toBe(3);expect(card.loyalty).toEqual(['灰色','灰色']);}render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src',`/cards/${id}.jpg`);});

it.each(['BQ088','TK001'])('%s retains original art and closed admission',id=>{const source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card.supported).toBe(false);if(id==='TK001'){expect(card.cost).toBe(0);expect(card.costAppearance).toBe('absent');expect(card.keywords).toEqual(['迟缓','非资产']);}else{expect(card.cost).toBe(1);expect(card.loyalty).toEqual(['黑色']);expect(card.text).toContain('目标副名为圣甲虫产业');}render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src',`/cards/${id}.jpg`);});

it('CQ27 preserves its original death-domain artwork and closed admission',()=>{const id='CQ27',source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/CQ27.jpg');expect(card.supported).toBe(false);expect(card.cost).toBe(5);expect(card.loyalty).toEqual(['黑色']);expect(card.magic).toBe('死亡');render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/CQ27.jpg');});
it('JZ53 retains original pixels, death domain, two black loyalty and closed admission',()=>{const id='JZ53',source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/JZ53.jpg');expect(card.supported).toBe(false);expect(card.cost).toBe(4);expect(card.loyalty).toEqual(['黑色','黑色']);expect(card.magic).toBe('死亡');expect(card.unique).toBe(true);render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:card.name+'原始牌面'})).toHaveAttribute('src','/cards/JZ53.jpg');});

it('JC097 original art preserves split combat icons and closed stealth admission',()=>{const id='JC097',source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(cardScanUrl(id)).toBe('/cards/JC097.jpg');expect(card.supported).toBe(false);expect(card.cost).toBe(3);expect(card.loyalty).toEqual([]);expect(card.icons.permanent.combat).toBe(1);expect(card.icons.temporary.combat).toBe(1);});

it('JC028 original art retains blue loyalty, blood domain and closed character admission',()=>{const id='JC028',source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/JC028.jpg');expect(card.supported).toBe(false);expect(card.kind).toBe('character');expect(card.cost).toBe(1);expect(card.loyalty).toEqual(['蓝色']);expect(card.magic).toBe('鲜血');expect(card.icons.permanent.combat).toBe(1);});
it('JC053 original gold unique red badge retains printed second cost 2 and closed admission',()=>{const id='JC053',source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe('/cards/JC053.jpg');expect(card).toMatchObject({supported:false,kind:'attachment',unique:true,cost:2,color:'红',loyalty:['红色']});expect(card.text).toContain('快速行动2');});

it("XQ13 retains original blue three-loyalty path art and closed admission",()=>{const id="XQ13",source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card).toMatchObject({supported:false,cost:1,color:"蓝",loyalty:["蓝色","蓝色","蓝色"],subtypes:["道途","吸血鬼"]});});

it("WM027 retains original blue three-loyalty path art and closed admission",()=>{const id="WM027",source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card).toMatchObject({supported:false,cost:1,color:"蓝",loyalty:["蓝色","蓝色","蓝色"],subtypes:["道途","吸血鬼"]});});

it("WM036 preserves original red ritual art and closed nonasset identity",()=>{const id="WM036",source=primary.cards[id].sourceVerification,card=registry.cards.find(c=>c.id===id);expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card).toMatchObject({supported:false,cost:1,color:"红",loyalty:["红色","红色"],subtypes:["仪式","灾难"],keywords:["非资产"]});});

it("WM031 keeps original art and remains closed",()=>{const id="WM031",source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(registry.cards.find(c=>c.id===id)).toMatchObject({supported:false,cost:2,color:"红",loyalty:["红色"],defense:1});});

it("BQ044 preserves its exact original and closed core identity",()=>{const id="BQ044",card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read("web/public/cards/BQ044.jpg"))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:"/cards/BQ044.jpg",source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe("/cards/BQ044.jpg");expect(card.supported).toBe(false);expect(card.loyalty).toEqual(["红色"]);expect(card.icons.permanent.influence).toBe(1);render(<DeckCardFace card={card} large/>);expect(screen.getByRole("img",{name:card.name+"原始牌面"})).toHaveAttribute("src","/cards/BQ044.jpg");});

it.each(["WM026","ZHWM026"])("%s original form art stays exact and closed",id=>{const card=registry.cards.find(c=>c.id===id),source=primary.cards[id].sourceVerification;expect(sha(read(source.imagePath))).toBe(source.imageSha256);expect(sha(read(`web/public/cards/${id}.jpg`))).toBe(source.imageSha256);expect(scans[id]).toEqual({url:`/cards/${id}.jpg`,source:source.imagePath,sha256:source.imageSha256});expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);expect(card.supported).toBe(false);render(<DeckCardFace card={card} large/>);expect(screen.getByRole("img",{name:card.name+"原始牌面"})).toHaveAttribute("src",`/cards/${id}.jpg`);});
