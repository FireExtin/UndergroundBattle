// Actual current Native choice nodes through the real Table controls. These
// component/remount checks are separate from browser and Worker acceptance.
import {cleanup,fireEvent,render,screen,within} from '@testing-library/react';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {kernel,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';

const fixture=loadCurrentFixture('rust-game-wasm/tests/current112/batch20-review-normal.json.gz');
expect(catalog.engineVersion).toBe('rust-v0.2.113-bq001-anchor-source-wip');
expect(catalog).toEqual(fixture.catalog);
const cases=fixture.cases.map(scenario=>{
  const kind=scenario.name.startsWith('cq28-')?'rumor':'batch3-entry-order';
  const index=scenario.steps.findIndex(row=>JSON.parse(row.state).game.pending?.choice.kind===kind);
  expect(index).toBeGreaterThan(0);
  const row=scenario.steps[index],previous=scenario.steps[index-1];
  expect(previous.expected.state).toBe(row.state);
  return {name:scenario.name,row,beforeViews:previous.views};
});
beforeEach(()=>sessionStorage.clear());
afterEach(()=>{cleanup();sessionStorage.clear();});

it.each(cases)('$name submits the exact current Native selection after a fresh component mount',({row,beforeViews})=>{
  const view=JSON.parse(kernel.view(row.state,row.seat));
  expect(view).toEqual(beforeViews[row.seat]);
  const submit=vi.fn();
  const props={catalog,busy:false,connection:'online',onAction:submit};
  const first=render(<Table {...props} view={view}/>);
  let dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  if(view.pendingChoice.kind==='batch3-entry-order'){
    expect(within(dialog).getByRole('button',{name:'确认选择'})).toBeDisabled();
    expect(within(dialog).queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  }
  const selected=row.command.action.action.selected;
  expect(selected).toHaveLength(1);
  const option=dialog.querySelector(`[data-choice-option="${selected[0]}"]`);
  expect(option).not.toBeNull();fireEvent.click(option);
  first.unmount();
  render(<Table {...props} view={JSON.parse(kernel.view(row.state,row.seat))}/>);
  dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(dialog.querySelector(`[data-choice-option="${selected[0]}"]`)).toHaveAttribute('aria-pressed','true');
  fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
  expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const actual=JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),row.serverNowMs));
  expect(actual).toEqual(row.expected);
  expect(actual.state).toBe(row.expected.state);
  row.views.forEach((expected,seat)=>expect(JSON.parse(kernel.view(actual.state,seat))).toEqual(expected));
});

it.each(cases)('$name exposes the real prompt only to its Native chooser',({row,beforeViews})=>{
  beforeViews.forEach((view,seat)=>{
    expect(JSON.parse(kernel.view(row.state,seat))).toEqual(view);
    const submit=vi.fn();const ui=render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={submit}/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(seat===row.seat);
    expect(!!view.pendingChoice).toBe(seat===row.seat);
    expect(submit).not.toHaveBeenCalled();ui.unmount();
  });
});
