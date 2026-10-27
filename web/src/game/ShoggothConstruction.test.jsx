import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent} from '@testing-library/react';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {DeckLibrary} from './DeckLibraryPanel';
import {decodeDraft,publicDeckDraft,saveDeckDraft,validateDeckDraft} from './deckLibrary';
import {cardScanUrl} from './cardScans';
afterEach(()=>{cleanup();localStorage.clear();});
const candidateCatalog=()=>{const c=structuredClone(catalog);for(const card of c.cards)if(['BQ046','BQ048','BQ050','BQ051'].includes(card.id))card.supported=true;c.societies.find(s=>s.id==='MSBQ02').supported=true;return c;};
const draft=()=>({id:'closed-shoggoth50',name:'封闭候选50+1',description:'Private test registry; public admission remains closed',societyId:'MSBQ02',coreCardId:'BQ046',cards:[['BQ064',3],['BQ048',3],['BQ050',3],['BQ051',3],['JC125',38]].map(([cardId,count])=>({cardId,count})),rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:'2026-10-27T00:00:00Z'});
it('keeps public admission closed and validates all50+1 constraints in an explicit test registry',()=>{
 const d=draft(),c=candidateCatalog();expect(validateDeckDraft(d,catalog).valid).toBe(false);expect(validateDeckDraft(d,c)).toMatchObject({valid:true,total:50});expect(decodeDraft(publicDeckDraft(d))).toEqual(d);
 const bad=structuredClone(d);bad.cards.find(e=>e.cardId==='BQ048').count=2;bad.cards.find(e=>e.cardId==='JC125').count=39;expect(validateDeckDraft(bad,c).issues.some(i=>i.code==='society-command')).toBe(true);
 const noCore={...d,coreCardId:undefined};expect(validateDeckDraft(noCore,c).valid).toBe(false);expect(validateDeckDraft({...d,societyId:null},c).valid).toBe(false);
 expect(catalog.societies.find(s=>s.id==='MSBQ02').supported).toBe(false);
});
it('selects, saves and submits a separate core through the existing deck editor',()=>{
 const d={...draft(),coreCardId:undefined},c=candidateCatalog(),submit=vi.fn();expect(saveDeckDraft(d)).toBeNull();render(<DeckLibrary catalog={c} onSelectDraft={submit}/>);
 expect(screen.getByRole('button',{name:'保存并选择此牌组'})).toBeDisabled();fireEvent.change(screen.getByRole('combobox',{name:'独立核心（50+1）'}),{target:{value:'BQ046'}});fireEvent.click(screen.getByRole('button',{name:'保存并选择此牌组'}));
 expect(submit).toHaveBeenCalledOnce();expect(submit.mock.calls[0][0]).toMatchObject({coreCardId:'BQ046',societyId:'MSBQ02',cards:d.cards});expect(submit.mock.calls[0][0].cards.reduce((n,e)=>n+e.count,0)).toBe(50);
});
it.each(['BQ050','BQ051','MSBQ02'])('%s retains original/public/index bytes and stays closed',id=>{
 const spec=JSON.parse(readFileSync('../docs/factions/card-specifications.json')).cards[id],scans=JSON.parse(readFileSync('public/card-scans.json'));
 const {imagePath,imageSha256}=spec.sourceVerification;const sha=p=>createHash('sha256').update(readFileSync(p)).digest('hex');expect(sha('../'+imagePath)).toBe(imageSha256);expect(sha('public/cards/'+id+'.jpg')).toBe(imageSha256);expect(scans[id]).toEqual({url:'/cards/'+id+'.jpg',source:imagePath,sha256:imageSha256});expect(cardScanUrl(id)).toBe('/cards/'+id+'.jpg');expect([...catalog.cards,...catalog.societies].find(c=>c.id===id).supported).toBe(false);
});
