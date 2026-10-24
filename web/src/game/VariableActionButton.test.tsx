import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { VariableActionButton } from './VariableActionButton';
import { CardTile } from './CardTile';
import { testCard } from './testFixtures';
import type { LegalAction } from './types';

const action: LegalAction = { id: 'fire:weapon:target:3', kind: 'activate', cardId: 'weapon', abilityId: 'import20-cq19-fire',
  targetId: 'target', amount: 3, amountRange: { min: 0, max: 3 }, label: '天基武器' };
describe('server authored variable cost', () => {
  it('submits the chosen integer with the exact original source and target', () => {
    const onAction = vi.fn();
    render(<VariableActionButton action={action} label="炮击" onAction={onAction} />);
    expect(screen.getByRole('spinbutton')).toHaveValue(3);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '2' } });
    fireEvent.click(screen.getByRole('button', { name: '炮击 · X=2' }));
    expect(onAction).toHaveBeenCalledWith({ ...action, amount: 2 });
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '0' } });
    expect(screen.getByRole('button')).toBeEnabled();
  });
  it('rejects blank, negative, fractional and over-budget values; refresh adopts the new server range', () => {
    const onAction = vi.fn();
    const ui = render(<VariableActionButton action={action} label="炮击" onAction={onAction} />);
    for (const value of ['', '-1', '1.5', '4']) {
      fireEvent.change(screen.getByRole('spinbutton'), { target: { value } });
      expect(screen.getByRole('button')).toBeDisabled();
    }
    ui.rerender(<VariableActionButton action={{ ...action, id: 'fire:weapon:target:1', amount: 1, amountRange: { min: 0, max: 1 } }} label="炮击" onAction={onAction} />);
    expect(screen.getByRole('spinbutton')).toHaveValue(1);
    expect(screen.getByRole('button')).toBeEnabled();
    ui.rerender(<VariableActionButton action={action} label="炮击" onAction={onAction} disabled />);
    expect(screen.getByRole('spinbutton')).toBeDisabled();
    expect(screen.getByRole('button')).toBeDisabled();
    expect(onAction).not.toHaveBeenCalled();
  });
  it('renders public charge counters without revealing hidden print', () => {
    const c = { ...testCard, owner: 'p0', controller: 'p0', region: 0, chargeMarkers: 2, faceDown: true };
    const ui = render(<CardTile card={c} viewerId="p1" compact />);
    for (const viewerId of ['p1', 'p2', 'p3']) {
      ui.rerender(<CardTile card={c} viewerId={viewerId} compact />);
      expect(screen.getByLabelText('充能标志 2')).toBeInTheDocument();
      expect(screen.queryByText('真实印刷文字')).not.toBeInTheDocument();
    }
    for (const kind of ['asset', 'sealed']) {
      ui.rerender(<CardTile card={{ ...c, kind, faceDown: false, region: undefined }} viewerId="p0" />);
      expect(screen.queryByLabelText('充能标志 2')).not.toBeInTheDocument();
    }
  });
});
