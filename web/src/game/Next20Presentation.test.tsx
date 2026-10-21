import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CardContent, visibleCard } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { Table } from './Table';
import { testCard, testChoice } from './testFixtures';
import { fixtureSocietyCatalog, fixtureSocietyView } from './societyTest.fixture';
import type { Choice } from './types';

const action = { id: 'choose-next', kind: 'choose', label: '选择' };
const payment: Choice = { ...testChoice, kind: 'jc055-time-payment', min: 0, max: 3, allowDecline: true,
  options: [
    { id: 'carrier-a/time/0', label: '载体甲第1枚' },
    { id: 'carrier-a/time/1', label: '载体甲第2枚' },
    { id: 'carrier-b/time/0', label: '载体乙第1枚' },
  ] };
const panel = (choice: Choice, submit = vi.fn(), key?: string) => <ChoicePanel choice={choice} action={action}
  definitions={new Map()} busy={false} onSubmit={submit} viewerId="p0"
  draftStore={key ? { storage: sessionStorage, key } : undefined} />;

describe('next twenty card decisions use server choices', () => {
  it.each([payment, { ...payment, options: [], max: 0 }])('can confirm X=0 without inventing a minimum payment', choice => {
    const submit = vi.fn();
    render(panel(choice, submit));
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '确认支付 0 枚时间标志' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: [] });
  });

  it('restores a partial payment across two carriers and submits their exact marker IDs', () => {
    const submit = vi.fn();
    const key = 'test.next20.time-payment';
    sessionStorage.removeItem(key);
    const first = render(panel(payment, submit, key));
    fireEvent.click(screen.getByRole('button', { name: '载体甲第2枚' }));
    fireEvent.click(screen.getByRole('button', { name: '载体乙第1枚' }));
    first.unmount();
    render(panel({ ...payment }, submit, key));
    expect(screen.getByRole('button', { name: '载体甲第1枚' })).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getByRole('button', { name: '载体甲第2枚' })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('button', { name: '载体乙第1枚' })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: '确认支付 2 枚时间标志' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: payment.id,
      selected: ['carrier-a/time/1', 'carrier-b/time/0'] });
    sessionStorage.removeItem(key);
  });

  it('explicitly confirms no payment even after selecting markers', () => {
    const submit = vi.fn();
    render(panel(payment, submit));
    fireEvent.click(screen.getByRole('button', { name: '载体乙第1枚' }));
    fireEvent.click(screen.getByRole('button', { name: '不移除标志（X=0）' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: payment.id, selected: [] });
  });

  it('completes an empty blue/gray search while still displaying the authorized private preview', () => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'next-blue-gray-search', min: 0, max: 0, allowDecline: false,
      options: [], previewCards: [{ ...testCard, name: '本次查看但不符合搜寻条件的牌' }] };
    render(panel(choice, submit));
    expect(screen.getByText('本次查看但不符合搜寻条件的牌')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '不取牌并洗牌' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: [] });
  });

  it('keeps a mandatory preparation target mandatory', () => {
    render(panel({ ...testChoice, kind: 'jc055-preparation-target', allowDecline: false }));
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '角色乙' }));
    expect(screen.getByRole('button', { name: '确认选择' })).toBeEnabled();
  });

  it('restores and submits the chosen bottom order rather than reverting to selection order', () => {
    const submit = vi.fn();
    const key = 'test.next20.bottom-order';
    sessionStorage.removeItem(key);
    const choice = { ...testChoice, kind: 'bq085-bottom', min: 2, max: 2, allowDecline: false,
      options: [...testChoice.options, { id: 'c', label: '角色丙' }] };
    const first = render(panel(choice, submit, key));
    fireEvent.click(screen.getByRole('button', { name: '角色甲' }));
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '角色丙' }));
    fireEvent.click(screen.getByRole('button', { name: '角色丙上移' }));
    first.unmount();
    const second = render(panel({ ...choice }, submit, key));
    expect(screen.getByText('自上而下排列，最后一张位于牌库最底。')).toBeInTheDocument();
    expect([...second.container.querySelectorAll('[data-choice-order-option]')].map(el => el.getAttribute('data-choice-order-option')))
      .toEqual(['c', 'a']);
    expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: ['c', 'a'] });
    sessionStorage.removeItem(key);
  });

  it.each([true, false])('supports accepting or declining a rumor replacement without choosing its effect locally (%s)', accept => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'rumor', min: 0, max: 1, allowDecline: true };
    render(panel(choice, submit));
    if (accept) fireEvent.click(screen.getByRole('button', { name: '角色乙' }));
    fireEvent.click(screen.getByRole('button', { name: accept ? '确认选择' : '跳过此选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: accept ? ['b'] : [] });
  });
});

describe('next twenty card public state', () => {
  it('shows the server-provided society counters and hand limit, including an empty society slot', () => {
    const view = { ...fixtureSocietyView, societyZones: fixtureSocietyView.societyZones!.map((zone, index) =>
      ({ ...zone, exposure: index === 1 ? 2 : 0, secrecy: index === 0 ? 3 : 0, handLimit: index === 1 ? 4 : 9 })) };
    const { rerender } = render(<Table view={view} catalog={fixtureSocietyCatalog} busy={false} connection="online" onAction={vi.fn()} />);
    const first = within(screen.getByRole('region', { name: '甲的秘社区' }));
    const empty = within(screen.getByRole('region', { name: '乙的秘社区' }));
    expect(first.getByText('隐秘 3')).toBeInTheDocument();
    expect(first.getByText('手牌上限 9')).toBeInTheDocument();
    expect(empty.getByText('暴露 2')).toBeInTheDocument();
    expect(empty.getByText('手牌上限 4')).toBeInTheDocument();
    rerender(<Table view={fixtureSocietyView} catalog={fixtureSocietyCatalog} busy={false} connection="online" onAction={vi.fn()} />);
    expect(screen.queryByText(/^手牌上限/)).not.toBeInTheDocument();
    expect(screen.queryByText(/^暴露 \d/)).not.toBeInTheDocument();
  });

  it('displays current public status and permanent combat markers only on a visible in-play character', () => {
    const card = { ...testCard, region: 0, currentPublic: true, permanentCombatMarkers: 2 };
    const { rerender } = render(<CardContent card={card} viewerId="p0" />);
    expect(screen.getByText('公开', { exact: true })).toBeInTheDocument();
    expect(screen.getByText('永久战斗图标标志 2')).toBeInTheDocument();
    rerender(<CardContent card={{ ...card, kind: 'asset' }} viewerId="p0" />);
    expect(screen.queryByText('公开', { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText(/永久战斗图标标志/)).not.toBeInTheDocument();
    rerender(<CardContent card={{ ...card, region: undefined }} viewerId="p0" />);
    expect(screen.queryByText('公开', { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText(/永久战斗图标标志/)).not.toBeInTheDocument();
  });

  it.each(['p0', 'p1'])('conceals current-only fields after turning face down from viewer %s', viewer => {
    const card = { ...testCard, region: 0, faceDown: true, currentPublic: true, permanentCombatMarkers: 2 };
    expect(visibleCard(card, viewer).currentPublic).toBeUndefined();
    expect(visibleCard(card, viewer).permanentCombatMarkers).toBeUndefined();
    render(<CardContent card={card} viewerId={viewer} />);
    expect(screen.queryByText('公开', { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText(/永久战斗图标标志/)).not.toBeInTheDocument();
  });
});
