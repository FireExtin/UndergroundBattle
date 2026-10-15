import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import native74 from './boundedEntryNative74Test.fixture.json';
import { testCard, testChoice } from './testFixtures';
import type { Choice } from './types';

const action = { id: 'choose-candidate', kind: 'choose', label: '确认选择', choiceId: testChoice.id };

describe('stable choice candidate identities', () => {
  it('distinguishes the five same-name Native74 search instances without exposing their internal IDs', () => {
    const view = native74.states.passers.views[0];
    const choice = view.pendingChoice as Choice;
    const read = vi.fn(), submit = vi.fn();
    const { container } = render(<ChoicePanel choice={choice} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId={view.you} />);
    const readers = screen.getAllByRole('button', { name: '放大阅读无知路人' });
    expect(readers).toHaveLength(5);
    readers.forEach((reader, index) => expect(reader).toHaveAccessibleDescription(`候选 ${index + 1}`));
    expect(screen.queryByRole('searchbox', { name: '检索选项' })).not.toBeInTheDocument();
    choice.options.forEach(option => expect(container.textContent).not.toContain(option.id));
    const fifth = screen.getByRole('button', { name: /候选 5.*无知路人/ });
    expect(fifth).toHaveAttribute('data-choice-option', choice.options[4].id);
    fireEvent.click(readers[4]);
    expect(read).toHaveBeenCalledExactlyOnceWith(choice.options[4].card);
    expect(fifth).toHaveAttribute('aria-pressed', 'false');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(fifth);
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: [choice.options[4].id] });
  });

  it('preserves original candidate numbers through name filtering, selected-only browsing and reading', () => {
    const options = Array.from({ length: 8 }, (_, index) => ({
      id: `candidate-${index}`, label: `服务器标签${index + 1}`,
      card: { ...testCard, instanceId: `card-${index}`, name: index === 0 || index === 7 ? '同名角色' : `其他角色${index + 1}` },
    }));
    const read = vi.fn(), submit = vi.fn();
    const { container } = render(<ChoicePanel choice={{ ...testChoice, options }} action={action} definitions={new Map()} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0" />);
    fireEvent.change(screen.getByRole('searchbox', { name: '检索选项' }), { target: { value: '同名角色' } });
    expect([...container.querySelectorAll('[data-choice-candidate-number]')].map(option => option.getAttribute('data-choice-candidate-number'))).toEqual(['1', '8']);
    fireEvent.click(screen.getByRole('button', { name: /候选 8.*同名角色/ }));
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已选' }));
    expect(container.querySelectorAll('[data-choice-option]')).toHaveLength(1);
    const reader = screen.getByRole('button', { name: '放大阅读同名角色' });
    expect(reader).toHaveAccessibleDescription('候选 8');
    fireEvent.click(reader);
    expect(read).toHaveBeenCalledExactlyOnceWith(options[7].card);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    const eighth = screen.getByRole('button', { name: /候选 8.*同名角色/ });
    expect(eighth).toHaveAttribute('data-choice-option', 'candidate-7');
    expect(eighth).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, selected: ['candidate-7'] });
  });

  it.each([false, true])('uses only public reader context for concealed instances with board context %s', onBoard => {
    const options = Array.from({ length: 2 }, (_, index) => ({
      id: `hidden-${index}`, label: `私有标签${index + 1}`,
      card: { ...testCard, instanceId: `hidden-${index}`, cardId: `secret-${index}`, name: `私有身份${index + 1}`, text: `私有能力${index + 1}`, owner: 'p1', controller: 'p2', faceDown: true, region: onBoard ? index : undefined },
    }));
    const read = vi.fn();
    const { container } = render(<ChoicePanel choice={{ ...testChoice, options }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} onReadCard={read} viewerId="p0" playerLabels={{ p1: '甲方', p2: '乙方' }} />);
    const readers = screen.getAllByRole('button', { name: '放大阅读暗藏者' });
    readers.forEach((reader, index) => expect(reader).toHaveAccessibleDescription(onBoard ? new RegExp(`^目标 ${index + 1}\\s*甲方 拥有 · 乙方 操控 · 地区 ${index + 1}$`) : `候选 ${index + 1}`));
    expect(container.innerHTML).not.toMatch(/私有标签|私有身份|私有能力|secret-/);
    fireEvent.click(readers[1]);
    expect(read).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ instanceId: 'hidden-1', cardId: undefined, name: '暗藏者', kind: 'hidden', text: undefined }));
  });

  it('keeps the existing exact reader name and compact presentation for one hand candidate', () => {
    render(<ChoicePanel choice={{ ...testChoice, kind: 'mulligan', options: [{ id: 'only-card', label: testCard.name, card: testCard }] }} action={action} definitions={new Map()} busy={false} onSubmit={vi.fn()} onReadCard={vi.fn()} viewerId="p0" />);
    expect(screen.getByRole('button', { name: `放大阅读${testCard.name}` })).not.toHaveAttribute('aria-describedby');
    expect(screen.queryByText(/候选 \d|目标 \d/)).not.toBeInTheDocument();
  });
});
