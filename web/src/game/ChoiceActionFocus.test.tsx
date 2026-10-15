import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { StrictMode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testCard, testChoice } from './testFixtures';

const action = { id: 'focus-choice', kind: 'choose', label: '确认选择', choiceId: testChoice.id };
const options = [...testChoice.options, { id: 'c', label: '角色丙' }];
const store = () => ({ storage: sessionStorage, key: 'choice-action-focus' });
afterEach(() => { vi.restoreAllMocks(); sessionStorage.clear(); });

describe('choice action focus recovery', () => {
  it.each(['分配', '增加'])('keeps damage %s edits on the same candidate after reaching the total and returning to zero', verb => {
    const submit = vi.fn(), read = vi.fn(), writes = vi.spyOn(Storage.prototype, 'setItem');
    render(<StrictMode><ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 2 }} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} draftStore={store()} /></StrictMode>);
    const source = screen.getByRole('button', { name: `给角色甲${verb}1点伤害` });
    const minus = screen.getByRole('button', { name: '给角色甲减少1点伤害' });
    const target = screen.getByRole('button', { name: '给角色甲分配1点伤害' });
    source.focus();
    fireEvent.click(source); expect(source).toHaveFocus();
    fireEvent.click(source); expect(minus).toHaveFocus();
    expect(source).toBeDisabled();
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(2);
    expect(writes).toHaveBeenCalledTimes(2);
    fireEvent.click(minus); expect(minus).toHaveFocus();
    fireEvent.click(minus); expect(target).toHaveFocus();
    expect(minus).toBeDisabled();
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(0);
    expect(writes).toHaveBeenCalledTimes(4);
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    expect(read).not.toHaveBeenCalled();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(['order', 'investigation', 'region_return'])('retains the %s candidate at both ordering boundaries without extra edits', kind => {
    const submit = vi.fn(), writes = vi.spyOn(Storage.prototype, 'setItem');
    render(<StrictMode><ChoicePanel choice={{ ...testChoice, kind, min: 3, max: 3, options }} action={action} definitions={new Map()} busy={false} onSubmit={submit} draftStore={store()} /></StrictMode>);
    const up = screen.getByRole('button', { name: '角色丙上移' });
    const down = screen.getByRole('button', { name: '角色丙下移' });
    up.focus();
    fireEvent.click(up); expect(up).toHaveFocus();
    fireEvent.click(up); expect(down).toHaveFocus();
    expect(up).toBeDisabled();
    fireEvent.click(down); expect(down).toHaveFocus();
    fireEvent.click(down); expect(up).toHaveFocus();
    expect(down).toBeDisabled();
    expect(writes).toHaveBeenCalledTimes(4);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(kind === 'region_return'
      ? { ...action, selected: ['a', 'b', 'c'] } : { ...action, top: ['a', 'b', 'c'], bottom: [] });
  });

  it('follows the same card and transfer control between piles instead of its reader', () => {
    const submit = vi.fn(), read = vi.fn(), writes = vi.spyOn(Storage.prototype, 'setItem');
    const choice = { ...testChoice, kind: 'investigation', options: options.map(option => ({ ...option, card: { ...testCard, instanceId: option.id, name: option.label } })) };
    const { container } = render(<ChoicePanel choice={choice} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} draftStore={store()} viewerId="p0" />);
    const row = () => container.querySelector<HTMLElement>('[data-choice-order-option="b"]')!;
    const transfer = within(row()).getByRole('button', { name: '置于底' });
    transfer.focus(); fireEvent.click(transfer);
    expect(transfer).not.toBeInTheDocument();
    const back = within(row()).getByRole('button', { name: '置于顶' });
    expect(back).toHaveFocus();
    fireEvent.click(back);
    expect(within(row()).getByRole('button', { name: '置于底' })).toHaveFocus();
    expect(within(row()).getByRole('button', { name: '放大阅读角色乙' })).not.toHaveFocus();
    expect(writes).toHaveBeenCalledTimes(2);
    expect(read).not.toHaveBeenCalled();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, top: ['a', 'c', 'b'], bottom: [] });
  });

  it('does not move focus back from the search field after a queued order edit', () => {
    const manyOptions = Array.from({ length: 8 }, (_, index) => ({ id: `option-${index}`, label: `候选${index + 1}` }));
    const { rerender } = render(<ChoicePanel choice={{ ...testChoice, kind: 'region_return', min: 8, max: 8, options: manyOptions }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} />);
    const up = screen.getByRole('button', { name: '候选2上移' });
    const search = screen.getByRole('searchbox', { name: '检索选项' });
    up.focus();
    act(() => { up.click(); search.focus(); });
    expect(up).toBeDisabled();
    expect(search).toHaveFocus();
    rerender(<ChoicePanel choice={{ ...testChoice, kind: 'region_return', min: 8, max: 8, options: manyOptions }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} />);
    expect(search).toHaveFocus();
  });

  it('discards queued recovery while a nested reader owns the active layer', () => {
    const content = (modalActive: boolean) => <><button type="button">阅读层控制</button><ChoicePanel choice={{ ...testChoice, kind: 'order' }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} modalActive={modalActive} /></>;
    const { rerender } = render(content(true));
    const up = screen.getByRole('button', { name: '角色乙上移' });
    up.focus();
    act(() => { up.click(); up.blur(); rerender(content(false)); });
    expect(screen.getByRole('button', { name: '角色乙下移' })).not.toHaveFocus();
    const reader = screen.getByRole('button', { name: '阅读层控制' });
    reader.focus();
    rerender(content(true));
    expect(reader).toHaveFocus();
  });
});
