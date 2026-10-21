import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { StrictMode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testChoice } from './testFixtures';
import { testCard } from './testFixtures';

const base = { id: 'choose-action', kind: 'choose', label: '选择', choiceId: 'choice-one' };
const boardChoice = { ...testChoice, options: [
  { id: 'anonymous-a', label: '标签中的私有身份甲', card: { ...testCard, instanceId: 'anonymous-a', cardId: 'secret-a', owner: 'p1', controller: 'p1', region: 0, faceDown: true, name: '私有身份甲', text: '私有能力甲' } },
  { id: 'anonymous-b', label: '标签中的私有身份乙', card: { ...testCard, instanceId: 'anonymous-b', cardId: 'secret-b', owner: 'p2', controller: 'p3', region: 2, faceDown: true, name: '私有身份乙', text: '私有能力乙' } },
] };
const playerLabels = { p0: '我方玩家', p1: '甲方玩家', p2: '乙方玩家', p3: '丁方玩家' };
const secretDefinitions = new Map(boardChoice.options.map(option => [option.card.cardId, { id: option.card.cardId, name: option.card.name, kind: 'character', cost: 7, text: option.card.text, permanentIcons: { investigation: 7, combat: 7, influence: 7 } }]));
const largeCardChoice = { ...testChoice, options: Array.from({ length: 8 }, (_, index) => ({
  id: `option-${index}`, label: `原标签${index + 1}`,
  card: { ...testCard, instanceId: `instance-${index}`, cardId: `JC${125 + index}`, name: `候选角色${index + 1}` },
})) };
describe('server-owned decisions', () => {
  it('confirms a JC032 zero-hit private view with no skip button', () => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'jc032_top_six', min: 0, max: 0, allowDecline: false, options: [], previewCards: [testCard] };
    render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '确认选择' })).toBeEnabled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: [] });
  });
  it('reads a mulligan card independently without selecting it or submitting the decision', () => {
    const submit = vi.fn();
    const read = vi.fn();
    const choice = { ...testChoice, kind: 'mulligan', min: 0, max: 6, allowDecline: true, options: [{ id: 'hand-a', label: testCard.name, card: testCard }] };
    const { container } = render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
    const option = container.querySelector('[data-choice-option="hand-a"]')!;
    fireEvent.click(screen.getByRole('button', { name: `放大阅读${testCard.name}` }));
    expect(read).toHaveBeenCalledExactlyOnceWith(testCard);
    expect(option).toHaveAttribute('aria-pressed', 'false');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(option);
    fireEvent.click(screen.getByRole('button', { name: `放大阅读${testCard.name}` }));
    expect(option).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: ['hand-a'] });
  });

  it('passes only a concealed public identity to a non-controller choice reader', () => {
    const read = vi.fn();
    const { container } = render(<ChoicePanel choice={boardChoice} action={base} definitions={secretDefinitions} busy={false} onSubmit={vi.fn()} onReadCard={read} viewerId="p0" playerLabels={playerLabels} />);
    fireEvent.click(screen.getAllByRole('button', { name: '放大阅读暗藏者' })[1]);
    expect(read).toHaveBeenCalledOnce();
    expect(read.mock.calls[0][0]).toMatchObject({ name: '暗藏者', owner: 'p2', controller: 'p3', kind: 'hidden' });
    expect(read.mock.calls[0][0].cardId).toBeUndefined();
    expect(read.mock.calls[0][0].text).toBeUndefined();
    expect(read.mock.calls[0][0].icons).toBeUndefined();
    expect(container.textContent).not.toMatch(/私有身份|私有能力/);
  });

  it('conceals private names in preview read labels as well as the displayed cards', () => {
    const read = vi.fn();
    const choice = { ...testChoice, kind: 'jc032_top_six', min: 0, max: 0, options: [], previewCards: boardChoice.options.map(option => option.card) };
    const { container } = render(<ChoicePanel choice={choice} action={base} definitions={secretDefinitions} busy={false} onSubmit={vi.fn()} onReadCard={read} viewerId="p0" />);
    const readers = screen.getAllByRole('button', { name: '放大阅读暗藏者' });
    expect(readers).toHaveLength(2);
    expect(container.innerHTML).not.toMatch(/私有身份|私有能力|secret-a|secret-b/);
    fireEvent.click(readers[1]);
    expect(read).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ instanceId: 'anonymous-b', name: '暗藏者', kind: 'hidden', cardId: undefined, text: undefined, icons: undefined }));
  });

  it.each([['anonymous-a', 1], ['anonymous-b', 2]] as const)('distinguishes anonymous board targets by public ownership and region and submits %s', (id, number) => {
    const submit = vi.fn();
    const { container } = render(<ChoicePanel choice={boardChoice} action={base} definitions={secretDefinitions} busy={false} onSubmit={submit} viewerId="p0" playerLabels={playerLabels} />);
    const first = screen.getByRole('button', { name: /目标 1.*甲方玩家 拥有.*地区 1/ });
    const second = screen.getByRole('button', { name: /目标 2.*乙方玩家 拥有.*丁方玩家 操控.*地区 3/ });
    expect(first).toHaveAttribute('data-choice-target-number', '1');
    expect(second).toHaveAttribute('data-choice-target-number', '2');
    expect(within(first).getByText('暗藏者', { exact: true })).toBeInTheDocument();
    expect(within(second).getByText('暗藏者', { exact: true })).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/私有身份|私有能力/);
    expect(screen.queryByLabelText('图标：调查7，战斗7，势力7')).not.toBeInTheDocument();
    const target = number === 1 ? first : second;
    expect(target).toHaveAttribute('data-choice-option', id);
    fireEvent.click(number === 1 ? second : first);
    expect(target).toBeEnabled();
    fireEvent.click(target);
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: boardChoice.id, selected: [id] });
  });

  it('disables board target selection and submission while busy', () => {
    const submit = vi.fn();
    render(<ChoicePanel choice={boardChoice} action={base} definitions={secretDefinitions} busy onSubmit={submit} viewerId="p0" playerLabels={playerLabels} />);
    const target = screen.getByRole('button', { name: /目标 2.*乙方玩家 拥有.*地区 3/ });
    expect(target).toBeDisabled();
    expect(screen.getByRole('button', { name: '正在提交…' })).toBeDisabled();
    fireEvent.click(target);
    fireEvent.click(screen.getByRole('button', { name: '正在提交…' }));
    expect(submit).not.toHaveBeenCalled();
  });

  it('keeps hand-only mulligan candidates free of board target metadata', () => {
    render(<ChoicePanel choice={{ ...testChoice, kind: 'mulligan', options: [{ id: 'hand-a', label: testCard.name, card: testCard }] }} action={base} definitions={new Map()} busy={false} onSubmit={vi.fn()} viewerId="p0" playerLabels={playerLabels} />);
    expect(screen.queryByText(/目标 \d/)).not.toBeInTheDocument();
    expect(screen.queryByText(/甲方玩家 拥有|地区 \d/)).not.toBeInTheDocument();
  });

  it('requires a selected target and only offers decline when explicitly allowed', () => {
    const submit = vi.fn();
    const { rerender } = render(<ChoicePanel choice={testChoice} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '角色乙' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit.mock.calls[0][0]).toMatchObject({ kind: 'choose', choiceId: testChoice.id, selected: ['b'] });
    rerender(<ChoicePanel choice={{ ...testChoice, allowDecline: true }} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '跳过此选择' }));
    expect(submit.mock.calls[1][0]).toMatchObject({ selected: [] });
  });

  it('replaces a single selection directly and cancels repeated clicks without submitting', () => {
    const submit = vi.fn();
    render(<ChoicePanel choice={testChoice} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    const first = screen.getByRole('button', { name: '角色甲' });
    const second = screen.getByRole('button', { name: '角色乙' });
    const confirm = screen.getByRole('button', { name: '确认选择' });
    expect(first).toHaveAccessibleDescription('再次点击已选项可取消；点击其他项可直接替换。');
    fireEvent.click(first);
    expect(first).toHaveAttribute('aria-pressed', 'true');
    expect(within(first).getByText('已选 · 再点取消')).toBeInTheDocument();
    expect(second).toBeEnabled();
    expect(within(second).getByText('换选此项')).toBeInTheDocument();
    fireEvent.click(second);
    expect(first).toHaveAttribute('aria-pressed', 'false');
    expect(second).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('status')).toHaveTextContent('已选 1 / 1');
    expect(confirm).toBeEnabled();
    fireEvent.click(second);
    expect(second).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getByRole('status')).toHaveTextContent('已选 0 / 1');
    expect(confirm).toBeDisabled();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(second);
    fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: testChoice.id, selected: ['b'] });
  });

  it('explains a multi-selection limit and keeps the remaining selection order when replacing one', () => {
    const submit = vi.fn();
    const choice = { ...testChoice, min: 2, max: 2, options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    const first = screen.getByRole('button', { name: '角色甲' });
    const second = screen.getByRole('button', { name: '角色乙' });
    const third = screen.getByRole('button', { name: '角色丙' });
    const confirm = screen.getByRole('button', { name: '确认选择' });
    fireEvent.click(first);
    expect(confirm).toBeDisabled();
    fireEvent.click(second);
    expect(third).toBeDisabled();
    expect(third).toHaveAccessibleDescription('已达选择上限，先取消一项可继续换选。');
    expect(within(third).getByText('已达上限')).toBeInTheDocument();
    fireEvent.click(third);
    expect(third).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(first);
    expect(third).toBeEnabled();
    expect(third).toHaveAccessibleDescription('还可再选 1 项；再次点击已选项可取消。');
    expect(confirm).toBeDisabled();
    fireEvent.click(third);
    expect(screen.getByRole('status')).toHaveTextContent('已选 2 / 2');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: ['b', 'c'] });
  });
  it('allows explicit investigation top/bottom order without losing options', () => {
    const submit = vi.fn();
    render(<ChoicePanel choice={{ ...testChoice, kind: 'investigation' }} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '角色乙上移' }));
    fireEvent.click(screen.getAllByRole('button', { name: '置于底' })[1]);
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit.mock.calls[0][0]).toMatchObject({ top: ['b'], bottom: ['a'] });
  });
  it('constrains damage allocations to the exact shared total', () => {
    const submit = vi.fn();
    render(<ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 3, allowDecline: true }} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' }), { target: { value: '2' } });
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    fireEvent.change(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' }), { target: { value: '7' } });
    expect(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' })).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit.mock.calls[0][0]).toMatchObject({ allocations: { a: 2, b: 1 } });
  });
  it('lets a phone user assign one damage by tapping a target, switch it, and adjust without typing', () => {
    const submit = vi.fn();
    render(<ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 1, options: testChoice.options.map(option => ({ ...option, card: testCard })) }} action={base} definitions={new Map()} busy={false} onSubmit={submit} playerLabels={{ p0: '甲方玩家' }} />);
    expect(screen.getByText('目标 1')).toBeInTheDocument();
    expect(screen.getByText('目标 2')).toBeInTheDocument();
    expect(screen.getAllByText('甲方玩家 拥有 · 防御 1 · 已受伤 0')).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: '给角色乙分配1点伤害' }));
    expect(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' })).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '给角色甲分配1点伤害' }));
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(1);
    expect(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' })).toHaveValue(0);
    fireEvent.click(screen.getByRole('button', { name: '给角色甲减少1点伤害' }));
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '给角色乙增加1点伤害' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit.mock.calls[0][0]).toMatchObject({ allocations: { a: 0, b: 1 } });
  });
});

describe('choice option browsing', () => {
  it('keeps selections outside a name or ID filter and restores focus when selected-only removal hides an option', () => {
    const submit = vi.fn();
    const choice = { ...largeCardChoice, kind: 'mulligan', min: 0, max: 8, allowDecline: true };
    const { container } = render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
    const search = screen.getByRole('searchbox', { name: '检索选项' });
    const result = screen.getByRole('status', { name: '选项筛选结果' });
    fireEvent.click(container.querySelector('[data-choice-option="option-0"]')!);
    fireEvent.click(container.querySelector('[data-choice-option="option-7"]')!);
    fireEvent.change(search, { target: { value: '  jc125  ' } });
    expect(result).toHaveTextContent('显示 1 / 8 项');
    expect(result).toHaveTextContent('另有 1 项已选不在当前结果中');
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
    const first = container.querySelector('[data-choice-option="option-0"]')!;
    expect(first).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已选' }));
    fireEvent.click(first);
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(0);
    expect(search).toHaveFocus();
    expect(screen.getByText('没有符合筛选的选项。已选的 1 项仍会保留。')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    expect(search).toHaveValue('');
    expect(screen.getByRole('checkbox', { name: '只看已选' })).not.toBeChecked();
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(8);
    expect(container.querySelector('[data-choice-option="option-7"]')).toHaveAttribute('aria-pressed', 'true');
    fireEvent.change(search, { target: { value: '候选角色8' } });
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
    expect(result).not.toHaveTextContent('不在当前结果中');
    fireEvent.change(search, { target: { value: '未找到的卡牌' } });
    expect(result).toHaveTextContent('另有 1 项已选不在当前结果中');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: ['option-7'] });
  });

  it('searches only projected names and card IDs while preserving original board target numbers', () => {
    const submit = vi.fn();
    const hiddenOptions = Array.from({ length: 8 }, (_, index) => ({
      ...boardChoice.options[index % 2], id: `concealed-${index}`,
      card: { ...boardChoice.options[index % 2].card, instanceId: `concealed-${index}` },
    }));
    const { container } = render(<ChoicePanel choice={{ ...testChoice, options: hiddenOptions }} action={base} definitions={secretDefinitions} busy={false} onSubmit={submit} viewerId="p0" playerLabels={playerLabels} />);
    const search = screen.getByRole('searchbox', { name: '检索选项' });
    for (const privateQuery of ['私有身份', 'secret-a', '标签中的', 'concealed-7']) {
      fireEvent.change(search, { target: { value: privateQuery } });
      expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(0);
    }
    fireEvent.change(search, { target: { value: '暗藏者' } });
    const eighth = screen.getByRole('button', { name: /目标 8.*乙方玩家 拥有.*地区 3/ });
    expect(eighth).toHaveAttribute('data-choice-option', 'concealed-7');
    expect(container.innerHTML).not.toMatch(/私有身份|私有能力|secret-a|secret-b/);
    fireEvent.click(eighth);
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已选' }));
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
    expect(eighth).toHaveAttribute('data-choice-target-number', '8');
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: testChoice.id, selected: ['concealed-7'] });
  });

  it('filters public non-card labels without matching internal option IDs', () => {
    const submit = vi.fn();
    const choice = { ...testChoice, options: largeCardChoice.options.map((option, index) => ({ id: option.id, label: `公开选项${index + 1}` })) };
    render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    const search = screen.getByRole('searchbox', { name: '检索选项' });
    fireEvent.change(search, { target: { value: 'option-7' } });
    expect(screen.getByRole('status', { name: '选项筛选结果' })).toHaveTextContent('显示 0 / 8 项');
    fireEvent.change(search, { target: { value: '公开选项8' } });
    fireEvent.click(screen.getByRole('button', { name: '公开选项8' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: ['option-7'] });
  });

  it('keeps the full region-return order when only one option matches the filter', () => {
    const submit = vi.fn();
    const choice = { ...largeCardChoice, kind: 'region_return', min: 8, max: 8 };
    const { container } = render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
    fireEvent.change(screen.getByRole('searchbox', { name: '检索选项' }), { target: { value: 'JC132' } });
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
    expect(screen.getByRole('status', { name: '选项筛选结果' })).toHaveTextContent('另有 7 项已选不在当前结果中');
    fireEvent.click(screen.getByRole('button', { name: '原标签8上移' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: ['option-0', 'option-1', 'option-2', 'option-3', 'option-4', 'option-5', 'option-7', 'option-6'] });
  });

  it.each(['damage', 'investigation', 'order'])('keeps %s decisions outside option filtering', kind => {
    render(<ChoicePanel choice={{ ...largeCardChoice, kind, amount: 1 }} action={base} definitions={new Map()} busy={false} onSubmit={vi.fn()} viewerId="p0" />);
    expect(screen.queryByRole('searchbox', { name: '检索选项' })).not.toBeInTheDocument();
    expect(screen.queryByRole('checkbox', { name: '只看已选' })).not.toBeInTheDocument();
  });
});

describe('choice grid keyboard navigation', () => {
  const buttons = (container: HTMLElement) => Array.from(container.querySelectorAll<HTMLButtonElement>('.hg-choice-option'));
  const layout = (cards: HTMLButtonElement[], columns: number) => cards.forEach((card, index) => {
    vi.spyOn(card, 'getBoundingClientRect').mockReturnValue(new DOMRect(index % columns * 220, Math.floor(index / columns) * 360, 200, 340));
  });

  it('moves through rendered rows and the nearest tail card without selecting, reading or submitting', () => {
    const submit = vi.fn(), read = vi.fn();
    const { container } = render(<ChoicePanel choice={largeCardChoice} action={base} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
    const cards = buttons(container);
    layout(cards, 3);
    cards[2].focus();
    fireEvent.keyDown(cards[2], { key: 'ArrowDown' }); expect(cards[5]).toHaveFocus();
    fireEvent.keyDown(cards[5], { key: 'ArrowDown' }); expect(cards[7]).toHaveFocus();
    fireEvent.keyDown(cards[7], { key: 'ArrowDown' }); expect(cards[7]).toHaveFocus();
    fireEvent.keyDown(cards[7], { key: 'ArrowUp' }); expect(cards[4]).toHaveFocus();
    fireEvent.keyDown(cards[4], { key: 'Home' }); expect(cards[0]).toHaveFocus();
    fireEvent.keyDown(cards[0], { key: 'ArrowRight' }); expect(cards[1]).toHaveFocus();
    fireEvent.keyDown(cards[1], { key: 'ArrowLeft' }); expect(cards[0]).toHaveFocus();
    fireEvent.keyDown(cards[0], { key: 'ArrowLeft' }); expect(cards[0]).toHaveFocus();
    fireEvent.keyDown(cards[0], { key: 'End' }); expect(cards[7]).toHaveFocus();
    fireEvent.keyDown(cards[7], { key: 'ArrowRight' }); expect(cards[7]).toHaveFocus();
    cards.forEach(card => expect(card).toHaveAttribute('aria-pressed', 'false'));
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    expect(read).not.toHaveBeenCalled();
    expect(submit).not.toHaveBeenCalled();
  });

  it('uses the filtered layout while retaining original candidate numbers and selected IDs', () => {
    const submit = vi.fn();
    const choice = { ...largeCardChoice, max: 3, options: largeCardChoice.options.map((option, index) => ({
      ...option, card: { ...option.card, name: [0, 3, 7].includes(index) ? '同名角色' : '其他角色' },
    })) };
    const { container } = render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
    fireEvent.change(screen.getByRole('searchbox', { name: '检索选项' }), { target: { value: '同名角色' } });
    const filtered = buttons(container);
    expect(filtered.map(card => card.dataset.choiceCandidateNumber)).toEqual(['1', '4', '8']);
    layout(filtered, 2);
    filtered[1].focus();
    fireEvent.keyDown(filtered[1], { key: 'ArrowDown' }); expect(filtered[2]).toHaveFocus();
    fireEvent.keyDown(filtered[2], { key: 'ArrowUp' }); expect(filtered[0]).toHaveFocus();
    fireEvent.click(filtered[0]);
    fireEvent.click(filtered[2]);
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已选' }));
    const selected = buttons(container);
    layout(selected, 1);
    selected[0].focus();
    fireEvent.keyDown(selected[0], { key: 'ArrowDown' }); expect(selected[1]).toHaveFocus();
    fireEvent.keyDown(selected[1], { key: 'Home' }); expect(selected[0]).toHaveFocus();
    fireEvent.keyDown(selected[0], { key: 'End' }); expect(selected[1]).toHaveFocus();
    expect(selected[1]).toHaveAttribute('data-choice-candidate-number', '8');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, selected: ['option-0', 'option-7'] });
  });

  it('skips disabled candidates at the multi-selection limit and restores them after cancellation', () => {
    const submit = vi.fn();
    const { container } = render(<ChoicePanel choice={{ ...largeCardChoice, max: 2 }} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
    const cards = buttons(container);
    layout(cards, 3);
    fireEvent.click(cards[0]);
    fireEvent.click(cards[7]);
    cards.slice(1, 7).forEach(card => expect(card).toBeDisabled());
    cards[0].focus();
    fireEvent.keyDown(cards[0], { key: 'ArrowRight' }); expect(cards[7]).toHaveFocus();
    fireEvent.keyDown(cards[7], { key: 'ArrowUp' }); expect(cards[0]).toHaveFocus();
    fireEvent.keyDown(cards[0], { key: 'ArrowDown' }); expect(cards[7]).toHaveFocus();
    fireEvent.keyDown(cards[7], { key: 'Home' }); expect(cards[0]).toHaveFocus();
    fireEvent.keyDown(cards[0], { key: 'End' }); expect(cards[7]).toHaveFocus();
    expect(cards[0]).toHaveAttribute('aria-pressed', 'true');
    expect(cards[7]).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(cards[7]);
    fireEvent.keyDown(cards[7], { key: 'ArrowLeft' }); expect(cards[6]).toHaveFocus();
    expect(cards[6]).toHaveAttribute('aria-pressed', 'false');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, selected: ['option-0'] });
  });

  it('leaves activation, modifier shortcuts and reader keys to their native behavior', () => {
    const submit = vi.fn(), read = vi.fn();
    const { container } = render(<ChoicePanel choice={largeCardChoice} action={base} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
    const first = buttons(container)[0];
    first.focus();
    for (const key of ['Enter', ' ', 'Tab']) expect(fireEvent.keyDown(first, { key })).toBe(true);
    expect(fireEvent.keyDown(first, { key: 'ArrowRight', altKey: true })).toBe(true);
    expect(first).toHaveFocus();
    const reader = screen.getByRole('button', { name: '放大阅读候选角色1' });
    reader.focus();
    expect(fireEvent.keyDown(reader, { key: 'ArrowRight' })).toBe(true);
    expect(reader).toHaveFocus();
    expect(first).toHaveAttribute('aria-pressed', 'false');
    expect(read).not.toHaveBeenCalled();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(['damage', 'investigation'])('does not intercept keys in the %s branch', kind => {
    render(<ChoicePanel choice={{ ...testChoice, kind, amount: 3 }} action={base} definitions={new Map()} busy={false} onSubmit={vi.fn()} />);
    const control = kind === 'damage' ? screen.getByRole('spinbutton', { name: '给角色甲分配伤害' }) : screen.getByRole('button', { name: '角色乙上移' });
    control.focus();
    for (const key of ['ArrowUp', 'ArrowDown', 'Home', 'End']) expect(fireEvent.keyDown(control, { key })).toBe(true);
    expect(control).toHaveFocus();
  });
});

describe('batched choice interactions', () => {
  it.each(['分配', '增加'])('counts each rapid damage %s click without exceeding the shared total', verb => {
    const submit = vi.fn();
    render(<StrictMode><ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 3 }} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    const first = screen.getByRole('button', { name: `给角色甲${verb}1点伤害` });
    const second = screen.getByRole('button', { name: `给角色乙${verb}1点伤害` });
    act(() => {
      first.click();
      first.click();
      second.click();
      second.click();
    });
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(2);
    expect(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' })).toHaveValue(1);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: testChoice.id, allocations: { a: 2, b: 1 } });
  });

  it('counts rapid damage decrements down to zero and still accepts an absolute input allocation', () => {
    const submit = vi.fn();
    render(<StrictMode><ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 3 }} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    const first = screen.getByRole('spinbutton', { name: '给角色甲分配伤害' });
    fireEvent.change(first, { target: { value: '3' } });
    const decrease = screen.getByRole('button', { name: '给角色甲减少1点伤害' });
    act(() => { decrease.click(); decrease.click(); decrease.click(); decrease.click(); });
    expect(first).toHaveValue(0);
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    fireEvent.change(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' }), { target: { value: '3' } });
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: testChoice.id, allocations: { a: 0, b: 3 } });
  });

  it('keeps one-damage taps as a target switch even when the last target is repeated', () => {
    const submit = vi.fn();
    render(<StrictMode><ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 1 }} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    const first = screen.getByRole('button', { name: '给角色甲分配1点伤害' });
    const second = screen.getByRole('button', { name: '给角色乙分配1点伤害' });
    act(() => { first.click(); second.click(); first.click(); second.click(); second.click(); });
    expect(screen.getByRole('spinbutton', { name: '给角色甲分配伤害' })).toHaveValue(0);
    expect(screen.getByRole('spinbutton', { name: '给角色乙分配伤害' })).toHaveValue(1);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: testChoice.id, allocations: { a: 0, b: 1 } });
  });

  it.each(['order', 'investigation', 'region_return'])('applies repeated up moves to the latest %s order', kind => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind, min: 3, max: 3, options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    render(<StrictMode><ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    const moveUp = screen.getByRole('button', { name: '角色丙上移' });
    act(() => { moveUp.click(); moveUp.click(); moveUp.click(); });
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, ...(kind === 'region_return' ? { selected: ['c', 'a', 'b'] } : { top: ['c', 'a', 'b'], bottom: [] }) });
  });

  it.each(['order', 'investigation', 'region_return'])('applies repeated down moves to the latest %s order', kind => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind, min: 3, max: 3, options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    render(<StrictMode><ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    const moveDown = screen.getByRole('button', { name: '角色甲下移' });
    act(() => { moveDown.click(); moveDown.click(); moveDown.click(); });
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, ...(kind === 'region_return' ? { selected: ['b', 'c', 'a'] } : { top: ['b', 'c', 'a'], bottom: [] }) });
  });

  it.each(['bottom', 'top'])('preserves every option when several cards move to %s in one batch', destination => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'investigation', options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    render(<StrictMode><ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    if (destination === 'top') for (const button of screen.getAllByRole('button', { name: '置于底' })) fireEvent.click(button);
    const transfer = screen.getAllByRole('button', { name: destination === 'bottom' ? '置于底' : '置于顶' });
    act(() => { transfer[0].click(); transfer[0].click(); transfer[1].click(); });
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id,
      top: destination === 'top' ? ['a', 'b'] : ['c'], bottom: destination === 'bottom' ? ['a', 'b'] : ['c'] });
  });

  it.each(['上移', '下移'])('applies repeated %s inside the bottom pile without crossing its boundary', direction => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'investigation', options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    render(<StrictMode><ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} /></StrictMode>);
    for (const button of screen.getAllByRole('button', { name: '置于底' })) fireEvent.click(button);
    const moveButton = screen.getByRole('button', { name: `${direction === '上移' ? '角色丙' : '角色甲'}${direction}` });
    act(() => { moveButton.click(); moveButton.click(); moveButton.click(); });
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, top: [], bottom: direction === '上移' ? ['c', 'a', 'b'] : ['b', 'c', 'a'] });
  });
});

it('confirms an empty batch search without treating the compulsory shuffle as decline', () => {
  const submit = vi.fn();
  const choice = { ...testChoice, kind: 'batch20_search', min: 0, max: 0, allowDecline: false, options: [] };
  render(<ChoicePanel choice={choice} action={base} definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0" />);
  expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: '不取牌并洗牌' }));
  expect(submit).toHaveBeenCalledExactlyOnceWith({ ...base, choiceId: choice.id, selected: [] });
});
