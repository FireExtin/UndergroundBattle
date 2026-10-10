import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Lobby } from './Lobby';
import { DECK_LIBRARY_STORAGE_KEY } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';
import { testCatalog } from './testFixtures';

beforeEach(() => localStorage.clear());
afterEach(() => { history.replaceState({}, '', '/'); localStorage.clear(); });
const draftCatalog: DeckCatalog = {
  ...testCatalog,
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  cards: testCatalog.cards.map(card => ({ ...card, supported: true, deckCopyLimit: null })),
};
describe('Chinese invitation lobby', () => {
  it('lets a player choose a 50-card deck and create a four-seat team room', () => {
    const create = vi.fn(); const catalog = { ...testCatalog, decks: [...testCatalog.decks, { ...testCatalog.decks[0], id: 'hunters', name: '公路猎手' }] };
    render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    fireEvent.click(screen.getByRole('button', { name: /公路猎手/ }));
    fireEvent.click(screen.getByRole('button', { name: /四人协作/ }));
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith('队长', 'teams', 'hunters');
    expect(screen.getByText(/并非官方预组/)).toBeInTheDocument();
  });
  it('opens invitation links directly in join mode and submits only a room code', () => {
    history.replaceState({}, '', '/?invite=ROOM-ONLY'); const join = vi.fn();
    render(<Lobby catalog={testCatalog} busy={false} onCreate={vi.fn()} onJoin={join} retry={vi.fn()} />);
    expect(screen.getByRole('textbox', { name: '邀请码' })).toHaveValue('ROOM-ONLY');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '伙伴' } });
    fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
    expect(join).toHaveBeenCalledWith('ROOM-ONLY', '伙伴', 'watchers');
    expect(location.href).not.toContain('token');
  });
  it('does not send a custom draft ID through the preset entry callbacks', () => {
    const create = vi.fn(); const join = vi.fn();
    render(<Lobby catalog={draftCatalog} busy={false} onCreate={create} onJoin={join} retry={vi.fn()} />);
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(screen.getByRole('button', { name: '创建牌桌 →' })).toBeDisabled();
    expect(screen.getByRole('alert')).toHaveTextContent('尚未支持自定义牌组');
    expect(create).not.toHaveBeenCalled(); expect(join).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /调查者.*控制与调查/ }));
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith('队长', 'duel', 'watchers');
  });
  it('routes a selected custom draft as a full object to the create callback', () => {
    const presetCreate = vi.fn(); const customCreate = vi.fn(); const select = vi.fn();
    render(<Lobby catalog={draftCatalog} busy={false} onCreate={presetCreate} onJoin={vi.fn()} retry={vi.fn()} onSelectDraft={select} onCreateDraft={customCreate} />);
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '我的构筑' } });
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    fireEvent.click(screen.getByRole('button', { name: /四人协作/ }));
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(select).toHaveBeenCalledOnce();
    expect(customCreate).toHaveBeenCalledWith('队长', 'teams', expect.objectContaining({ name: '我的构筑', societyId: null, cards: [{ cardId: 'JC125', count: 50 }], cardPoolVersion: draftCatalog.cardPoolVersion }));
    expect(presetCreate).not.toHaveBeenCalled();
  });
  it('keeps draft editing separate from the chosen snapshot and joins through the custom callback', () => {
    history.replaceState({}, '', '/?invite=CUSTOM-ROOM'); const join = vi.fn(); const presetJoin = vi.fn();
    render(<Lobby catalog={draftCatalog} busy={false} onCreate={vi.fn()} onJoin={presetJoin} retry={vi.fn()} onJoinDraft={join} />);
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '伙伴' } });
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    fireEvent.change(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' }), { target: { value: '1' } });
    fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
    expect(join).toHaveBeenCalledWith('CUSTOM-ROOM', '伙伴', expect.objectContaining({ cards: [{ cardId: 'JC125', count: 50 }] }));
    expect(presetJoin).not.toHaveBeenCalled();
  });
});
