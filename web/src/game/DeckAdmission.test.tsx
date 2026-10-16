import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DeckLibrary } from './DeckLibraryPanel';
import { deckCardAdmission } from './deckLibrary';
import type { DeckCatalog } from './deckLibrary';

const colors = ['红', '黑', '蓝', '灰'];
const cards = colors.flatMap((color, index) => [
  { id: `ready-${index}`, name: `${color}已准入`, color, kind: 'character', cost: 1, text: '', supported: true, deckCopyLimit: 3 },
  { id: `pending-${index}`, name: `${color}未实现`, color, kind: 'character', cost: 1, text: '', supported: false, deckCopyLimit: 3 },
  { id: `unknown-${index}`, name: `${color}副本未核实`, color, kind: 'character', cost: 1, text: '', supported: true },
]);
const catalog: DeckCatalog = {
  rulesVersion: 'rules', cardPoolVersion: 'pool', engineVersion: 'engine',
  deckBuildRules: { minimumCards: 1, serviceCardCapacity: 2048, societySupported: false },
  decks: [], cards,
};
beforeEach(() => { localStorage.clear(); });

describe('per-card construction admission across red, black, blue and gray', () => {
  it('defaults to genuinely buildable cards and never admits a whole faction at once', () => {
    const ui = render(<DeckLibrary catalog={catalog} />);
    expect(screen.getByRole('combobox', { name: '组牌准入' })).toHaveValue('implemented');
    expect(ui.container.querySelectorAll('[data-catalog-card]')).toHaveLength(4);
    expect(ui.container.querySelectorAll('[data-card-admission="buildable"]')).toHaveLength(4);
    for (const [index, color] of colors.entries()) {
      expect(screen.getByRole('button', { name: `添加 ${color}已准入（ready-${index}）` })).toBeEnabled();
      expect(screen.queryByRole('button', { name: `添加 ${color}未实现（pending-${index}）` })).not.toBeInTheDocument();
    }
    fireEvent.change(screen.getByRole('combobox', { name: '组牌准入' }), { target: { value: 'all' } });
    expect(ui.container.querySelectorAll('[data-catalog-card]')).toHaveLength(12);
    expect(ui.container.querySelectorAll('[data-card-admission="unavailable"]')).toHaveLength(8);
    for (const [index, color] of colors.entries()) {
      expect(screen.getByRole('button', { name: `添加 ${color}未实现（pending-${index}）` })).toBeDisabled();
      expect(screen.getByRole('button', { name: `添加 ${color}副本未核实（unknown-${index}）` })).toBeDisabled();
      const row = ui.container.querySelector(`[data-catalog-card="pending-${index}"]`)!;
      expect(within(row as HTMLElement).getByText('暂不可用')).toBeInTheDocument();
      expect(row).toHaveTextContent('尚未确认完整实现');
    }
  });

  it('keeps card admission separate from the current deck copy limit', () => {
    const ui = render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
    const add = screen.getByRole('button', { name: '添加 红已准入（ready-0）' });
    for (let count = 0; count < 4; count++) fireEvent.click(add);
    const row = ui.container.querySelector('[data-catalog-card="ready-0"]')!;
    expect(row).toHaveAttribute('data-card-admission', 'buildable');
    expect(row).toHaveTextContent('可组牌');
    expect(row).toHaveTextContent('当前牌组：');
    expect(row).toHaveTextContent('最多 3 张');
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    fireEvent.change(screen.getByRole('spinbutton', { name: '红已准入（ready-0）张数' }), { target: { value: '3' } });
    expect(row).toHaveAttribute('data-card-admission', 'buildable');
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
  });

  it('marks world and society cards as view-only and explains their separate entry rules', () => {
    const world = { id: 'world', name: '世界地区', kind: 'region', cost: 0, text: '', supported: true };
    const society = { id: 'society', name: '测试秘社', kind: 'society', cost: 0, text: '', supported: true };
    expect(deckCardAdmission(world).status).toBe('view-only');
    expect(deckCardAdmission(society).status).toBe('view-only');
    render(<DeckLibrary catalog={{ ...catalog, cards: [world, society] }} />);
    fireEvent.change(screen.getByRole('combobox', { name: '组牌准入' }), { target: { value: 'all' } });
    expect(screen.getAllByText('仅查看')).toHaveLength(2);
    expect(screen.getByRole('button', { name: '添加 世界地区（world）' })).toBeDisabled();
    expect(screen.getByText('地区由世界牌库提供，不加入玩家牌组。')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '添加 测试秘社（society）' })).toBeDisabled();
  });

  it('keeps a buildable black card admitted while the chosen society still requires more red cards', () => {
    const ui = render(<DeckLibrary catalog={{ ...catalog,
      deckBuildRules: { ...catalog.deckBuildRules!, societySupported: true },
      societies: [{ id: 'red-society', name: '红色构筑要求', kind: 'society', cost: 0, text: '', supported: true,
        startingHand: 6, deckConstraints: [{ kind: 'minimumColor', color: '红色', count: 2 }], unresolvedAbilities: {} }],
    }} onSelectDraft={vi.fn()} />);
    fireEvent.change(screen.getByRole('combobox', { name: '秘社（可选）' }), { target: { value: 'red-society' } });
    fireEvent.click(screen.getByRole('button', { name: '添加 黑已准入（ready-1）' }));
    expect(ui.container.querySelector('[data-catalog-card="ready-1"]')).toHaveAttribute('data-card-admission', 'buildable');
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    for (let count = 0; count < 2; count++) fireEvent.click(screen.getByRole('button', { name: '添加 红已准入（ready-0）' }));
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
    expect(ui.container.querySelector('[data-catalog-card="ready-1"]')).toHaveAttribute('data-card-admission', 'buildable');
  });
});
