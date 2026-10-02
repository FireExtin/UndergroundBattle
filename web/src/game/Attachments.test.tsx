import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Attachment, Card, Catalog, Region, View } from './types';

const opponent = { ...testView.players[0], id: 'p1', seat: 1, name: '乙', team: 1 };
const host: Card = { ...testCard, instanceId: 'host', name: '公开宿主', owner: 'p1', controller: 'p1', region: 0, icons: { investigation: 0, combat: 6, influence: 1 } };
const attachment: Attachment = { instanceId: 'attachment-1', cardId: 'example-attachment', name: '测试附属', kind: 'attachment', hostId: host.instanceId, owner: 'p1', controller: 'p0', region: 0, faceDown: false, exhausted: false, cost: 1, text: '这是附属的公开规则。' };
const catalog: Catalog = { ...testCatalog, cards: [...testCatalog.cards, { id: attachment.cardId!, name: attachment.name, kind: 'attachment', cost: 1, text: attachment.text!, loyalty: [], supported: true }] };
const region: Region = { id: 'r0', index: 0, cardId: 'world-0', name: '地区甲', threshold: 3, points: 3, influence: [0, 0], characters: [host] };
const view: View = { ...testView, status: 'playing', players: [...testView.players, opponent], hand: [], legalActions: [], regions: [region], attachments: [attachment] };

function openAttachment() {
  fireEvent.click(screen.getByRole('button', { name: '查看公开宿主，附属 1 张' }));
  fireEvent.click(screen.getByRole('button', { name: '放大阅读附属测试附属' }));
  return screen.getByRole('dialog', { name: '放大阅读测试附属' });
}

describe('public attachment presentation', () => {
  it('uses only server-authored hosts, including an opponent in another region, and forwards the original play action', () => {
    const submit = vi.fn();
    const handCard: Card = { ...attachment, region: undefined, owner: 'p0', controller: 'p0' };
    const other = { ...testCard, instanceId: 'not-a-target', name: '未列为目标', region: 0 };
    const action = { id: 'attach-on-opponent', kind: 'play', cardId: attachment.instanceId, targetId: host.instanceId, label: '附着于乙的公开宿主' };
    const { container } = render(<Table view={{ ...view, attachments: [], hand: [handCard], legalActions: [action], regions: [{ ...region, characters: [other] }, { ...region, id: 'r1', index: 1, characters: [{ ...host, region: 1 }] }] }} catalog={catalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看测试附属' }));
    expect(container.querySelector('[data-card-instance="host"]')).toHaveAttribute('data-card-targeted', 'true');
    expect(container.querySelector('[data-card-instance="not-a-target"]')).toHaveAttribute('data-card-targeted', 'false');
    fireEvent.click(screen.getByRole('button', { name: '查看公开宿主' }));
    expect(container.querySelector('[data-card-instance="attachment-1"]')).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: action.label }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(action);
  });

  it('counts and reads mounted cards without treating them as region characters or recomputing the host bonus', () => {
    const { container } = render(<Table view={{ ...view, attachments: [attachment, { ...attachment, instanceId: 'attachment-2' }] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    const piece = screen.getByRole('button', { name: '查看公开宿主，附属 2 张' });
    expect(piece).toHaveAttribute('data-attachment-count', '2');
    expect(within(piece).getByLabelText('图标：调查0，战斗6，势力1')).toBeInTheDocument();
    expect(container.querySelectorAll('.hg-region-characters .hg-card')).toHaveLength(1);
    fireEvent.click(piece);
    const attachments = screen.getByRole('region', { name: '此角色的附属' });
    expect(within(attachments).getByText('附属 · 2 张')).toBeInTheDocument();
    expect(within(attachments).getAllByText('乙 拥有 · 甲 操控 · 附着于 乙 的公开宿主 · 地区 1 · 地区甲')).toHaveLength(2);
    fireEvent.click(within(attachments).getAllByRole('button', { name: '放大阅读附属测试附属' })[0]);
    expect(within(screen.getByRole('dialog')).getByText('这是附属的公开规则。')).toBeInTheDocument();
  });

  it('updates an open attachment reader from the latest moved host and keeps the same attachment identity', () => {
    const { rerender } = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
    const dialog = openAttachment();
    expect(within(dialog).getByText(/地区 1 · 地区甲/)).toBeInTheDocument();
    const moved: View = { ...view, version: 2, regions: [{ ...region, characters: [] }, { ...region, id: 'r1', index: 1, name: '地区乙', characters: [{ ...host, region: 1 }] }], attachments: [{ ...attachment, region: 1 }] };
    rerender(<Table view={moved} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(within(dialog).getByText(/地区 2 · 地区乙/)).toBeInTheDocument();
    expect(within(dialog).queryByText(/地区 1 · 地区甲/)).not.toBeInTheDocument();
    expect(dialog).toHaveAttribute('data-reading-card', attachment.instanceId);
  });

  it('closes an open public reader when the attachment returns to another owner’s hidden hand', () => {
    const { rerender } = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
    openAttachment();
    rerender(<Table view={{ ...view, version: 2, attachments: [], regions: [{ ...region, characters: [{ ...host, kind: 'hidden', faceDown: true, name: '暗藏者', cardId: undefined, text: undefined, icons: undefined }] }], players: [testView.players[0], { ...opponent, handCount: opponent.handCount + 1 }] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(screen.queryByText(attachment.name)).not.toBeInTheDocument();
    expect(screen.queryByText(attachment.text!)).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '查看暗藏者' })).not.toHaveAttribute('data-attachment-count');
    expect(screen.queryByRole('region', { name: '此角色的附属' })).not.toBeInTheDocument();
  });

  it('uses the newest own-hand projection after recovery and drops the old host context', () => {
    const ownAttachment = { ...attachment, owner: 'p0', controller: 'p0' };
    const ownView = { ...view, attachments: [ownAttachment] };
    const { rerender } = render(<Table view={ownView} catalog={catalog} busy={false} onAction={vi.fn()} />);
    openAttachment();
    const returned: Card = { ...ownAttachment, region: undefined, effectiveCost: 0 };
    rerender(<Table view={{ ...ownView, version: 2, attachments: [], hand: [returned] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    const dialog = screen.getByRole('dialog', { name: '放大阅读测试附属' });
    expect(within(dialog).getByLabelText('当前费用 0，印刷费用 1')).toBeInTheDocument();
    expect(within(dialog).queryByText(/附着于/)).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '查看公开宿主' })).not.toHaveAttribute('data-attachment-count');
  });

  it('clears the reading selection on a viewer change rather than reusing a face from the previous seat', () => {
    const { rerender } = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
    openAttachment();
    rerender(<Table view={{ ...view, you: 'p1' }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('reads a chooser-only search card from the current choice and closes it when that private projection ends', () => {
    const optionCard = { ...testCard, instanceId: 'private-search-option', name: '本人的检索牌', text: '仅在此选择中提供的牌面。' };
    const pendingChoice = { id: 'search', kind: 'search', title: '选择检索牌', description: '私人检索选择', playerId: 'p0', min: 1, max: 1, options: [{ id: 'search-option', label: optionCard.name, card: optionCard }] };
    const { rerender } = render(<Table view={{ ...view, pendingChoice, legalActions: [{ id: 'choose-search', kind: 'choose', choiceId: 'search', label: '确认检索' }] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '放大阅读本人的检索牌' }));
    expect(within(screen.getByRole('dialog')).getByText(optionCard.text)).toBeInTheDocument();
    rerender(<Table view={{ ...view, version: 2 }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(screen.queryByText(optionCard.text)).not.toBeInTheDocument();
  });

  it('keeps an old room readable without an attachments field', () => {
    render(<Table view={{ ...view, attachments: undefined }} catalog={catalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByRole('button', { name: '查看公开宿主' })).toBeInTheDocument();
    expect(screen.queryByLabelText(/附属 \d+ 张/)).not.toBeInTheDocument();
  });
});
