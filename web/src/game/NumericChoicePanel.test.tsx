import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { CardTile, visibleCard } from './CardTile';
import { testCard, testChoice } from './testFixtures';
import type { Choice, LegalAction } from './types';
const choice: Choice = { ...testChoice, id: 'number-original', kind: 'import20-xg20-number', title: '禁令：选择费用', min: 0, max: 0, options: [] };
const action: LegalAction = { id: 'choose:number-original', kind: 'choose', label: '确认', choiceId: choice.id, amount: 0, amountRange: { min: 0, max: 4294967295 } };
const props = { choice, action, definitions: new Map(), busy: false };
describe('server numeric choice and public ban projection', () => {
  it('requires an explicit value and submits zero to the exact original continuation', () => {
    const onSubmit = vi.fn();render(<ChoicePanel {...props} onSubmit={onSubmit} />);
    expect(screen.getByRole('button', { name: '确认费用' })).toBeDisabled();
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '0' } });
    fireEvent.click(screen.getByRole('button', { name: '确认费用' }));
    expect(onSubmit).toHaveBeenCalledWith({ ...action, selected: [], amount: 0 });
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
  });
  it('supports an integer outside today’s catalog and rejects negative fractional or overflowing input', () => {
    const onSubmit = vi.fn();render(<ChoicePanel {...props} onSubmit={onSubmit} />);
    for (const value of ['', '-1', '1.5', '4294967296']) {
      fireEvent.change(screen.getByRole('spinbutton'), { target: { value } });
      expect(screen.getByRole('button', { name: '确认费用' })).toBeDisabled();
    }
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '4294967295' } });
    fireEvent.click(screen.getByRole('button', { name: '确认费用' }));
    expect(onSubmit).toHaveBeenCalledWith({ ...action, selected: [], amount: 4294967295 });
  });
  it('a different Pending clears the draft; busy or missing server authorization disables submission', () => {
    const onSubmit = vi.fn();const ui=render(<ChoicePanel {...props} onSubmit={onSubmit} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '3' } });
    ui.rerender(<ChoicePanel {...props} busy onSubmit={onSubmit} />);expect(screen.getByRole('button', { name: '确认费用' })).toBeDisabled();
    ui.rerender(<ChoicePanel {...props} choice={{...choice,id:'number-new'}} onSubmit={onSubmit} />);expect(screen.getByRole('spinbutton')).toHaveValue(null);
    ui.rerender(<ChoicePanel {...props} action={undefined} onSubmit={onSubmit} />);fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '3' } });expect(screen.getByRole('button', { name: '确认费用' })).toBeDisabled();expect(onSubmit).not.toHaveBeenCalled();
  });
  it('zero is public on the face-up ban, and the private zones never project the old choice', () => {
    const c={...testCard,cardId:'XG20',name:'禁令',kind:'attachment',faceDown:false,chosenNumber:0};
    const ui=render(<CardTile card={c} viewerId="p0" compact />);expect(screen.getByLabelText('禁令费用 0')).toBeInTheDocument();
    for (const kind of ['asset','sealed']) {const v={...c,kind};expect(visibleCard(v,'p0').chosenNumber).toBeUndefined();ui.rerender(<CardTile card={v} viewerId="p0" />);expect(screen.queryByLabelText('禁令费用 0')).not.toBeInTheDocument();}
    for(const viewerId of ['p0','p2']){const hidden={...c,faceDown:true};expect(visibleCard(hidden,viewerId).chosenNumber).toBeUndefined();ui.rerender(<CardTile card={hidden} viewerId={viewerId} />);expect(screen.queryByLabelText('禁令费用 0')).not.toBeInTheDocument();}
  });
});
