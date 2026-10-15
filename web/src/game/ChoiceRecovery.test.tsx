import { StrictMode } from 'react';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { createChoiceDraftStore, readChoiceDraft } from './choiceDraftStorage';
import { Table } from './Table';
import { testCard, testCatalog, testChoice, testView } from './testFixtures';
import type { Choice, View } from './types';

const choose = { id: 'recover-choice', kind: 'choose', label: '确认选择', choiceId: testChoice.id };
const choosing = (choice: Choice = testChoice): View => ({
  ...testView, status: 'playing', pendingChoice: choice, legalActions: [choose],
});

describe('batched choice edits', () => {
  const stored = (view: View) => readChoiceDraft(createChoiceDraftStore(view.roomId, view.you), view.pendingChoice!);

  it('accumulates target clicks and persists the latest edit synchronously in StrictMode', () => {
    const submit = vi.fn();
    const writes = vi.spyOn(Storage.prototype, 'setItem');
    const view = choosing({ ...testChoice, min: 2, max: 2 });
    const first = render(<StrictMode><Table view={view} catalog={testCatalog} busy={false} onAction={submit} /></StrictMode>);
    const a = option('a'), b = option('b');
    act(() => {
      a.click(); b.click();
      expect(stored(view)?.selected).toEqual(['a', 'b']);
    });
    expect(writes).toHaveBeenCalledTimes(2);
    expect(option('a')).toHaveAttribute('aria-pressed', 'true');
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: ['a', 'b'] });
  });

  it('accumulates damage on repeated and different targets without exceeding the shared total', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind: 'damage', amount: 3 });
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    const a = option('a'), b = option('b');
    act(() => {
      a.click(); a.click(); b.click(); b.click();
      expect(stored(view)?.allocations).toEqual({ a: 2, b: 1 });
    });
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(2);
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, allocations: { a: 2, b: 1 } });
  });

  it('moves each ordered card once even when its previous pile button is clicked again', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind: 'investigation' });
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    const [a, b] = screen.getAllByRole<HTMLButtonElement>('button', { name: '置于底' });
    act(() => {
      a.click(); a.click(); b.click();
      expect(stored(view)).toMatchObject({ top: [], bottom: ['a', 'b'] });
    });
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, top: [], bottom: ['a', 'b'] });
  });

  it.each(['investigation', 'region_return'])('applies successive moves to the latest %s order', kind => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind, min: 3, max: 3, options: [...testChoice.options, { id: 'c', label: '角色丙' }] });
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    const up = screen.getByRole<HTMLButtonElement>('button', { name: '角色丙上移' });
    act(() => {
      up.click(); up.click();
      expect(stored(view)?.[kind === 'investigation' ? 'top' : 'selected']).toEqual(['c', 'a', 'b']);
    });
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith(kind === 'investigation'
      ? { ...choose, top: ['c', 'a', 'b'], bottom: [] }
      : { ...choose, selected: ['c', 'a', 'b'] });
  });
});
const option = (id: string) => screen.getByRole('dialog', { name: '待完成的选择' })
  .querySelector<HTMLButtonElement>(`[data-choice-option="${id}"]`)!;
const confirm = () => screen.getByRole('button', { name: '确认选择' });

beforeEach(() => { localStorage.clear(); sessionStorage.clear(); });
afterEach(() => { vi.restoreAllMocks(); localStorage.clear(); sessionStorage.clear(); });

describe('pending choice draft recovery', () => {
  it('restores a selected target after a fresh table mount without submitting automatically', () => {
    const submit = vi.fn();
    const view = choosing();
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(option('b'));
    first.unmount();
    render(<Table view={{ ...view, version: 2 }} catalog={testCatalog} busy={false} onAction={submit} />);
    expect(option('b')).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: ['b'] });
  });

  it('retains a partial selection while the room is paused and after resuming', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, min: 2, max: 2 });
    const content = (current: View) => <Table view={current} catalog={testCatalog} busy={!!current.pause} onAction={submit} />;
    const { rerender } = render(content(view));
    fireEvent.click(option('b'));
    expect(confirm()).toBeDisabled();
    rerender(content({ ...view, version: 2, pause: { pausedAtMs: 2_000, pausedBy: 0 }, legalActions: [] }));
    expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
    rerender(content({ ...view, version: 3 }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'true');
    expect(confirm()).toBeDisabled();
    fireEvent.click(option('a'));
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: ['b', 'a'] });
  });

  it('restores the top and bottom order without returning moved cards to the default pile', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind: 'investigation' });
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '角色乙上移' }));
    fireEvent.click(screen.getAllByRole('button', { name: '置于底' })[1]);
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, top: ['b'], bottom: ['a'] });
  });

  it('restores the ordered selection for cards returned from a region', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind: 'region_return', min: 2, max: 2 });
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '角色乙上移' }));
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: ['b', 'a'] });
  });

  it('restores an incomplete damage allocation and still requires the exact total', () => {
    const submit = vi.fn();
    const view = choosing({ ...testChoice, kind: 'damage', amount: 3 });
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.change(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' }), { target: { value: '2' } });
    first.unmount();
    render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(2);
    expect(confirm()).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '给角色乙增加1点伤害' }));
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, allocations: { a: 2, b: 1 } });
  });

  it('does not carry selections into a replacement choice in an unkeyed standalone panel', () => {
    const submit = vi.fn();
    const content = (choice: Choice) => <ChoicePanel choice={choice} action={{ ...choose, choiceId: choice.id }} definitions={new Map()} busy={false} onSubmit={submit} />;
    const { rerender } = render(content(testChoice));
    fireEvent.click(screen.getByRole('button', { name: '+角色乙' }));
    rerender(content({ ...testChoice, id: 'next-choice', options: [{ id: 'new-target', label: '新目标' }] }));
    expect(confirm()).toBeDisabled();
    expect(within(screen.getByRole('region', { name: '选择选项，可滚动查看' })).getByRole('button', { name: '+新目标' })).toHaveAttribute('aria-pressed', 'false');
    expect(submit).not.toHaveBeenCalled();
  });

  it.each([
    ['constraints', { ...testChoice, min: 2, max: 2 }],
    ['decline permission', { ...testChoice, allowDecline: true }],
    ['option instances', { ...testChoice, options: testChoice.options.map(item => ({ ...item, card: { ...testCard, instanceId: `new-${item.id}` } })) }],
    ['option order', { ...testChoice, options: [...testChoice.options].reverse() }],
  ] as const)('invalidates a previous selection when the same choice ID changes its %s', (_, changed) => {
    const submit = vi.fn();
    const view = choosing();
    const content = (current: View) => <Table view={current} catalog={testCatalog} busy={false} onAction={submit} />;
    const { rerender, unmount } = render(content(view));
    fireEvent.click(option('b'));
    rerender(content({ ...view, version: 2, pendingChoice: changed }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'false');
    unmount();
    render(content({ ...view, version: 3 }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'false');
    expect(submit).not.toHaveBeenCalled();
  });

  it('keeps unacknowledged selections but clears them after the server closes the choice', () => {
    const submit = vi.fn();
    const view = choosing();
    const content = (current: View) => <Table view={current} catalog={testCatalog} busy={false} onAction={submit} />;
    const first = render(content(view));
    fireEvent.click(option('b'));
    fireEvent.click(confirm());
    first.unmount();
    const retry = render(content(view));
    expect(option('b')).toHaveAttribute('aria-pressed', 'true');
    expect(submit).toHaveBeenCalledTimes(1);
    retry.rerender(content({ ...view, version: 2, pendingChoice: null, legalActions: [] }));
    retry.unmount();
    render(content({ ...view, version: 3 }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'false');
  });

  it('keeps each room and seat draft separate while switching between tables', () => {
    const submit = vi.fn();
    const view = choosing();
    const content = (current: View) => <Table view={current} catalog={testCatalog} busy={false} onAction={submit} />;
    const { rerender } = render(content(view));
    fireEvent.click(option('b'));
    rerender(content({ ...view, roomId: 'another-room' }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(option('a'));
    rerender(content({ ...view, you: 'p1', pendingChoice: { ...testChoice, playerId: 'p1' } }));
    expect(option('b')).toHaveAttribute('aria-pressed', 'false');
    rerender(content(view));
    expect(option('b')).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
  });

  it('shows only the latest server option labels when restoring draft IDs', () => {
    const submit = vi.fn();
    const view = choosing();
    const first = render(<Table view={view} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(option('b'));
    first.unmount();
    const updated = { ...testChoice, title: '新的标题', options: testChoice.options.map(item => ({ ...item, label: `最新角色${item.id}` })) };
    render(<Table view={choosing(updated)} catalog={testCatalog} busy={false} onAction={submit} />);
    expect(option('b')).toHaveAccessibleName('1最新角色b');
    expect(option('b')).toHaveAttribute('aria-pressed', 'true');
    expect(screen.queryByText('角色乙')).not.toBeInTheDocument();
    expect(Array.from({ length: sessionStorage.length }, (_, i) => sessionStorage.getItem(sessionStorage.key(i)!)).join('')).not.toMatch(/角色|选择目标/);
    expect(localStorage.length).toBe(0);
  });

  it('continues explicit selection and submission when browser storage is unavailable', () => {
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new DOMException('denied', 'SecurityError'); });
    const submit = vi.fn();
    render(<Table view={choosing()} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(option('b'));
    fireEvent.click(confirm());
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: ['b'] });
  });
});
