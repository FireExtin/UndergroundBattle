import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testChoice } from './testFixtures';
import { testCard } from './testFixtures';

const base = { id: 'choose-action', kind: 'choose', label: '选择', choiceId: 'choice-one' };
const boardChoice = { ...testChoice, options: [
  { id: 'anonymous-a', label: '标签中的私有身份甲', card: { ...testCard, instanceId: 'anonymous-a', cardId: 'secret-a', owner: 'p1', controller: 'p1', region: 0, faceDown: true, name: '私有身份甲', text: '私有能力甲' } },
  { id: 'anonymous-b', label: '标签中的私有身份乙', card: { ...testCard, instanceId: 'anonymous-b', cardId: 'secret-b', owner: 'p2', controller: 'p0', region: 2, faceDown: true, name: '私有身份乙', text: '私有能力乙' } },
] };
const playerLabels = { p0: '我方玩家', p1: '甲方玩家', p2: '乙方玩家' };
const secretDefinitions = new Map(boardChoice.options.map(option => [option.card.cardId, { id: option.card.cardId, name: option.card.name, kind: 'character', cost: 7, text: option.card.text, permanentIcons: { investigation: 7, combat: 7, influence: 7 } }]));
describe('server-owned decisions', () => {
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

  it('passes only a concealed public identity to a non-owner choice reader', () => {
    const read = vi.fn();
    const { container } = render(<ChoicePanel choice={boardChoice} action={base} definitions={secretDefinitions} busy={false} onSubmit={vi.fn()} onReadCard={read} viewerId="p0" playerLabels={playerLabels} />);
    fireEvent.click(screen.getAllByRole('button', { name: '放大阅读暗藏者' })[1]);
    expect(read).toHaveBeenCalledOnce();
    expect(read.mock.calls[0][0]).toMatchObject({ name: '暗藏者', owner: 'p2', controller: 'p0', kind: 'hidden' });
    expect(read.mock.calls[0][0].cardId).toBeUndefined();
    expect(read.mock.calls[0][0].text).toBeUndefined();
    expect(read.mock.calls[0][0].icons).toBeUndefined();
    expect(container.textContent).not.toMatch(/私有身份|私有能力/);
  });

  it.each([['anonymous-a', 1], ['anonymous-b', 2]] as const)('distinguishes anonymous board targets by public ownership and region and submits %s', (id, number) => {
    const submit = vi.fn();
    const { container } = render(<ChoicePanel choice={boardChoice} action={base} definitions={secretDefinitions} busy={false} onSubmit={submit} viewerId="p0" playerLabels={playerLabels} />);
    const first = screen.getByRole('button', { name: /目标 1.*甲方玩家 拥有.*地区 1/ });
    const second = screen.getByRole('button', { name: /目标 2.*乙方玩家 拥有.*我方玩家 操控.*地区 3/ });
    expect(first).toHaveAttribute('data-choice-target-number', '1');
    expect(second).toHaveAttribute('data-choice-target-number', '2');
    expect(within(first).getByText('暗藏者', { exact: true })).toBeInTheDocument();
    expect(within(second).getByText('暗藏者', { exact: true })).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/私有身份|私有能力/);
    expect(screen.queryByLabelText('图标：调查7，战斗7，势力7')).not.toBeInTheDocument();
    const target = number === 1 ? first : second;
    expect(target).toHaveAttribute('data-choice-option', id);
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
    fireEvent.click(screen.getByRole('button', { name: '+角色乙' }));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit.mock.calls[0][0]).toMatchObject({ kind: 'choose', choiceId: testChoice.id, selected: ['b'] });
    rerender(<ChoicePanel choice={{ ...testChoice, allowDecline: true }} action={base} definitions={new Map()} busy={false} onSubmit={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '跳过此选择' }));
    expect(submit.mock.calls[1][0]).toMatchObject({ selected: [] });
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
