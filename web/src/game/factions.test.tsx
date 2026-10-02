import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { FactionCoverage } from './FactionCoverage';
import { DeckPicker, Lobby } from './Lobby';
import { deckColors, factionCoverage } from './factions';
import { testCatalog } from './testFixtures';
import type { CardDefinition, Catalog, Deck } from './types';

const definition = (id: string, color: string, kind = 'character'): CardDefinition => ({
  id, color, kind, name: id, cost: 1, text: '已开放牌', supported: true,
});
const colors = [definition('yellow-card', '黄'), definition('green-card', '绿'), definition('blue-card', '蓝'),
  definition('red-card', '红'), definition('gray-card', '灰'), definition('black-card', '黑'), definition('neutral-card', '中立')];
const deck = (id: string, name: string, composition: [string, number][]): Deck => ({
  id, name, description: '受限卡池预组', cardCount: 50, cards: composition.map(([cardId, count]) => ({ cardId, count })),
});
const catalog: Catalog = { ...testCatalog, cards: colors, decks: [
  deck('watchers', '帷幕调查', [['yellow-card', 9], ['neutral-card', 41]]),
  deck('hunters', '公路猎手', [['green-card', 9], ['neutral-card', 41]]),
  deck('keepers', '城市守卫', [['gray-card', 9], ['neutral-card', 41]]),
  deck('reclaimers', '墓地回声', [['black-card', 6], ['blue-card', 3], ['neutral-card', 41]]),
  deck('responders', '牺牲与响应', [['red-card', 5], ['black-card', 4], ['neutral-card', 41]]),
] };

describe('official factions and curated deck boundaries', () => {
  it('counts distinct live player definitions, excluding worlds, societies and unreleased records', () => {
    const coverage = factionCoverage([...colors, { ...colors[0] }, definition('world', '中立', 'region'),
      definition('society', '白', 'society'), { ...definition('unreleased-white', '白'), supported: false },
      { ...definition('unreleased-purple', '紫'), supported: false }]);
    expect(coverage.playerCount).toBe(7); expect(coverage.neutralCount).toBe(1);
    expect(coverage.factions.map(({ faction, count }) => [faction.color, faction.name, count])).toEqual([
      ['黄', '帷幕守望', 1], ['绿', '猎魔人', 1], ['蓝', '王座会', 1], ['红', '鸣钟教派', 1],
      ['灰', '国家机构', 1], ['白', '圣贤', 0], ['黑', '方碑序列', 1], ['紫', '梦境行者', 0],
    ]);
  });
  it('keeps eight faction descriptions separate from five selectable curated decks and does not unlock societies or construction', () => {
    const create = vi.fn();
    const { container } = render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    expect(container.querySelectorAll('[data-deck-id]')).toHaveLength(5);
    expect(screen.getByRole('heading', { name: '5 套自组预组' })).toBeInTheDocument();
    expect(screen.getByText('7 种玩家牌 · 白、紫未实现 · 查看覆盖')).toBeInTheDocument();
    expect(screen.queryByText('02 / 选择你的秘社')).not.toBeInTheDocument();
    fireEvent.click(screen.getByText('官方八色派系'));
    const list = screen.getByRole('list', { name: '官方八色派系覆盖' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(8);
    for (const id of ['white', 'purple']) {
      const row = list.querySelector(`[data-faction-id="${id}"]`)!;
      expect(row).toHaveAttribute('data-open-card-count', '0');
      expect(within(row as HTMLElement).getByText('未实现')).toBeInTheDocument();
    }
    expect(screen.queryByRole('button', { name: /圣贤|梦境行者|自由构筑|秘社选择/ })).not.toBeInTheDocument();
    expect(screen.getByText(/编辑、保存自己的牌组；秘社牌尚未开放/)).toBeInTheDocument();
    expect(screen.getByText(/不是第九个派系/)).toBeInTheDocument();
    expect(container.querySelector('[data-deck-id="watchers"]')).toHaveAttribute('data-deck-main-faction', 'yellow');
    expect(container.querySelector('[data-deck-id="hunters"]')).toHaveAttribute('data-deck-main-faction', 'green');
    expect(container.querySelector('[data-deck-id="keepers"]')).toHaveAttribute('data-deck-main-faction', 'gray');
    expect(container.querySelector('[data-deck-id="reclaimers"]')).toHaveAttribute('data-deck-main-faction', 'black');
    expect(container.querySelector('[data-deck-id="responders"]')).toHaveAttribute('data-deck-main-faction', 'red');
    const responder = screen.getByRole('button', { name: /牺牲与响应/ });
    expect(within(responder).getByText('牺牲 · 响应')).toBeInTheDocument();
    expect(within(responder).getByText('红 · 鸣钟教派 5 张')).toBeInTheDocument();
    expect(within(responder).getByText('黑 · 方碑序列 4 张')).toBeInTheDocument();
    expect(within(responder).getByText('褐 · 中立（无派系） 41 张')).toBeInTheDocument();
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '玩家' } });
    fireEvent.click(responder); fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith('玩家', 'duel', 'responders');
  });
  it('recomputes coverage and deck copies after a catalog change instead of preserving an audit snapshot or neutral 41', () => {
    const changedCards = [...colors, definition('another-yellow', '黄色')];
    const changedDeck = deck('watchers', '帷幕调查', [['yellow-card', 12], ['another-yellow', 3], ['neutral-card', 35]]);
    expect(deckColors(changedDeck, changedCards).colors.map(({ faction, count }) => [faction.id, count])).toEqual([['yellow', 15], ['neutral', 35]]);
    const { container, rerender } = render(<><FactionCoverage catalog={catalog} /><DeckPicker decks={[catalog.decks[0]]} cards={catalog.cards} value="watchers" onChange={vi.fn()} /></>);
    expect(container.querySelector('[data-faction-id="yellow"]')).toHaveAttribute('data-open-card-count', '1');
    rerender(<><FactionCoverage catalog={{ ...catalog, cards: changedCards }} /><DeckPicker decks={[changedDeck]} cards={changedCards} value="watchers" onChange={vi.fn()} /></>);
    expect(container.querySelector('[data-faction-id="yellow"]')).toHaveAttribute('data-open-card-count', '2');
    expect(screen.getByText('黄 · 帷幕守望 15 张')).toBeInTheDocument();
    expect(screen.getByText('褐 · 中立（无派系） 35 张')).toBeInTheDocument();
    expect(screen.queryByText('褐 · 中立（无派系） 41 张')).not.toBeInTheDocument();
  });
  it('keeps room deck changes gated by server-approved deck IDs despite visible faction information', () => {
    const change = vi.fn();
    render(<DeckPicker decks={catalog.decks} cards={catalog.cards} value="watchers" allowedIds={['watchers']} onChange={change} />);
    expect(screen.getByRole('button', { name: /公路猎手/ })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: /公路猎手/ }));
    expect(change).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /帷幕调查/ }));
    expect(change).toHaveBeenCalledExactlyOnceWith('watchers');
  });
});
