import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {DeckCardFace} from './DeckCardFace';
import {cardScanUrl} from './cardScans';
import {actionForRoom} from './api';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/cq10.json.gz');
const card=catalog.cards.find(c=>c.id==='CQ10');
afterEach(cleanup);
it('delivers the pinned CQ10 face and full closed vampire scholar definition',()=>{
 const specs=JSON.parse(readFileSync(resolve('../docs/factions/card-specifications.json'))).cards.CQ10;
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json')));
 const {imagePath,imageSha256}=specs.sourceVerification;
 const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
 expect(hash(resolve('..',imagePath))).toBe(imageSha256);expect(hash(resolve('public/cards/CQ10.jpg'))).toBe(imageSha256);
 expect(scans.CQ10).toEqual({url:'/cards/CQ10.jpg',source:imagePath,sha256:imageSha256});expect(cardScanUrl('CQ10')).toBe('/cards/CQ10.jpg');
 expect(card).toMatchObject({supported:false,unique:true,cost:8,permanentIcons:{investigation:1,combat:1,influence:2},temporaryIcons:{investigation:1,combat:0,influence:1},defense:4,magic:'鲜血',keywords:['公开〔日形〕领袖']});
 render(<DeckCardFace card={card} large/>);expect(screen.getByRole('img',{name:'朱苏德拉·纳匹什提姆原始牌面'})).toHaveAttribute('src','/cards/CQ10.jpg');
});
const choices=fixture.cases.flatMap(row=>row.steps.filter(s=>s.command.action.action?.kind==='choose').map(step=>({name:row.name+' '+JSON.parse(abi.view(step.state,step.seat)).pendingChoice.kind,step})));
it.each(choices)('$name submits the exact Native player, discard or preparation choice',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action;
 const submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 for(const id of wanted.selected){const option=Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id);expect(option).toBeDefined();fireEvent.click(option);}
 const confirm=within(dialog).getByRole('button',{name:'确认选择'});expect(confirm).toBeEnabled();fireEvent.click(confirm);
 expect(submit).toHaveBeenCalled();expect(actionForRoom(view,submit.mock.calls.at(-1)[0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
