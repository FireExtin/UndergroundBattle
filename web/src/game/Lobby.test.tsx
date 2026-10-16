import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Lobby } from './Lobby';
import { createDeckDraft, DECK_LIBRARY_STORAGE_KEY, saveDeckLibrary } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';
import { testCatalog } from './testFixtures';

beforeEach(() => localStorage.clear());
afterEach(() => { history.replaceState({}, '', '/'); localStorage.clear(); });
const draftCatalog: DeckCatalog = {
  ...testCatalog,
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  cards: testCatalog.cards.map(card => ({ ...card, supported: true, deckCopyLimit: null })),
};
const openPage = (name: string) => fireEvent.click(within(screen.getByRole('navigation', { name: '大厅导航' })).getByRole('button', { name }));
const openEditor = () => { openPage('卡牌收藏'); fireEvent.click(screen.getByRole('button', { name: /牌组设置与模板/ })); };
describe('Chinese invitation lobby', () => {
  it('keeps every invitation URL join-only even with a home hash or an empty code', () => {
    history.replaceState({}, '', '/?invite=TARGET#home');
    const create = vi.fn(); const join = vi.fn();
    const app = render(<Lobby catalog={testCatalog} busy={false} onCreate={create} onJoin={join} retry={vi.fn()} />);
    expect(screen.queryByRole('button', { name: /创建牌桌/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: '对战模式' })).not.toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: '邀请码' })).toHaveAttribute('readonly');
    openPage('卡牌收藏'); openPage('入席');
    expect(screen.getByRole('textbox', { name: '邀请码' })).toHaveValue('TARGET');
    expect(screen.queryByRole('button', { name: /创建牌桌/ })).not.toBeInTheDocument();
    app.unmount(); history.replaceState({}, '', '/?invite=#home');
    render(<Lobby catalog={testCatalog} busy={false} onCreate={create} onJoin={join} retry={vi.fn()} />);
    expect(screen.getByRole('button', { name: '加入牌桌 →' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: /创建牌桌/ })).not.toBeInTheDocument();
    expect(create).not.toHaveBeenCalled(); expect(join).not.toHaveBeenCalled();
  });
  it('generates and remembers a readable blank nickname and reuses it for repeated submission', () => {
    const create = vi.fn();
    const app = render(<Lobby catalog={testCatalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    openPage('入席');
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '   ' } });
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    const name = create.mock.calls[0][0]; expect(name).toMatch(/^旅人·[A-F0-9]{6}$/);
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create.mock.calls[1][0]).toBe(name);
    app.unmount();
    render(<Lobby catalog={testCatalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    expect(screen.getByLabelText('你的称呼')).toHaveValue(name);
    fireEvent.change(screen.getByLabelText('你的称呼'), { target: { value: '  新称呼  ' } });
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create.mock.calls[2][0]).toBe('新称呼');
  });
  it('offers saved decks directly at entry and submits the selected full snapshot', () => {
    const a = { ...createDeckDraft(draftCatalog, draftCatalog.decks[0]), name: '甲牌组' };
    const b = { ...a, id: 'saved-b', name: '乙牌组', cards: [{ cardId: 'JC125', count: 51 }] };
    saveDeckLibrary([a, b]); const create = vi.fn();
    render(<Lobby catalog={draftCatalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} onCreateDraft={create} retry={vi.fn()} />);
    openPage('入席');
    fireEvent.change(screen.getByRole('combobox', { name: '已保存牌组' }), { target: { value: 'saved:saved-b' } });
    expect(screen.getByLabelText('所选牌组')).toHaveTextContent('乙牌组 · 51 张');
    saveDeckLibrary([{ ...b, cards: [{ cardId: 'JC125', count: 52 }] }]);
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith(expect.any(String), 'duel', expect.objectContaining({ name: '乙牌组', cards: [{ cardId: 'JC125', count: 51 }] }));
  });
  it('shows deck naming and saved decks before opening settings', () => {
    const saved = { ...createDeckDraft(draftCatalog, draftCatalog.decks[0]), name: '看得见的牌组' };
    saveDeckLibrary([saved]);
    render(<Lobby catalog={draftCatalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} retry={vi.fn()} />);
    openPage('卡牌收藏');
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toBeVisible();
    expect(screen.getByRole('list', { name: '已保存牌组' })).toBeVisible();
    expect(screen.getByRole('button', { name: /牌组设置与模板/ })).toHaveAttribute('aria-expanded', 'false');
  });
  it('lets a player choose a 50-card deck and create a four-seat team room', () => {
    const create = vi.fn(); const catalog = { ...testCatalog, decks: [...testCatalog.decks, { ...testCatalog.decks[0], id: 'hunters', name: '公路猎手' }] };
    render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    openPage('入席');
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
    openPage('入席');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    openEditor();
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
    openPage('入席');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    openEditor();
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
    openEditor();
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    openPage('卡牌收藏');
    fireEvent.change(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' }), { target: { value: '1' } });
    openPage('入席');
    fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
    expect(join).toHaveBeenCalledWith('CUSTOM-ROOM', '伙伴', expect.objectContaining({ cards: [{ cardId: 'JC125', count: 50 }] }));
    expect(presetJoin).not.toHaveBeenCalled();
  });
});
