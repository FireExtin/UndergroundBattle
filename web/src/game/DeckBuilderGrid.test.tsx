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
    fireEvent.change(screen.getByRole('combobox', { name: '组牌准入' }), { target: { value: 'all' } });
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

  it('keeps the expanded composition in view while settings retain edits behind their disclosure', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    expect(screen.queryByRole('textbox', { name: '牌组名称' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
    expect(screen.getByRole('list', { name: '牌组校验问题' })).toBeInTheDocument();
    const toggle = screen.getByRole('button', { name: '牌组设置与模板' });
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '红黑试组' } });
    fireEvent.click(screen.getByRole('button', { name: '收起牌组设置' }));
    expect(screen.getByLabelText('当前草稿')).toHaveTextContent('红黑试组');
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0]).toMatchObject({ name: '红黑试组', cards: [{ cardId: 'JC002', count: 1 }] });
    fireEvent.click(screen.getByRole('button', { name: '返回大厅组卡' }));
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('红黑试组');
  });

  it('shows same-name violations inside the reader and clears them immediately after correction', () => {
    render(<DeckLibrary catalog={{ ...catalog, cards: [...catalog.cards, { ...catalog.cards[1], id: 'same-name' }] }} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    const add = screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' });
    fireEvent.click(add); fireEvent.click(add); fireEvent.click(add);
    fireEvent.click(screen.getByRole('button', { name: '预览 巨石阵看护人（same-name）' }));
    fireEvent.click(screen.getByRole('button', { name: '预览添加 巨石阵看护人（same-name）' }));
    const preview = screen.getByRole('dialog');
    expect(within(preview).getByRole('list', { name: '此卡构筑提示' })).toHaveTextContent('当前合计 4 张');
    expect(within(preview).getByText('其他编号也有此牌：同名合计 4 张。')).toBeInTheDocument();
    expect(within(preview).getByText('当前牌组共 54 张 · 暂不可开局')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '预览减少 巨石阵看护人（same-name）' }));
    expect(within(preview).queryByRole('list', { name: '此卡构筑提示' })).not.toBeInTheDocument();
    expect(within(preview).getByText('当前牌组共 53 张 · 构筑校验通过')).toBeInTheDocument();
  });

  it('browses the filtered card sequence without adding cards or leaving the active reader', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌排序' }), { target: { value: 'cost-up' } });
    const opener = screen.getByRole('button', { name: '预览 无知路人（JC125）' });
    opener.focus(); fireEvent.click(opener);
    const reader = screen.getByRole('dialog');
    const close = screen.getByRole('button', { name: '关闭卡面预览' });
    expect(close).toHaveFocus();
    expect(screen.getByRole('button', { name: '上一张卡面' })).toBeDisabled();
    reader.scrollTop = 400;
    act(() => { fireEvent.keyDown(close, { key: 'ArrowRight' }); fireEvent.keyDown(close, { key: 'ArrowRight' }); });
    expect(reader).toHaveAccessibleName('卡面预览 巨石阵看护人（JC002）');
    expect(reader.scrollTop).toBe(0);
    expect(screen.getByRole('button', { name: '下一张卡面' })).toBeDisabled();
    expect(close).toHaveFocus();
    fireEvent.keyDown(close, { key: 'ArrowLeft' });
    expect(reader).toHaveAccessibleName('卡面预览 缺图事务（no-scan）');
    expect(screen.getByRole('status')).toHaveTextContent('已加入 0 张');
    expect(screen.getByText('当前牌组共 0 张 · 暂不可开局')).toBeInTheDocument();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面组卡工作台');
    expect(opener).toHaveFocus();
  });

  it('keeps the selected-only preview sequence stable while its last copies are removed', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已加入' }));
    const opener = screen.getByRole('button', { name: '预览 无知路人（JC125）' });
    opener.focus(); fireEvent.click(opener);
    fireEvent.click(screen.getByRole('button', { name: '预览减少 无知路人（JC125）' }));
    fireEvent.click(screen.getByRole('button', { name: '下一张卡面' }));
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面预览 巨石阵看护人（JC002）');
    expect(screen.getByRole('status')).toHaveTextContent('已加入 1 张');
    fireEvent.click(screen.getByRole('button', { name: '上一张卡面' }));
    expect(screen.getByRole('status')).toHaveTextContent('已加入 0 张');
    fireEvent.click(screen.getByRole('button', { name: '预览添加 无知路人（JC125）' }));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(within(options()).getAllByRole('listitem')).toHaveLength(2);
    expect(count('无知路人（JC125）')).toHaveValue(1);
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
  });

  it('keeps keyboard focus inside the reader after a counter or boundary navigation disables itself', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    fireEvent.click(screen.getByRole('button', { name: '预览 无知路人（JC125）' }));
    const subtract = screen.getByRole('button', { name: '预览减少 无知路人（JC125）' });
    subtract.focus(); fireEvent.click(subtract);
    expect(screen.getByRole('button', { name: '预览添加 无知路人（JC125）' })).toHaveFocus();
    const next = screen.getByRole('button', { name: '下一张卡面' });
    next.focus(); fireEvent.click(next); fireEvent.click(next);
    const previous = screen.getByRole('button', { name: '上一张卡面' });
    expect(previous).toHaveFocus();
    expect(next).toBeDisabled();
    fireEvent.keyDown(previous, { key: 'ArrowLeft' });
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面预览 巨石阵看护人（JC002）');
    expect(screen.getByRole('status')).toHaveTextContent('已加入 0 张');
  });

  it('dismisses a removed catalog card and restores the workspace without discarding the draft', () => {
    const { rerender } = render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    const opener = screen.getByRole('button', { name: '预览 巨石阵看护人（JC002）' });
    opener.focus(); fireEvent.click(opener);
    rerender(<DeckLibrary catalog={{ ...catalog, cards: catalog.cards.filter(card => card.id !== 'JC002') }} />);
    expect(screen.getByRole('dialog')).toHaveAccessibleName('卡面组卡工作台');
    expect(screen.getByRole('searchbox', { name: '检索卡牌' })).toHaveFocus();
    expect(count('未知卡牌（JC002）')).toHaveValue(1);
    expect(screen.getByText('目录已更新，刚才预览的卡牌已不在当前目录。草稿内容仍保留。')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '保存牌组' })).toBeEnabled();
  });

  it('combines priority faction shortcuts with sorting and keeps every unavailable cost searchable', () => {
    const extra = [
      { ...catalog.cards[1], id: 'FIXTURE_RED', name: '红色测试牌', color: '红', cost: 3 },
      { ...catalog.cards[1], id: 'FIXTURE_BLUE', name: '蓝色测试牌', color: '蓝', cost: 1 },
      { ...catalog.cards[1], id: 'FIXTURE_GRAY', name: '灰色测试牌', color: '灰', cost: 9, supported: false },
    ];
    render(<DeckLibrary catalog={{ ...catalog, cards: [...catalog.cards, ...extra] }} />);
    const quick = screen.getByRole('group', { name: '派系速选' });
    const red = within(quick).getByRole('button', { name: '红 · 鸣钟教派' });
    fireEvent.click(red);
    expect(screen.getByRole('combobox', { name: '派系' })).toHaveValue('red');
    expect(within(options()).getAllByRole('listitem')).toHaveLength(1);
    expect(red).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(red);
    expect(screen.getByRole('combobox', { name: '派系' })).toHaveValue('all');
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌排序' }), { target: { value: 'cost-down' } });
    expect(within(options()).getAllByRole('listitem')[0]).toHaveAttribute('data-catalog-card', 'FIXTURE_RED');
    fireEvent.click(screen.getByRole('button', { name: '添加 蓝色测试牌（FIXTURE_BLUE）' }));
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌排序' }), { target: { value: 'selected' } });
    expect(within(options()).getAllByRole('listitem')[0]).toHaveAttribute('data-catalog-card', 'FIXTURE_BLUE');
    fireEvent.change(screen.getByRole('combobox', { name: '组牌准入' }), { target: { value: 'unavailable' } });
    fireEvent.change(screen.getByRole('combobox', { name: '印刷费用' }), { target: { value: '9' } });
    fireEvent.click(within(quick).getByRole('button', { name: '灰 · 国家机构' }));
    expect(within(options()).getAllByRole('listitem')).toHaveLength(1);
    expect(screen.getByRole('button', { name: '添加 灰色测试牌（FIXTURE_GRAY）' })).toBeDisabled();
    expect(count('蓝色测试牌（FIXTURE_BLUE）')).toHaveValue(1);
  });

  it('navigates rendered grid rows with arrows and leaves counts unchanged', () => {
    render(<DeckLibrary catalog={catalog} />);
    const previews = Array.from(options().querySelectorAll<HTMLButtonElement>('.hg-library-card-preview'));
    previews.forEach((button, index) => vi.spyOn(button, 'getBoundingClientRect').mockReturnValue(new DOMRect(index % 2 * 180, Math.floor(index / 2) * 300, 160, 260)));
    previews[0].focus();
    fireEvent.keyDown(previews[0], { key: 'ArrowRight' }); expect(previews[1]).toHaveFocus();
    fireEvent.keyDown(previews[1], { key: 'ArrowDown' }); expect(previews[2]).toHaveFocus();
    fireEvent.keyDown(previews[2], { key: 'ArrowUp' }); expect(previews[0]).toHaveFocus();
    fireEvent.keyDown(previews[0], { key: 'End' }); expect(previews[2]).toHaveFocus();
    fireEvent.keyDown(previews[2], { key: 'Home' }); expect(previews[0]).toHaveFocus();
    fireEvent.keyDown(previews[0], { key: 'ArrowLeft' }); expect(previews[0]).toHaveFocus();
    fireEvent.keyDown(previews[0], { key: 'ArrowRight', altKey: true }); expect(previews[0]).toHaveFocus();
    fireEvent.click(screen.getByRole('button', { name: '舒展' }));
    expect(options()).toHaveAttribute('data-density', 'comfortable');
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument();
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('keeps keyboard focus usable when count controls remove their own row', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    fireEvent.click(screen.getByRole('checkbox', { name: '只看已加入' }));
    const subtract = screen.getByRole('button', { name: '目录减少 无知路人（JC125）' });
    subtract.focus(); fireEvent.click(subtract);
    expect(screen.getByRole('button', { name: '预览 巨石阵看护人（JC002）' })).toHaveFocus();
    const remove = screen.getByRole('button', { name: '移除 巨石阵看护人（JC002）' });
    remove.focus(); fireEvent.click(remove);
    expect(screen.getByRole('searchbox', { name: '检索卡牌' })).toHaveFocus();
    expect(within(options()).queryByRole('listitem')).not.toBeInTheDocument();
  });

  it('keeps collapsed filter conditions visible and starts new results at the top', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.change(screen.getByRole('combobox', { name: '派系' }), { target: { value: 'yellow' } });
    fireEvent.change(screen.getByRole('combobox', { name: '印刷费用' }), { target: { value: '2' } });
    fireEvent.click(screen.getByRole('button', { name: '展开组卡工作台' }));
    expect(screen.queryByRole('combobox', { name: '印刷费用' })).not.toBeInTheDocument();
    expect(screen.getByLabelText('生效的更多筛选')).toHaveTextContent('帷幕守望 · 2 费');
    fireEvent.click(screen.getByRole('button', { name: '更多筛选 · 2 项' }));
    expect(screen.getByRole('combobox', { name: '印刷费用' })).toHaveValue('2');
    options().scrollTop = 500;
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    expect(options().scrollTop).toBe(0);
    options().scrollTop = 200;
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    expect(options().scrollTop).toBe(200);
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌排序' }), { target: { value: 'cost-down' } });
    expect(options().scrollTop).toBe(0);
  });

  it('preserves keyboard focus when removing a retained card that no longer has a catalog preview', () => {
    const { rerender } = render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 无知路人（JC125）' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    rerender(<DeckLibrary catalog={{ ...catalog, cards: catalog.cards.filter(card => card.id !== 'JC125') }} />);
    const removeFirst = screen.getByRole('button', { name: '移除 未知卡牌（JC125）' });
    removeFirst.focus(); fireEvent.click(removeFirst);
    expect(screen.getByRole('button', { name: '预览已选 巨石阵看护人（JC002）' })).toHaveFocus();
    rerender(<DeckLibrary catalog={{ ...catalog, cards: catalog.cards.filter(card => card.id !== 'JC002') }} />);
    const removeLast = screen.getByRole('button', { name: '移除 未知卡牌（JC002）' });
    removeLast.focus(); fireEvent.click(removeLast);
    expect(screen.getByRole('searchbox', { name: '检索卡牌' })).toHaveFocus();
  });

  it('does not discard current edits when the selected saved deck is clicked again', () => {
    render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '复制为草稿' }));
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '未保存的牌组名' } });
    const saved = within(screen.getByRole('list', { name: '已保存牌组' })).getByRole('button', { pressed: true });
    act(() => { fireEvent.click(saved); fireEvent.click(saved); });
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('未保存的牌组名');
    expect(saved).toHaveTextContent('正在编辑未保存修改');
    expect(readDeckLibrary().drafts[0].cards).not.toContainEqual({ cardId: 'JC002', count: 1 });
  });

  it('explains filter values removed by a catalog refresh instead of displaying an unfiltered select', () => {
    const extra = { ...catalog.cards[1], id: 'FIXTURE_NINE', name: '九费测试附属', kind: 'attachment', cost: 9 };
    const { rerender } = render(<DeckLibrary catalog={{ ...catalog, cards: [...catalog.cards, extra] }} />);
    fireEvent.change(screen.getByRole('combobox', { name: '卡牌类型' }), { target: { value: 'attachment' } });
    fireEvent.change(screen.getByRole('combobox', { name: '印刷费用' }), { target: { value: '9' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 九费测试附属（FIXTURE_NINE）' }));
    rerender(<DeckLibrary catalog={catalog} />);
    expect(screen.getByRole('combobox', { name: '卡牌类型' })).toHaveDisplayValue('附属（当前目录无此类型）');
    expect(screen.getByRole('combobox', { name: '印刷费用' })).toHaveDisplayValue('9 费（当前目录无此费用）');
    expect(within(options()).queryByRole('listitem')).not.toBeInTheDocument();
    expect(count('未知卡牌（FIXTURE_NINE）')).toHaveValue(1);
    fireEvent.click(screen.getByRole('button', { name: '重置筛选' }));
    expect(within(options()).getAllByRole('listitem')).toHaveLength(3);
  });

  it('requires an explicit new template choice after the old preset leaves the catalog', () => {
    const { rerender } = render(<DeckLibrary catalog={catalog} />);
    fireEvent.click(screen.getByRole('button', { name: '添加 巨石阵看护人（JC002）' }));
    rerender(<DeckLibrary catalog={{ ...catalog, decks: [{ ...catalog.decks[0], id: 'new-preset', name: '新模板' }] }} />);
    expect(screen.getByRole('combobox', { name: '预组模板' })).toHaveDisplayValue('原模板已不在当前目录');
    expect(screen.getByRole('button', { name: '复制为草稿' })).toBeDisabled();
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
    fireEvent.change(screen.getByRole('combobox', { name: '预组模板' }), { target: { value: 'new-preset' } });
    expect(screen.getByRole('button', { name: '复制为草稿' })).toBeEnabled();
    expect(count('巨石阵看护人（JC002）')).toHaveValue(1);
  });
});
