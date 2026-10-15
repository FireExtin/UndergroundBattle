import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import lifecycle from './currentLifecycle74.fixture.json';
import { testCard, testChoice } from './testFixtures';
import type { Choice } from './types';

afterEach(() => { vi.restoreAllMocks(); sessionStorage.clear(); });

describe('ordered card reading', () => {
  it.each(lifecycle.cases.filter(row => /^red:002[16]-/.test(row.name)))('$name reads duplicate-name cards without changing the draft and retains candidate identities across piles', row => {
    const view = row.beforeViews[row.seat];
    const choice = view.pendingChoice as Choice;
    const action = view.legalActions.find(item => item.kind === 'choose')!;
    const read = vi.fn(), submit = vi.fn();
    const store = { storage: sessionStorage, key: `ordered-reading-${row.name}` };
    const writes = vi.spyOn(Storage.prototype, 'setItem');
    const { container } = render(<ChoicePanel choice={choice} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId={view.you} draftStore={store} />);
    const candidate = (number: number) => container.querySelector<HTMLLIElement>(`.hg-order-list [data-choice-candidate-number="${number}"]`)!;
    const readCandidate = (number: number) => {
      const saved = store.storage.getItem(store.key), writeCount = writes.mock.calls.length;
      fireEvent.click(within(candidate(number)).getByRole('button', { name: `放大阅读${choice.options[number - 1].card!.name}` }));
      expect(read).toHaveBeenLastCalledWith(choice.options[number - 1].card);
      expect(store.storage.getItem(store.key)).toBe(saved);
      expect(writes).toHaveBeenCalledTimes(writeCount);
      expect(submit).not.toHaveBeenCalled();
    };
    expect(choice.kind).toBe('investigation');
    expect(choice.previewCards).toBeUndefined();
    const sameNameReaders = screen.getAllByRole('button', { name: '放大阅读无知路人' });
    expect(sameNameReaders).toHaveLength(2);
    sameNameReaders.forEach((reader, index) => expect(reader).toHaveAccessibleDescription(`候选 ${index + 2}`));
    choice.options.forEach(option => expect(container.textContent).not.toContain(option.id));
    expect(candidate(3).querySelector('.hg-order-number')).toHaveTextContent('3');
    readCandidate(3);
    expect(writes).not.toHaveBeenCalled();
    fireEvent.click(within(candidate(3)).getByRole('button', { name: '无知路人上移' }));
    expect(candidate(3).querySelector('.hg-order-number')).toHaveTextContent('2');
    readCandidate(3);
    fireEvent.click(within(candidate(3)).getByRole('button', { name: '置于底' }));
    fireEvent.click(within(candidate(1)).getByRole('button', { name: '置于底' }));
    expect(candidate(3).querySelector('.hg-order-number')).toHaveTextContent('1');
    readCandidate(3);
    fireEvent.click(within(candidate(3)).getByRole('button', { name: '置于顶' }));
    expect(candidate(3).querySelector('.hg-order-number')).toHaveTextContent('2');
    fireEvent.click(within(candidate(3)).getByRole('button', { name: '无知路人上移' }));
    expect(candidate(3).querySelector('.hg-order-number')).toHaveTextContent('1');
    expect(within(candidate(3)).getByRole('button', { name: '放大阅读无知路人' })).toHaveAccessibleDescription('候选 3');
    fireEvent.click(within(candidate(2)).getByRole('button', { name: '置于底' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, ...row.command.action.action });
  });

  it('uses public card identities in ordered labels, controls and reader payloads', () => {
    const read = vi.fn(), submit = vi.fn();
    const choice: Choice = { ...testChoice, kind: 'order', options: [{ id: 'hidden-option', label: '私有服务器标签', card: {
      ...testCard, cardId: 'secret-card', name: '私有卡名', text: '私有能力', owner: 'p1', controller: 'p1', faceDown: true,
    } }] };
    const { container } = render(<ChoicePanel choice={choice} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
    const reader = screen.getByRole('button', { name: '放大阅读暗藏者' });
    expect(reader).toHaveAccessibleDescription('候选 1');
    expect(screen.getByRole('button', { name: '暗藏者上移' })).toBeDisabled();
    expect(container.innerHTML).not.toMatch(/私有|secret-card/);
    fireEvent.click(reader);
    expect(read).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ name: '暗藏者', kind: 'hidden', cardId: undefined, text: undefined, icons: undefined }));
    expect(submit).not.toHaveBeenCalled();
  });

  it('keeps text-only order controls and their action unchanged', () => {
    const submit = vi.fn();
    const action = { id: 'text-order', kind: 'choose', label: '确认选择', choiceId: testChoice.id };
    const { container } = render(<ChoicePanel choice={{ ...testChoice, kind: 'order' }} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={vi.fn()} />);
    expect(screen.queryByRole('button', { name: /^放大阅读/ })).not.toBeInTheDocument();
    expect(container.querySelector('[data-choice-candidate-number]')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '角色乙上移' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, top: ['b', 'a'], bottom: [] });
  });
});
