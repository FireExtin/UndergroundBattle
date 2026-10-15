import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testCard, testChoice } from './testFixtures';

const action = { id: 'damage-identities', kind: 'choose', label: '确认选择', choiceId: testChoice.id };
const players = { p0: '我方', p1: '甲方', p2: '乙方' };
afterEach(() => { vi.restoreAllMocks(); sessionStorage.clear(); });

describe('damage candidate identities', () => {
  it('distinguishes every control for same-name targets and submits their original option IDs', () => {
    const options = [
      { id: 'same-target-a', label: '末日信徒', card: { ...testCard, cardId: 'JC042', instanceId: 'same-card-a', name: '末日信徒', owner: 'p1', controller: 'p1', region: 0 } },
      { id: 'same-target-b', label: '末日信徒', card: { ...testCard, cardId: 'JC042', instanceId: 'same-card-b', name: '末日信徒', owner: 'p2', controller: 'p0', region: 2, defense: 2, damage: 1, exhausted: true } },
    ];
    const submit = vi.fn(), read = vi.fn(), writes = vi.spyOn(Storage.prototype, 'setItem');
    const store = { storage: sessionStorage, key: 'damage-candidate-identities' };
    const { container } = render(<ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 3, options }} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" playerLabels={players} draftStore={store} />);
    const descriptions = ['目标 1 甲方 拥有 · 地区 1 · 防御 1 · 已受伤 0', '目标 2 乙方 拥有 · 我方 操控 · 地区 3 · 防御 2 · 已受伤 1 · 已横置'];
    const rows = Array.from(container.querySelectorAll<HTMLElement>('[data-choice-damage-option]'));
    rows.forEach((row, index) => {
      for (const name of ['给末日信徒分配1点伤害', '给末日信徒减少1点伤害', '给末日信徒增加1点伤害', '放大阅读末日信徒']) {
        expect(within(row).getByRole('button', { name })).toHaveAccessibleDescription(descriptions[index]);
      }
      expect(within(row).getByRole('spinbutton', { name: '给末日信徒分配伤害' })).toHaveAccessibleDescription(descriptions[index]);
      fireEvent.click(within(row).getByRole('button', { name: '放大阅读末日信徒' }));
      expect(read).toHaveBeenLastCalledWith(options[index].card);
    });
    expect(writes).not.toHaveBeenCalled();
    expect(submit).not.toHaveBeenCalled();
    options.forEach(option => expect(container.textContent).not.toContain(option.id));
    fireEvent.click(within(rows[0]).getByRole('button', { name: '给末日信徒分配1点伤害' }));
    fireEvent.change(within(rows[1]).getByRole('spinbutton', { name: '给末日信徒分配伤害' }), { target: { value: '2' } });
    fireEvent.click(within(rows[1]).getByRole('button', { name: '给末日信徒减少1点伤害' }));
    fireEvent.click(within(rows[0]).getByRole('button', { name: '给末日信徒增加1点伤害' }));
    const saved = sessionStorage.getItem(store.key);
    fireEvent.click(within(rows[1]).getByRole('button', { name: '放大阅读末日信徒' }));
    expect(sessionStorage.getItem(store.key)).toBe(saved);
    expect(writes).toHaveBeenCalledTimes(4);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, allocations: { 'same-target-a': 2, 'same-target-b': 1 } });
  });

  it('describes only public context when a target card is concealed from the viewer', () => {
    const card = { ...testCard, cardId: 'secret-card', name: '私有卡名', text: '私有规则', owner: 'p1', controller: 'p2', region: 1, faceDown: true, defense: 7, damage: 6 };
    const read = vi.fn();
    const { container } = render(<ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 1, options: [{ id: 'hidden-target', label: '私有标签', card }] }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} onReadCard={read} viewerId="p0" playerLabels={players} />);
    const description = '目标 1 甲方 拥有 · 乙方 操控 · 地区 2';
    const reader = screen.getByRole('button', { name: '放大阅读暗藏者' });
    expect(reader).toHaveAccessibleDescription(description);
    expect(screen.getByRole('button', { name: '给暗藏者分配1点伤害' })).toHaveAccessibleDescription(description);
    expect(screen.getByRole('spinbutton', { name: '给暗藏者分配伤害' })).toHaveAccessibleDescription(description);
    expect(container.innerHTML).not.toMatch(/私有|secret-card|防御 7|已受伤 6/);
    fireEvent.click(reader);
    expect(read).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ name: '暗藏者', cardId: undefined, text: undefined, defense: undefined, damage: undefined }));
  });

  it('still gives text-only damage controls a stable target description', () => {
    const { container } = render(<ChoicePanel choice={{ ...testChoice, kind: 'damage', amount: 2 }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} />);
    const rows = Array.from(container.querySelectorAll<HTMLElement>('[data-choice-damage-option]'));
    rows.forEach((row, index) => {
      within(row).getAllByRole('button').forEach(button => expect(button).toHaveAccessibleDescription(`目标 ${index + 1}`));
      expect(within(row).getByRole('spinbutton')).toHaveAccessibleDescription(`目标 ${index + 1}`);
    });
  });
});
