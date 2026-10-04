import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { RoomLobby, Table } from './Table';
import { groupObjectActions } from './objectActions';
import { fixtureSociety, fixtureSocietyCard, fixtureSocietyCatalog as catalog, fixtureSocietyView as view } from './societyTest.fixture';
import type { LegalAction, View } from './types';

const props = { catalog, busy: false, connection: 'online' as const };
const ownCard = () => within(screen.getByRole('region', { name: '甲的秘社区' })).getByRole('button', { name: `查看${fixtureSociety.name}` });
const select = () => fireEvent.click(ownCard());
const openTarget = () => { select(); fireEvent.click(screen.getByRole('button', { name: '合成秘社甲：合成目标能力' })); };

describe('per-seat society UI on synthetic authorized projections', () => {
  it('shows subtitle and starting hand without presenting the internal zero cost or character stats', () => {
    render(<Table {...props} view={view} onAction={vi.fn()} />);
    select(); fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
    const modal = screen.getByRole('dialog');
    expect(modal).toHaveTextContent('合成副标题');
    expect(modal).toHaveTextContent('起手 6 张');
    expect(modal).toHaveTextContent(fixtureSociety.text);
    expect(modal.querySelector('.hg-cost, .hg-icons')).toBeNull();
    expect(modal).not.toHaveTextContent('防御');
    expect(modal).not.toHaveTextContent('白底图标');
    expect(modal).not.toHaveTextContent('独有');
  });
  it('renders every occupied seat including empty zones without treating societies as regions', () => {
    const { container } = render(<Table {...props} view={view} onAction={vi.fn()} />);
    expect(container.querySelectorAll('[data-society-zone]')).toHaveLength(4);
    expect(screen.getAllByText('未选择')).toHaveLength(2);
    expect(container.querySelector('[data-society-zone="society:p2"] .hg-card')).toHaveAttribute('data-card-exhausted', 'true');
    expect(container.querySelectorAll('.hg-region')).toHaveLength(0);
    expect(container.querySelector('[data-society-zone="society:p0"] .hg-cost')).toBeNull();
  });
  it('uses only legalActions for abilities and permits a server-legal ability when the viewer is not first', () => {
    const action: LegalAction = { ...view.legalActions[0], targetId: undefined, sourceZoneId: 'society:p2', label: '服务器允许的秘社能力' };
    const submit = vi.fn(); render(<Table {...props} view={{ ...view, legalActions: [action] }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: action.label })).not.toBeInTheDocument();
    select();
    fireEvent.click(screen.getByRole('button', { name: action.label }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(action);
    expect(screen.queryByRole('button', { name: '合成未决能力' })).not.toBeInTheDocument();
    expect(JSON.parse(JSON.stringify(actionForRoom(view, action)))).toEqual({ kind: 'activate', cardId: fixtureSocietyCard.instanceId, abilityId: 'FIXTURE_TARGET' });
  });
  it('reuses direct targets, keyboard cancel and duplicate confirmation protection', () => {
    const submit = vi.fn(); const { container } = render(<Table {...props} view={view} onAction={submit} />);
    openTarget(); fireEvent.keyDown(document, { key: 'Escape' });
    expect(container.querySelector('[data-targeting-action]')).toBeNull();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '合成秘社甲：合成目标能力' }));
    fireEvent.click(screen.getByRole('button', { name: '前往高亮对象' }));
    expect(document.activeElement).toHaveAttribute('data-card-targeted', 'true');
    fireEvent.click(screen.getByRole('button', { name: '查看合成墓地目标' }));
    const confirm = screen.getByRole('button', { name: `确认 · ${view.legalActions[0].label}` });
    fireEvent.click(confirm); fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(view.legalActions[0]);
  });
  it('reflects exhaustion and reset from current projection and never toggles them locally', () => {
    const submit = vi.fn(); const { container, rerender } = render(<Table {...props} view={view} onAction={submit} />);
    select(); expect(container.querySelector('[data-card-instance="fixture-society-p0"]')).not.toHaveClass('hg-exhausted');
    const exhausted = { ...view, version: 2, societyZones: view.societyZones!.map(zone => zone.playerId === 'p0' ? { ...zone, card: { ...fixtureSocietyCard, exhausted: true } } : zone), legalActions: [] };
    rerender(<Table {...props} view={exhausted} onAction={submit} />);
    expect(container.querySelector('[data-card-instance="fixture-society-p0"]')).toHaveClass('hg-exhausted');
    expect(within(screen.getByRole('region', { name: '甲的秘社区' })).getByText('已横置', { selector: 'small' })).toBeInTheDocument();
    rerender(<Table {...props} view={{ ...view, version: 3 }} onAction={submit} />);
    expect(container.querySelector('[data-card-instance="fixture-society-p0"]')).not.toHaveClass('hg-exhausted');
    expect(submit).not.toHaveBeenCalled();
  });
  it('reads public cards from projection and closes stale readers or drafts after entity removal or seat switch', () => {
    const { rerender, container } = render(<Table {...props} view={view} onAction={vi.fn()} />);
    select(); fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
    expect(screen.getByRole('dialog')).toHaveTextContent(fixtureSociety.text);
    expect(screen.getByRole('dialog')).toHaveTextContent('甲的秘社区 · 不属于地区');
    rerender(<Table {...props} view={{ ...view, version: 2, societyZones: view.societyZones!.map(zone => zone.playerId === 'p0' ? { ...zone, card: null } : zone), legalActions: [] }} onAction={vi.fn()} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    rerender(<Table {...props} view={view} onAction={vi.fn()} />); openTarget();
    rerender(<Table {...props} view={{ ...view, you: 'p1', hand: [] }} onAction={vi.fn()} />);
    expect(container.querySelector('[data-targeting-action]')).toBeNull();
    expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
  });
  it('falls back smoothly for old kernels and does not disclose choices before simultaneous reveal', () => {
    const { container, rerender } = render(<Table {...props} view={{ ...view, societyZones: undefined, legalActions: [] }} onAction={vi.fn()} />);
    expect(screen.getAllByText('未开放')).toHaveLength(4);
    expect(container.querySelector('[data-card-instance="fixture-society-p0"]')).toBeNull();
    const privateDeck = { id: 'private', name: 'private', description: '', societyId: fixtureSociety.id, cards: [], rulesVersion: '', cardPoolVersion: '', engineVersion: '', updatedAt: '' };
    rerender(<Table {...props} view={{ ...view, status: 'lobby', yourDeck: privateDeck, societyZones: view.societyZones!.map(zone => ({ ...zone, card: null })), legalActions: [] }} onAction={vi.fn()} />);
    expect(screen.getAllByText('开局时公开')).toHaveLength(4);
    expect(screen.queryByText(fixtureSociety.name)).not.toBeInTheDocument();
  });
  it('keeps occupied lobby society zones without exposing the viewer’s selected card', () => {
    const { container } = render(<RoomLobby {...props} view={{ ...view, status: 'lobby', societyZones: view.societyZones!.map(zone => ({ ...zone, card: null })), legalActions: [] }} onAction={vi.fn()} />);
    expect(container.querySelectorAll('.hg-lobby-society')).toHaveLength(4);
    expect(screen.getAllByText('开局时同时公开')).toHaveLength(4);
    expect(screen.queryByText(fixtureSociety.name)).not.toBeInTheDocument();
  });
  it('does not split target alternatives by a read-only zone label', () => {
    const variants = [view.legalActions[0], { ...view.legalActions[0], id: 'other-target', targetId: 'other-target', sourceZoneId: 'untrusted-zone-label' }];
    expect(groupObjectActions(variants)).toHaveLength(1);
  });
  it('retains response and private-choice gating while stripping source labels from commands', () => {
    const responding: View = { ...view, serverNowMs: 1000, responseWindow: { id: 'window', stackTopId: 'stack', holderTeam: 0, members: [{ playerId: 'p0', status: 'composing' }], canBegin: false, myIntentId: 'intent' } };
    expect(JSON.parse(JSON.stringify(actionForRoom(responding, view.legalActions[0])))).toEqual({ kind: 'submitResponse', windowId: 'window', intentId: 'intent', action: { kind: 'activate', cardId: fixtureSocietyCard.instanceId, abilityId: 'FIXTURE_TARGET', targetId: 'fixture-grave-p1' } });
    const submit = vi.fn(); render(<Table {...props} view={{ ...responding, waitingChoice: { playerId: 'p1', title: '私有选择', kind: 'mulligan' } }} onAction={submit} />);
    select(); expect(screen.getByRole('button', { name: '合成秘社甲：合成目标能力' })).toBeDisabled();
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });
  it('invalidates a target removed at the same revision without offering a blind confirmation', () => {
    const submit = vi.fn(); const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    openTarget(); fireEvent.click(screen.getByRole('button', { name: '查看合成墓地目标' }));
    rerender(<Table {...props} view={{ ...view, graveyard: [] }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByText(/部分目标当前未显示/)).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });
});
