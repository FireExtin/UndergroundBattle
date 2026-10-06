import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CardTile, sameNameTags } from './CardTile';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Attachment, Card, LegalAction, View } from './types';

const piece = (instanceId: string, extra: Partial<Card> = {}): Card => ({ ...testCard, instanceId, ...extra });
const opponent = { ...testView.players[0], id: 'p1', name: '乙', seat: 1, team: 1 };

describe('same-name instance tags', () => {
  it('groups only by public labels and widens colliding suffixes until instances differ', () => {
    const tags = sameNameTags([
      piece('room-a-000123'), piece('room-b-000123'), piece('room-b-000123'),
      piece('solo', { name: '其他角色' }),
      piece('i9', { faceDown: true, owner: 'p1', controller: 'p1' }),
      piece('i10', { name: '另一张暗牌', faceDown: true, owner: 'p1', controller: 'p1' }),
    ]);
    expect(tags.get('room-a-000123')).toBe('…a-000123');
    expect(tags.get('room-b-000123')).toBe('…b-000123');
    expect(tags.has('solo')).toBe(false);
    // Face-down pieces always group as 暗藏者, so concealed print never joins a visible name group.
    expect(tags.get('i9')).toBe('i9');
    expect(tags.get('i10')).toBe('i10');
    expect(sameNameTags([piece('x1'), piece('x1')]).size).toBe(0);
  });

  it('describes owner, differing controller and instance without changing the accessible name', () => {
    const owner = testView.players[0];
    const { rerender } = render(<CardTile card={piece('i1', { controller: 'p1' })} viewerId="p0" owner={owner} controller={opponent} sameNameTag="i1" compact />);
    const tile = screen.getByRole('button', { name: '查看无知路人' });
    expect(tile).toHaveAttribute('aria-description', '甲（你） 拥有，乙 操控；同名对象 #i1');
    expect(tile).toHaveAttribute('data-control-differs', 'true');
    expect(tile).toHaveTextContent('操控 · 乙');
    rerender(<CardTile card={piece('i2')} viewerId="p0" sameNameTag="i2" compact />);
    expect(screen.getByRole('button', { name: '查看无知路人' })).toHaveAttribute('aria-description', '同名对象 #i2');
    expect(screen.getByText('#i2')).toBeInTheDocument();
  });

  it('distinguishes same-name table pieces through target confirmation and never tags by concealed print', () => {
    const submit = vi.fn();
    const funeral = piece('spell-1', { kind: 'spell', cardId: 'XQ49', name: '葬礼' });
    const mine = piece('i101');
    const taken = piece('i102', { controller: 'p1' });
    const concealed = piece('i104', { owner: 'p1', controller: 'p1', faceDown: true });
    const mounted: Attachment[] = [
      { ...piece('i201', { kind: 'attachment', name: '示例附属', owner: 'p1' }), hostId: 'i101' },
      { ...piece('i202', { kind: 'attachment', name: '示例附属' }), hostId: 'i102' },
    ];
    const region = { id: 'r0', index: 0, cardId: 'world', name: '纽约', threshold: 3, points: 3, influence: [0, 0], characters: [mine, taken, concealed] };
    const actions: LegalAction[] = [mine, taken].map(card => ({ id: `funeral-${card.instanceId}`, kind: 'play', cardId: funeral.instanceId, abilityId: 'funeral', targetId: card.instanceId, label: '葬礼：发动 → 无知路人' }));
    const view: View = { ...testView, status: 'playing', players: [...testView.players, opponent], regions: [region], hand: [funeral, piece('i103')], attachments: mounted, legalActions: [...actions, { id: 'pass', kind: 'pass', label: '让过' }] };
    const { container } = render(<Table catalog={testCatalog} busy={false} connection="online" view={view} onAction={submit} />);
    const tile = (id: string) => container.querySelector<HTMLElement>(`.hg-card[data-card-instance="${id}"]`)!;
    const attachment = (id: string) => container.querySelector<HTMLElement>(`.hg-mounted-object[data-attachment-instance="${id}"]`)!;

    expect(screen.getAllByRole('button', { name: '查看无知路人，附属 1 张' })).toHaveLength(2);
    expect(screen.getByRole('button', { name: '查看无知路人' })).toBe(tile('i103'));
    expect(tile('i101')).toHaveTextContent('#i101');
    expect(tile('i102')).toHaveAttribute('aria-description', '甲（你） 拥有，乙 操控；同名对象 #i102');
    expect(tile('i103')).toHaveAttribute('aria-description', '同名对象 #i103');
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toHaveAttribute('aria-description', '乙 拥有');
    expect(container).not.toHaveTextContent('i104');
    expect(screen.getAllByRole('button', { name: '查看附属示例附属' })).toHaveLength(2);
    expect(attachment('i201')).toHaveAttribute('aria-description', '乙 拥有，甲（你） 操控；同名对象 #i201');
    expect(attachment('i202')).toHaveAttribute('aria-description', '甲（你） 拥有；同名对象 #i202');

    fireEvent.click(tile('i101'));
    expect(screen.getByText('甲 拥有 · 地区 1 · 纽约 · 同名对象 #i101')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
    fireEvent.click(tile('i102'));
    expect(screen.getByText('目标：甲的无知路人 #i102 · 地区 1 · 纽约')).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认 · 葬礼：发动 → 无知路人' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[1]);
  });
});
