import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Lobby } from './Lobby';
import { readDeckLibrary, removeDeckDraft } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';
import { saveLobbySelection } from './deckWorkspace';

const catalog: DeckCatalog = {
  rulesVersion: 'rules', cardPoolVersion: 'pool', engineVersion: 'engine',
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  decks: [{ id: 'preset', name: '起步预组', description: '本地用例', cardCount: 50, cards: [{ cardId: 'JC125', count: 50 }] }],
  cards: [
    { id: 'JC125', name: '无知路人', kind: 'character', color: '中立', cost: 0, text: '', supported: true, deckCopyLimit: null },
    { id: 'WM056', name: '赃物贩子', kind: 'character', color: '黑', cost: 1, text: '相邻地区', subtypes: ['人类', '罪犯'], supported: true, deckCopyLimit: 3 },
    { id: 'red', name: '红方人物', kind: 'character', color: '红', cost: 0, text: '目录原文', subtypes: ['人类'], keywords: ['公开'], magic: '鲜血', supported: true, deckCopyLimit: 3 },
  ],
};
const nav = (name: string) => fireEvent.click(within(screen.getByRole('navigation', { name: '大厅导航' })).getByRole('button', { name, exact: true }));
const settings = () => fireEvent.click(screen.getByRole('button', { name: /牌组设置与模板/ }));
const mount = (create = vi.fn()) => render(<Lobby catalog={catalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} onCreateDraft={create} retry={vi.fn()} />);
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); history.replaceState({}, '', '/'); });
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); history.replaceState({}, '', '/'); });

describe('separate collection and invitation pages', () => {
  it('keeps all card content off the homepage and preserves the editor across page navigation', () => {
    const ui = mount();
    expect(ui.container.querySelector('[data-catalog-card]')).toBeNull();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(screen.queryByRole('textbox', { name: '你的称呼' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /卡牌收藏与组牌/ }));
    expect(location.hash).toBe('#collection');
    fireEvent.click(screen.getByRole('button', { name: '添加 赃物贩子（WM056）' }));
    nav('首页');
    expect(ui.container.querySelector('[data-catalog-card]')).toBeNull();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    nav('卡牌收藏');
    expect(screen.getByRole('spinbutton', { name: '赃物贩子（WM056）张数' })).toHaveValue(1);
  });

  it('combines real zero-cost, faction, type and feature fields without clearing selected cards', () => {
    mount(); nav('卡牌收藏');
    fireEvent.change(screen.getByRole('combobox', { name: '派系' }), { target: { value: 'red' } });
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌类型' }), { target: { value: 'character' } });
    fireEvent.change(screen.getByRole('combobox', { name: '印刷费用' }), { target: { value: '0' } });
    fireEvent.change(screen.getByRole('combobox', { name: '特性' }), { target: { value: 'keyword:公开' } });
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: '鲜血' } });
    const grid = screen.getByRole('list', { name: '卡面目录' });
    expect(within(grid).getAllByRole('listitem')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: '添加 红方人物（red）' }));
    for (const feature of ['subtype:人类', 'magic:鲜血']) {
      fireEvent.change(screen.getByRole('combobox', { name: '特性' }), { target: { value: feature } });
      expect(within(grid).getAllByRole('listitem')).toHaveLength(1);
    }
    fireEvent.change(screen.getByRole('combobox', { name: '派系' }), { target: { value: 'black' } });
    expect(within(grid).queryByRole('listitem')).not.toBeInTheDocument();
    expect(screen.getByRole('spinbutton', { name: '红方人物（red）张数' })).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    expect(within(grid).getAllByRole('listitem')).toHaveLength(3);
  });

  it('recovers unsaved edits and filters on reload without silently saving the library', () => {
    let ui = mount(); nav('卡牌收藏'); settings();
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '未保存构筑' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 赃物贩子（WM056）' }));
    fireEvent.change(screen.getByRole('combobox', { name: '特性' }), { target: { value: 'subtype:人类' } });
    expect(readDeckLibrary().drafts).toHaveLength(0);
    ui.unmount(); ui = mount(); settings();
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('未保存构筑');
    expect(screen.getByRole('spinbutton', { name: '赃物贩子（WM056）张数' })).toHaveValue(1);
    expect(screen.getByRole('combobox', { name: '特性' })).toHaveValue('subtype:人类');
    expect(readDeckLibrary().drafts).toHaveLength(0);
    expect(screen.getByText('已恢复当前标签尚未保存的编辑。')).toBeInTheDocument();
  });

  it('restores the explicitly chosen snapshot separately from later unsaved editing', () => {
    const create = vi.fn(); let ui = mount(create); nav('卡牌收藏'); settings();
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    nav('卡牌收藏');
    fireEvent.change(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' }), { target: { value: '1' } });
    ui.unmount(); ui = mount(create); nav('入席');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '玩家' } });
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith('玩家', 'duel', expect.objectContaining({ cards: [{ cardId: 'JC125', count: 50 }] }));
    nav('卡牌收藏');
    expect(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' })).toHaveValue(1);
  });

  it('does not resurrect a draft deleted in another tab when restoring the workspace', () => {
    let ui = mount(); nav('卡牌收藏'); settings();
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    const oldId = readDeckLibrary().drafts[0].id;
    fireEvent.change(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' }), { target: { value: '49' } });
    removeDeckDraft(oldId);
    ui.unmount(); ui = mount();
    expect(screen.getByText(/此草稿已被删除/)).toBeInTheDocument();
    expect(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' })).toHaveValue(49);
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts).toHaveLength(0);
    fireEvent.click(screen.getByRole('button', { name: '另存为新草稿' }));
    expect(readDeckLibrary().drafts).toHaveLength(1);
    expect(readDeckLibrary().drafts[0].id).not.toBe(oldId);
  });

  it('tolerates corrupt tab recovery and reports blocked autosave while explicit library save still works', () => {
    sessionStorage.setItem('hegemony.deckWorkspace.v1', '{broken');
    const original = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      if (this === sessionStorage) throw new Error('tab storage unavailable');
      return original.call(this, key, value);
    });
    mount(); nav('卡牌收藏');
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    expect(screen.getByText(/当前编辑进度无法自动恢复/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0].cards).toEqual([{ cardId: 'JC125', count: 1 }]);
  });

  it('blocks a restored preset missing from the current catalog until a real preset is chosen', () => {
    saveLobbySelection({ kind: 'preset', deckId: 'retired' });
    mount(); nav('入席');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '玩家' } });
    expect(screen.getByRole('alert')).toHaveTextContent('预组已不在当前目录');
    expect(screen.getByRole('button', { name: '创建牌桌 →' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: /起步预组/ }));
    expect(screen.getByRole('button', { name: '创建牌桌 →' })).toBeEnabled();
  });
});
