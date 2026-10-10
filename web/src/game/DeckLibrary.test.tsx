import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, DECK_LIBRARY_STORAGE_KEY, readDeckLibrary, removeDeckDraft, saveDeckLibrary } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';

const catalog: DeckCatalog = {
  rulesVersion: 'rules-current', cardPoolVersion: 'pool-current', engineVersion: 'engine-current',
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  decks: [{ id: 'preset', name: '起步预组', description: '已有预组说明', cardCount: 50, cards: [{ cardId: 'JC125', count: 50 }] }],
  cards: [
    { id: 'JC125', name: '无知路人', kind: 'character', cost: 0, text: '已核实的规则文字', color: '中立', supported: true, deckCopyLimit: null },
    { id: 'yellow', name: '黄色角色', kind: 'character', cost: 1, text: '本地区', color: '黄', supported: true, deckCopyLimit: 3 },
    { id: 'green', name: '绿色事务', kind: 'spell', cost: 1, text: '选择目标', color: '绿', supported: true, deckCopyLimit: 3 },
    { id: 'world', name: '世界地区', kind: 'region', cost: 0, text: '地区规则', supported: true, deckCopyLimit: 3 },
    { id: 'pending', name: '未实现卡', kind: 'character', cost: 0, text: '', supported: false, deckCopyLimit: 3 },
  ],
};
beforeEach(() => localStorage.clear());
afterEach(() => localStorage.clear());

describe('local named deck editor', () => {
  it('copies a preset, names and saves it, reloads it, and selects a full public draft', () => {
    const select = vi.fn();
    const first = render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '夜行者' } });
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    const savedId = readDeckLibrary().drafts[0].id;
    expect(readDeckLibrary().drafts[0]).toMatchObject({ name: '夜行者', description: '已有预组说明', societyId: null, cards: [{ cardId: 'JC125', count: 50 }] });
    first.unmount();
    render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('夜行者');
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(select).toHaveBeenCalledWith(expect.objectContaining({ id: savedId, name: '夜行者', engineVersion: 'engine-current', cards: [{ cardId: 'JC125', count: 50 }] }));
  });
  it('saves an incomplete draft while blocking selection, then filters and adjusts cards', () => {
    const select = vi.fn(); render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    fireEvent.change(screen.getByRole('combobox', { name: '派系' }), { target: { value: 'green' } });
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌类型' }), { target: { value: 'spell' } });
    expect(screen.queryByRole('button', { name: /添加 黄色角色/ })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '添加 绿色事务（green）' }));
    expect(screen.getByRole('spinbutton', { name: '绿色事务（green）张数' })).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '增加 绿色事务（green）' }));
    expect(screen.getByRole('spinbutton', { name: '绿色事务（green）张数' })).toHaveValue(2);
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0].cards).toEqual([{ cardId: 'green', count: 2 }]);
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    expect(screen.getByText(/还差 48 张/)).toBeInTheDocument(); expect(select).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '移除 绿色事务（green）' }));
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument();
  });
  it('copies a spell-containing preset, searches transactions, adds one, and selects the corrected deck', () => {
    const select = vi.fn();
    const withSpells = { ...catalog, decks: [{ ...catalog.decks[0], cards: [{ cardId: 'JC125', count: 48 }, { cardId: 'green', count: 2 }] }] };
    render(<DeckLibrary catalog={withSpells} onSelectDraft={select} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌类型' }), { target: { value: 'spell' } });
    expect(screen.getByRole('option', { name: '事务' })).toHaveValue('spell');
    expect(screen.queryByRole('option', { name: '事务（旧目录）' })).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: 'green' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 绿色事务（green）' }));
    fireEvent.change(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' }), { target: { value: '47' } });
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(select).toHaveBeenCalledWith(expect.objectContaining({ cards: expect.arrayContaining([{ cardId: 'JC125', count: 47 }, { cardId: 'green', count: 3 }]) }));
  });
  it('shows copy violations immediately and allows a draft to be saved for later correction', () => {
    render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 黄色角色（yellow）' }));
    fireEvent.change(screen.getByRole('spinbutton', { name: '黄色角色（yellow）张数' }), { target: { value: '4' } });
    expect(within(screen.getByRole('list', { name: '牌组校验问题' })).getByText(/黄色角色.*最多 3 张.*4 张/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0].cards).toContainEqual({ cardId: 'yellow', count: 4 });
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
  });
  it('never enables adding regions or unsupported cards and searches only the supplied catalog', () => {
    render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
    fireEvent.change(screen.getByRole('combobox', { name: '实现状态' }), { target: { value: 'all' } });
    expect(screen.getByRole('button', { name: '添加 世界地区（world）' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '添加 未实现卡（pending）' })).toBeDisabled();
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: 'green' } });
    expect(screen.getByRole('button', { name: '添加 绿色事务（green）' })).toBeEnabled();
    expect(screen.queryByRole('button', { name: /添加 无知路人/ })).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: '未授权的研究卡' } });
    expect(screen.getByText(/没有符合筛选/)).toBeInTheDocument();
  });
  it('keeps obsolete draft provenance until the user explicitly revalidates it', () => {
    const old = { ...createDeckDraft(catalog, catalog.decks[0]), engineVersion: 'old-engine' }; saveDeckLibrary([old]);
    render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    expect(readDeckLibrary().drafts[0].engineVersion).toBe('old-engine');
    fireEvent.click(screen.getByRole('button', { name: '按当前卡池重新校验' }));
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0].engineVersion).toBe('engine-current');
  });
  it('recovers from corrupt storage and reports a failed save without selecting an unsaved draft', () => {
    localStorage.setItem(DECK_LIBRARY_STORAGE_KEY, '{broken');
    const first = render(<DeckLibrary catalog={catalog} />);
    expect(screen.getByRole('alert')).toHaveTextContent('未覆盖原数据');
    expect(localStorage.getItem(DECK_LIBRARY_STORAGE_KEY)).toBe('{broken');
    first.unmount(); const select = vi.fn();
    const setItem = vi.fn(() => { throw new Error('quota'); });
    render(<DeckLibrary catalog={catalog} onSelectDraft={select} storage={{ getItem: () => null, key: () => null, length: 0, setItem }} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(screen.getByRole('alert')).toHaveTextContent('未能保存');
    expect(screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' })).toHaveValue(50);
    expect(select).not.toHaveBeenCalled();
    expect(setItem).toHaveBeenCalledOnce();
  });
  it('keeps the deleted draft save-as action available after a real quota failure and allows retry', () => {
    const original = { ...createDeckDraft(catalog, catalog.decks[0]), name: 'original' }; saveDeckLibrary([original]);
    let quota = false;
    const setItem = vi.fn((key: string, value: string) => { if (quota) throw new Error('quota'); localStorage.setItem(key, value); });
    const storage = { get length() { return localStorage.length; }, key: (i: number) => localStorage.key(i), getItem: (key: string) => localStorage.getItem(key), setItem };
    render(<DeckLibrary catalog={catalog} storage={storage} />);
    expect(removeDeckDraft(original.id)).toBeNull();
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: 'retained edits' } });
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(screen.getByRole('alert')).toHaveTextContent('已在另一页删除');
    quota = true;
    fireEvent.click(screen.getByRole('button', { name: '另存为新草稿' }));
    expect(setItem).toHaveBeenCalledOnce();
    expect(screen.getByRole('alert')).toHaveTextContent('未能保存');
    expect(screen.getByRole('button', { name: '另存为新草稿' })).toBeEnabled();
    expect(readDeckLibrary().drafts).toEqual([]);
    quota = false;
    fireEvent.click(screen.getByRole('button', { name: '另存为新草稿' }));
    expect(setItem).toHaveBeenCalledTimes(2);
    const copy = readDeckLibrary().drafts[0];
    expect(copy.id).not.toBe(original.id); expect(copy.name).toBe('retained edits'); expect(copy.cards).toEqual(original.cards);
    expect(screen.queryByRole('button', { name: '另存为新草稿' })).not.toBeInTheDocument();
  });
  it('retains a saved draft and permits a later deletion when writing its deletion marker fails', () => {
    const original = { ...createDeckDraft(catalog), name: 'preserved' }; saveDeckLibrary([original]);
    const setItem = vi.fn(() => { throw new Error('quota'); });
    const denied = { get length() { return localStorage.length; }, key: (i: number) => localStorage.key(i), getItem: (key: string) => localStorage.getItem(key), setItem };
    expect(removeDeckDraft(original.id, denied)).toContain('未能删除');
    expect(setItem).toHaveBeenCalledOnce(); expect(readDeckLibrary().drafts).toEqual([original]);
    expect(removeDeckDraft(original.id)).toBeNull(); expect(readDeckLibrary().drafts).toEqual([]);
  });
  it('does not certify custom decks when construction metadata is missing', () => {
    const legacy = { ...catalog, deckBuildRules: undefined, cards: catalog.cards.map(card => ({ ...card, deckCopyLimit: undefined })) };
    render(<DeckLibrary catalog={legacy} onSelectDraft={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    expect(screen.getByText(/缺少构筑规则元数据/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '保存牌组' })).toBeEnabled();
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    expect(screen.queryByText(/同名最多 3 张，当前合计 50/)).not.toBeInTheDocument();
  });
});
