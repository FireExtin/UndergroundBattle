import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DeckLibrary } from './DeckLibraryPanel';
import { DECK_LIBRARY_STORAGE_KEY, readDeckLibrary } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';

const catalog: DeckCatalog = {
  rulesVersion: 'rules', cardPoolVersion: 'pool', engineVersion: 'engine',
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  decks: [{ id: 'preset', name: '起步预组', description: '', cardCount: 50, cards: [{ cardId: 'JC125', count: 50 }] }],
  cards: [
    { id: 'JC125', name: '无知路人', kind: 'character', cost: 0, text: '真实目录文字', color: '中立', supported: true, deckCopyLimit: null },
    { id: 'JC002', name: '巨石阵看护人', kind: 'character', cost: 2, text: '检视', color: '黄', supported: true, deckCopyLimit: 3 },
    { id: 'no-scan', name: '缺图事务', kind: 'spell', cost: 0, text: '没有登记原图', color: '绿', supported: true, deckCopyLimit: 3 },
    { id: 'JZ50', name: '墓穴食尸鬼', kind: 'character', cost: 2, text: '原图存在但不可加入', color: '黑', supported: false, deckCopyLimit: 3 },
    { id: 'world', name: '地区', kind: 'region', cost: 0, text: '世界牌', supported: true },
  ],
};
beforeEach(() => { localStorage.clear(); });
const count = (name: string) => screen.getByRole('spinbutton', { name: `${name}张数` });
const options = () => screen.getByRole('list', { name: '卡面目录' });

describe('card-face deck workspace', () => {
  it('uses registered originals and replaces a failed image with an explicit placeholder', () => {
    render(<DeckLibrary catalog={catalog} />);
    const original = within(options()).getByRole('img', { name: '无知路人原始牌面' });
    expect(original).toHaveAttribute('src', '/cards/JC125.jpg');
    const missing = screen.getByRole('button', { name: '预览 缺图事务（no-scan）' });
    expect(missing).toHaveTextContent('暂无原始牌面');
    expect(within(missing).queryByRole('img')).not.toBeInTheDocument();
    fireEvent.error(original);
    expect(screen.getByRole('button', { name: '预览 无知路人（JC125）' })).toHaveTextContent('暂无原始牌面');
    expect(within(options()).queryByRole('img', { name: '无知路人原始牌面' })).not.toBeInTheDocument();
  });

  it('links repeated grid, preview and deck-list clicks without losing batched additions', () => {
    render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
    const add = screen.getByRole('button', { name: '添加 无知路人（JC125）' });
    act(() => { for (let i = 0; i < 6; i++) fireEvent.click(add); });
    expect(count('无知路人（JC125）')).toHaveValue(6);
    expect(add.closest('li')).toHaveAttribute('data-selected-count', '6');
    const opener = screen.getByRole('button', { name: '预览 无知路人（JC125）' });
    opener.focus(); fireEvent.click(opener);
    const preview = screen.getByRole('dialog', { name: '卡面预览 无知路人（JC125）' });
    expect(within(preview).getByRole('img')).toHaveAttribute('src', '/cards/JC125.jpg');
    const previewAdd = within(preview).getByRole('button', { name: '预览添加 无知路人（JC125）' });
    act(() => { for (let i = 0; i < 4; i++) fireEvent.click(previewAdd); });
    expect(within(preview).getByRole('status')).toHaveTextContent('已加入 10 张');
    fireEvent.click(within(preview).getByRole('button', { name: '预览减少 无知路人（JC125）' }));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(opener).toHaveFocus();
    expect(count('无知路人（JC125）')).toHaveValue(9);
    fireEvent.click(screen.getByRole('button', { name: '减少 无知路人（JC125）' }));
    expect(add.closest('li')).toHaveAttribute('data-selected-count', '8');
    fireEvent.click(screen.getByRole('button', { name: '移除 无知路人（JC125）' }));
    expect(add.closest('li')).toHaveAttribute('data-selected-count', '0');
    expect(screen.getByRole('button', { name: '目录减少 无知路人（JC125）' })).toBeDisabled();
  });

  it('combines zero-cost, type, faction, text and selected filters and resets an empty result', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 缺图事务（no-scan）' }));
    fireEvent.change(screen.getByRole('combobox', { name: '印刷费用' }), { target: { value: '0' } });
    fireEvent.change(screen.getByRole('combobox', { name: '派系' }), { target: { value: 'green' } });
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌类型' }), { target: { value: 'spell' } });
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: '没有登记' } });
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已加入' }));
    expect(within(options()).getAllByRole('listitem')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: '目录减少 缺图事务（no-scan）' }));
    expect(screen.getByText(/没有符合筛选/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    expect(within(options()).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.getByRole('checkbox', { name: '只看已加入' })).not.toBeChecked();
  });

  it('returns keyboard focus to search when preview removal filters out its opener', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已加入' }));
    const opener = screen.getByRole('button', { name: '预览 巨石阵看护人（JC002）' });
    opener.focus(); fireEvent.click(opener);
    fireEvent.click(screen.getByRole('button', { name: '预览减少 巨石阵看护人（JC002）' }));
    expect(screen.getByRole('status')).toHaveTextContent('已加入 0 张');
    expect(screen.getByRole('button', { name: '预览减少 巨石阵看护人（JC002）' })).toBeDisabled();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.getByRole('searchbox', { name: '检索卡牌' })).toHaveFocus();
    expect(within(options()).queryByRole('listitem')).not.toBeInTheDocument();
  });

  it('keeps admission catalog-driven even when an unavailable card has a real original', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.change(screen.getByRole('combobox', { name: '实现状态' }), { target: { value: 'all' } });
    expect(within(options()).getByRole('img', { name: '墓穴食尸鬼原始牌面' })).toHaveAttribute('src', '/cards/JZ50.jpg');
    expect(screen.getByRole('button', { name: '添加 墓穴食尸鬼（JZ50）' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '预览 墓穴食尸鬼（JZ50）' }));
    const preview = screen.getByRole('dialog');
    expect(within(preview).getByRole('button', { name: '预览添加 墓穴食尸鬼（JZ50）' })).toBeDisabled();
    fireEvent.click(within(preview).getByRole('button', { name: '关闭卡面预览' }));
    expect(screen.getByRole('button', { name: '添加 地区（world）' })).toBeDisabled();
  });

  it('reuses same-name limits and updates invalid/valid feedback across grid and preview', () => {
    const duplicate = { ...catalog.cards[1], id: 'same-name' };
    render(<DeckLibrary catalog={{ ...catalog, cards: [...catalog.cards, duplicate] }} onSelectDraft={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    const add = screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' });
    fireEvent.click(add); fireEvent.click(add);
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（same-name）' }));
    fireEvent.click(screen.getByRole('button', { name: '预览 巨石阵看护人（same-name）' }));
    fireEvent.click(screen.getByRole('button', { name: '预览添加 巨石阵看护人（same-name）' }));
    fireEvent.click(screen.getByRole('button', { name: '关闭卡面预览' }));
    expect(within(screen.getByRole('list', { name: '牌组校验问题' })).getByText(/同名最多 3 张.*合计 4 张/)).toBeInTheDocument();
    expect(add.closest('li')).toHaveAttribute('data-card-issue', 'copies');
    expect(screen.getByRole('button', { name: '添加 巨石阵看护人（same-name）' }).closest('li')).toHaveAttribute('data-card-issue', 'copies');
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '目录减少 巨石阵看护人（same-name）' }));
    expect(screen.queryByRole('list', { name: '牌组校验问题' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
  });

  it('preserves unsaved edits and filters through nested preview and workspace close/reopen', () => {
    document.body.style.overflow = 'auto';
    const result = render(<><button>大厅按钮</button><DeckLibrary catalog={catalog} /></>);
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    expect(document.body.style.overflow).toBe('hidden');
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面组卡工作台');
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: 'JC125' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    const opener = screen.getByRole('button', { name: '预览 无知路人（JC125）' });
    opener.focus(); fireEvent.click(opener);
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面预览 无知路人（JC125）');
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面组卡工作台');
    expect(opener).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(document.body.style.overflow).toBe('auto');
    expect(screen.getByRole('button', { name: '展开组卡工作台' })).toHaveFocus();
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    expect(screen.getByRole('searchbox')).toHaveValue('JC125');
    expect(count('无知路人（JC125）')).toHaveValue(1);
    result.unmount(); expect(document.body.style.overflow).toBe('auto');
    document.body.style.overflow = '';
  });

  it('saves and restores grid edits and delivers an immutable selected public snapshot', () => {
    const select = vi.fn();
    const first = render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '卡面牌组' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    const selected = select.mock.calls[0][0];
    expect(selected.cards).toContainEqual({ cardId: 'JC002', count: 1 });
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    expect(selected.cards).toContainEqual({ cardId: 'JC002', count: 1 });
    first.unmount(); render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('卡面牌组');
    expect(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }).closest('li')).toHaveAttribute('data-selected-count', '1');
    expect(readDeckLibrary().drafts[0].cards).toContainEqual({ cardId: 'JC002', count: 1 });
  });
});
