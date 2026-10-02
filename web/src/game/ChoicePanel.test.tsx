import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testChoice } from './testFixtures';

const base = { id: 'choose-action', kind: 'choose', label: '选择', choiceId: 'choice-one' };
describe('server-owned decisions', () => {
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
});
