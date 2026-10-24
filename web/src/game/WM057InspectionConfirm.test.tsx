import { fireEvent, render, screen, cleanup } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testCard, testChoice } from './testFixtures';

// Native import20_broker.rs creates this explicit, non-declinable 0/0
// acknowledgement after a legal inspection of the originally bound target.
// Overseas ABI112, table 97EC1A99E58C, turn 4 reached it through normal UI.
const choice = { ...testChoice, id: 'wm057-inspection-ack',
  kind: 'wm057-inspection-confirm', title: '信息掮客：确认私密检视',
  min: 0, max: 0, allowDecline: false, options: [],
  previewCards: [{ ...testCard, instanceId: 'inspected-hidden', cardId: 'CQ24', name: '卡门', owner: 'p1', controller: 'p1' }],
};
const action = { id: 'confirm-inspection', kind: 'choose', label: '确认选择', choiceId: choice.id };
afterEach(() => { cleanup(); sessionStorage.clear(); });

it('can acknowledge the real 0/0 broker inspection after a remount, without an option or decline', () => {
  const submit = vi.fn();
  const props = { choice, action, definitions: new Map(), busy: false, onSubmit: submit, viewerId: 'p0' };
  const first = render(<ChoicePanel {...props} />);
  expect(screen.getByRole('button', { name: '确认选择' })).toBeEnabled();
  expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
  first.unmount();
  render(<ChoicePanel {...props} />);
  expect(screen.getByRole('heading', { name: '你私密检视的牌' })).toBeInTheDocument();
  expect(screen.queryByRole('heading', { name: '你查看的牌库顶牌' })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, selected: [] });
});

it.each([
  { kind: 'unknown-zero-choice' },
  { min: 1, max: 1 },
  { min: 0, max: 1 },
  { options: [{ id: 'unexpected', label: '需选择的对象' }] },
])('keeps unrelated or nonzero choice contracts closed: %j', change => {
  const submit = vi.fn();
  render(<ChoicePanel choice={{ ...choice, ...change }} action={action} definitions={new Map()} busy={false} onSubmit={submit} />);
  expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  expect(submit).not.toHaveBeenCalled();
});

it.each(['busy', 'missing-action'])('does not submit an acknowledgement while %s', state => {
  const submit = vi.fn();
  render(<ChoicePanel choice={choice} action={state === 'missing-action' ? undefined : action} definitions={new Map()} busy={state === 'busy'} onSubmit={submit} />);
  const button = screen.getByRole('button', { name: state === 'busy' ? '正在提交…' : '确认选择' });
  expect(button).toBeDisabled();
  fireEvent.click(button);
  expect(submit).not.toHaveBeenCalled();
});
