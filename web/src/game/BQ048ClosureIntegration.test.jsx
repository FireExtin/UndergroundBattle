import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,fireEvent} from '@testing-library/react';
import {kernel,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/bq048-closure.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('retains red identity, original three modes, closed admission and real fifty-plus-core input',()=>{expect(catalog.cards.find(c=>c.id==='BQ048')).toMatchObject({supported:false,color:'红',cost:1,subtypes:['命令','攻击']});expect(fixture.cases[0].construction).toMatchObject({ordinaryCards:50,separateCore:1,minimumCommandCards:12,noLayoutOrResourceMutation:true});});
for(const c of fixture.cases.filter(c=>c.name.includes('ready'))){
 it(`${c.name}: submits paid ready through the actual object button after refresh`,()=>{const step=c.steps.find(s=>s.command.action.action?.option==='ready'),a=step.command.action.action,view=JSON.parse(kernel.view(step.state,step.seat)),send=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={send}/>);const legal=view.legalActions.find(l=>l.kind==='play'&&l.cardId===a.cardId&&l.option==='ready');expect(legal).toBeDefined();fireEvent.click(ui.container.querySelector(`[data-card-instance="${a.cardId}"]`));const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(b=>b.getAttribute('data-action-id')===legal.id);expect(button).toBeEnabled();fireEvent.click(button);expect(send).toHaveBeenCalledOnce();expect(actionForRoom(view,send.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(kernel.applyRoom(step.state,step.seat,JSON.stringify(step.command),step.serverNowMs))).toEqual(step.expected);});
 it(`${c.name}: final core marker is public and survives rerender`,()=>{const step=c.steps.at(-1);for(let seat=0;seat<4;seat++){const view=JSON.parse(kernel.view(step.expected.state,seat));const core=view.coreZones[0].card||view.regions.flatMap(r=>r.characters).find(c=>c.cardId==='BQ046');expect(core.permanentCombatMarkers).toBe(1);const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={vi.fn()}/>);expect(ui.container.querySelector(`[data-card-instance="${core.instanceId}"]`)).toBeTruthy();ui.unmount();}});
}
