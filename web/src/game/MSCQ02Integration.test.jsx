import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/mscq02.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('keeps red Norvi closed and checks its actual printed construction',()=>{const draft=fixture.cases[0].construction.deck;expect(catalog.societies.find(s=>s.id==='MSCQ02')).toMatchObject({color:'红',supported:false,startingHand:5});expect(validateDeckDraft(draft,catalog).valid).toBe(false);const candidate={...catalog,societies:catalog.societies.map(s=>s.id==='MSCQ02'?{...s,supported:true}:s)};expect(validateDeckDraft(draft,candidate).valid).toBe(true);for(const card of catalog.cards.filter(c=>c.supported&&c.kind!=='region')){const probe={...draft,cards:[{cardId:'JC125',count:49},{cardId:card.id,count:1}]};const allowed=['红','中立'].includes(card.color)||card.subtypes.some(s=>['灾难','时间','命运'].includes(s));expect(validateDeckDraft(probe,candidate).issues.some(i=>i.code==='society-disaster-time-fate')).toBe(!allowed);}});
const triggerSteps=fixture.cases[0].steps.filter(s=>{const r=JSON.parse(s.state);return r.game?.pending?.resolution?.Declare?.declaration?.source?.card?.definition==='MSCQ02'&&s.command.action.action?.kind==='choose';});
it('records both original optional trigger entries',()=>{expect(triggerSteps).toHaveLength(2);});
it.each(triggerSteps.map((step,i)=>({step,name:i===0?'initial third-region destruction':'destroyed-region win draw'})))('$name submits the actual optional trigger choice',({step})=>{const view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);const dialog=screen.getByRole('dialog',{name:'待完成的选择'});expect(view.pendingChoice.min).toBe(0);fireEvent.click(dialog.querySelector('[data-choice-option="accept"]'));fireEvent.click(within(dialog).getByRole('button',{name:/确认/}));expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);});

it('does not present internal event observations as once-per-game abilities',()=>{const step=fixture.cases[0].steps[0],view=JSON.parse(abi.view(step.state,step.seat));render(<Table catalog={catalog} view={view} busy={false} onAction={()=>{}}/>);expect(screen.queryByText(/本局已使用/)).not.toBeInTheDocument();expect(view.societyZones[0].card.usedOncePerGame).toBeUndefined();});
