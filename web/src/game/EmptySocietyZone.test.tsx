import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { Table } from './Table';
import { fixtureSocietyCatalog as catalog, fixtureSocietyView as view } from './societyTest.fixture';
import type { Card, LegalAction, View } from './types';

const props = { catalog, busy: false, connection: 'online' as const };
const cards = [
  ['JC033', '家族传承'], ['JC034', '心兽之道'], ['JZ29', '锐齿颈环'], ['JC068', '阿格斯IV'],
] as const;

function projection(id: string, name: string): { current: View; attachment: Card; action: LegalAction } {
  const attachment: Card = { instanceId: `held-${id}`, cardId: id, name, kind: 'attachment', owner: 'p0', controller: 'p0', exhausted: false, faceDown: false };
  const action: LegalAction = { id: `attach-${id}`, kind: 'play', cardId: attachment.instanceId, targetId: 'society:p0', abilityId: 'attach', label: `${name}：结附 → 甲的秘社` };
  return { attachment, action, current: { ...view, hand: [attachment], attachments: [],
    societyZones: view.societyZones!.map(zone => ({ ...zone, card: null })), legalActions: [action] } };
}

function startTarget(name: string) {
  fireEvent.click(screen.getByRole('button', { name: `查看${name}` }));
  fireEvent.click(screen.getByRole('button', { name: `${name}：结附` }));
}

describe('attachment targets are society zones independently of optional society cards', () => {
  it.each(cards)('%s targets an empty headquarters, submits once and displays the restored attachment', (id, name) => {
    const { current, attachment, action } = projection(id, name);
    const submit = vi.fn();
    const { container, rerender } = render(<Table {...props} view={current} onAction={submit} />);
    expect(screen.getAllByText('未选择')).toHaveLength(4);
    startTarget(name);
    fireEvent.click(screen.getByRole('button', { name: '前往高亮对象' }));
    const target = screen.getByRole('button', { name: '选择甲的秘社' });
    expect(target).toHaveFocus();
    fireEvent.click(target);
    expect(target).toHaveAttribute('aria-pressed', 'true');
    const confirm = screen.getByRole('button', { name: `确认 · ${action.label}` });
    fireEvent.click(confirm); fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(action);
    expect(JSON.parse(JSON.stringify(actionForRoom(current, action)))).toEqual({
      kind: 'play', cardId: attachment.instanceId, abilityId: 'attach', targetId: 'society:p0',
    });
    const restored = JSON.parse(JSON.stringify({ ...current, version: current.version + 1, hand: [], legalActions: [],
      attachments: [{ ...attachment, instanceId: `live-${id}`, hostId: 'society:p0' }] })) as View;
    rerender(<Table {...props} view={restored} onAction={submit} />);
    const zone = screen.getByRole('region', { name: '甲的秘社区' });
    expect(within(zone).getByText('未选择')).toBeInTheDocument();
    fireEvent.click(within(zone).getByRole('button', { name: `查看${name}` }));
    expect(screen.getByText(/附着于 甲 的秘社/)).toBeInTheDocument();
    expect(container.querySelectorAll('.hg-society-piece')).toHaveLength(0);
    rerender(<Table {...props} view={{ ...restored, version: restored.version + 1, attachments: [] }} onAction={submit} />);
    expect(within(zone).queryByRole('button', { name: `查看${name}` })).not.toBeInTheDocument();
    expect(within(zone).getByText('未选择')).toBeInTheDocument();
  });

  it('offers only server-authorized zones and drops the action when control changes', () => {
    const { current } = projection('JC033', '家族传承');
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={current} onAction={submit} />);
    startTarget('家族传承');
    expect(screen.getByRole('button', { name: '选择甲的秘社' })).toBeInTheDocument();
    for (const name of ['乙', '丙', '丁']) {
      expect(screen.queryByRole('button', { name: `选择${name}的秘社` })).not.toBeInTheDocument();
    }
    fireEvent.click(screen.getByRole('button', { name: '选择甲的秘社' }));
    rerender(<Table {...props} view={{ ...current, you: 'p1', hand: [], legalActions: [] }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '选择甲的秘社' })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it('invalidates a selected zone that is absent in the latest projection', () => {
    const { current } = projection('JC068', '阿格斯IV');
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={current} onAction={submit} />);
    startTarget('阿格斯IV');
    fireEvent.click(screen.getByRole('button', { name: '选择甲的秘社' }));
    rerender(<Table {...props} view={{ ...current, societyZones: current.societyZones!.filter(zone => zone.playerId !== 'p0') }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByText(/部分目标当前未显示/)).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it('does not infer actions or society abilities from an empty headquarters', () => {
    const { current } = projection('JZ29', '锐齿颈环');
    const submit = vi.fn();
    const { container } = render(<Table {...props} view={{ ...current, legalActions: [] }} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看锐齿颈环' }));
    expect(screen.queryByRole('button', { name: '锐齿颈环：结附' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /选择.的秘社/ })).not.toBeInTheDocument();
    expect(container.querySelectorAll('.hg-society-piece')).toHaveLength(0);
    expect(submit).not.toHaveBeenCalled();
  });
});
