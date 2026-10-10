import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testCard, testChoice } from './testFixtures';
import { Table } from './Table';
import native48 from './jz50Native48Test.fixture.json';
import type { Catalog, View } from './types';

const action = {id:'choose-search',kind:'choose',label:'完成检索',choiceId:'search-choice'};
const base = {...testChoice,id:'search-choice',kind:'jz50_death_search',min:0,max:1,allowDecline:false,title:'检索零或一张死亡领域角色置墓并洗牌'};

it.each([{options:[]},{options:[{id:'death-one',label:'墓穴食尸鬼',card:{...testCard,instanceId:'death-one',cardId:'JZ50',name:'墓穴食尸鬼'}}]}])('completes an accepted JZ50 search with zero cards and still requests the shuffle for %j', ({options}) => {
  const submit=vi.fn();
  render(<ChoicePanel choice={{...base,options}} action={action} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
  expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  const confirm=screen.getByRole('button',{name:'不取牌并洗牌'});
  expect(confirm).toBeEnabled();fireEvent.click(confirm);
  expect(submit).toHaveBeenCalledExactlyOnceWith({...action,choiceId:base.id,selected:[]});
});

it('reads privately without selecting, chooses one real instance and disables another until deselection', () => {
  const submit=vi.fn(),read=vi.fn();
  const options=['first','second'].map(id=>({id,label:id,card:{...testCard,instanceId:id,cardId:'JZ50',name:'墓穴食尸鬼'}}));
  const {container}=render(<ChoicePanel choice={{...base,options}} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
  fireEvent.click(screen.getAllByRole('button',{name:'放大阅读墓穴食尸鬼'})[0]);
  expect(read).toHaveBeenCalledExactlyOnceWith(options[0].card);expect(submit).not.toHaveBeenCalled();
  const first=container.querySelector('[data-choice-option="first"]')!;
  const second=container.querySelector('[data-choice-option="second"]')!;
  expect(first).toHaveAttribute('aria-pressed','false');fireEvent.click(first);
  expect(second).toBeDisabled();
  fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
  expect(submit).toHaveBeenCalledExactlyOnceWith({...action,choiceId:base.id,selected:['first']});
});

it('keeps unrelated mandatory private choices from inheriting JZ50 zero confirmation', () => {
  render(<ChoicePanel choice={{...base,kind:'search',options:[]}} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} viewerId="p0" />);
  expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
  expect(screen.queryByRole('button',{name:'不取牌并洗牌'})).not.toBeInTheDocument();
});

it('uses actual Native controller-private choices for zero completion and closes the pending modal after real owner-graveyard projection', () => {
  const submit=vi.fn();
  const initial=native48.private.views[0] as unknown as View;
  const catalog=native48.private.catalog as unknown as Catalog;
  const {rerender}=render(<Table view={initial} catalog={catalog} busy={false} onAction={submit} />);
  expect(screen.getByRole('dialog',{name:'待完成的选择'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'不取牌并洗牌'}));
  const legal=initial.legalActions.find(a=>a.kind==='choose')!;
  expect(submit).toHaveBeenCalledExactlyOnceWith({...legal,choiceId:initial.pendingChoice!.id,selected:[]});
  // This projection comes from an actual one-card choice. It is not generated
  // by the mocked callback above and does not pretend to test a live backend.
  const complete=native48.selected.views[0] as unknown as View;
  rerender(<Table view={complete} catalog={catalog} busy={false} onAction={submit} />);
  expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
  expect(complete.graveyard.some(c=>c.cardId==='JC032')).toBe(true);
});

it('offers the same empty private confirmation while every other actual Native seat remains unable to inspect or choose', () => {
  const catalog=native48.empty.catalog as unknown as Catalog;
  const empty=native48.empty.views[0] as unknown as View;
  const {rerender,container}=render(<Table view={empty} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.getByRole('button',{name:'不取牌并洗牌'})).toBeEnabled();
  for(const seat of [1,2,3]) {
    rerender(<Table view={native48.private.views[seat] as unknown as View} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
    expect(screen.queryByRole('button',{name:'不取牌并洗牌'})).not.toBeInTheDocument();
    expect(container.querySelector('[data-choice-option]')).toBeNull();
  }
});
