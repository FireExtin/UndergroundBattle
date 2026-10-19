// Actual new kernel/Native P2 data; component checks are not browser playtests.
import {render} from '@testing-library/react';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {kernel as abi,catalog,moduleBytes} from '../../../rust-game-wasm/tests/current88-kernel.mjs';
import {loadP2Fixture,reviewSummaryCases} from '../../../tools/xq23-p2-oracle.mjs';
import {sha256} from '../../../tools/xq23-oracle.mjs';
import {StackTargets} from './ResponseWindow';
import {Table} from './Table';
const fixture=loadP2Fixture(),boundary=reviewSummaryCases();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());

it.each([0,1,2,3])('seat%s binds the real paid target UID to the public target DOM',seat=>{
 const view=JSON.parse(abi.view(boundary.control,seat)),effect=view.stack[0];
 expect(effect.targetSummaries[0].instanceId).toBe('i243');
 const mounted=render(<StackTargets effect={effect} view={view}/>);
 expect(mounted.container.querySelector('[data-target-instance="i243"]')).toHaveAttribute('data-target-valid','true');
 expect(mounted.container.querySelector('[data-target-instance="i241"]')).toBeNull();
});

it.each(boundary.invalid)('$name cannot produce a view for a target DOM',bad=>{
 expect(()=>abi.view(bad.state,0)).toThrow(/XQ23/);
});

it('removes the departed owner asset from the actual table while retaining a live foreign owner asset',()=>{
 const before=JSON.parse(fixture.beforeElimination).game;
 const removed=before.players[0].assets.find(c=>c.definition==='XQ23'&&c.owner===2).id;
 const retained=before.players[0].assets.find(c=>c.definition==='XQ23'&&c.owner===3).id;
 const view=JSON.parse(abi.view(fixture.afterElimination,0));
 expect(view.players[2]).toMatchObject({eliminated:true,score:4});expect(view.players[3]).toMatchObject({eliminated:false,score:2});
 const mounted=render(<Table view={view} catalog={catalog} busy={false} connection='online' onAction={vi.fn()}/>);
 expect(mounted.container.querySelector(`[data-card-instance="${removed}"]`)).toBeNull();
 const asset=mounted.container.querySelector(`[data-card-instance="${retained}"]`);
 expect(asset).toHaveAccessibleName('查看资产');expect(asset).toHaveAttribute('data-card-owner','p3');expect(asset).toHaveAttribute('data-card-controller','p0');
});
