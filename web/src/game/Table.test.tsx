import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { RoomLobby, Table } from './Table';
import { createDeckDraft, DECK_LIBRARY_STORAGE_KEY, readDeckLibrary, saveDeckLibrary } from './deckLibrary';
import { testCatalog, testView, testChoice } from './testFixtures';
import { testCard } from './testFixtures';
import type { Catalog, View } from './types';

describe('room deck selection', () => {
  const catalog: Catalog = { ...testCatalog, deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false }, cards: testCatalog.cards.map(card => ({ ...card, supported: true, deckCopyLimit: null })) };
  const saved = { ...createDeckDraft(catalog, catalog.decks[0]), name: '已保存的牌组' };
  const frozen = { ...saved, id: 'frozen-deck', name: '冻结牌组' };
  const view: View = { ...testView, yourDeck: frozen, players: [{ ...testView.players[0], deckName: frozen.name, ready: true }], legalActions: [{ id: 'deck', kind: 'deck', option: 'watchers', label: '更换预组' }] };
  beforeEach(() => { localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY); saveDeckLibrary([saved]); });
  afterEach(() => localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY));

  it('saves edits locally and sends a full draft only after explicit selection, retaining the frozen public name', () => {
    const submit = vi.fn();
    render(<RoomLobby view={view} catalog={catalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByText('更换牌组'));
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue(saved.name);
    fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: '修改后的草稿' } });
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0].name).toBe('修改后的草稿');
    expect(submit).not.toHaveBeenCalled();
    expect(screen.getByRole('heading', { name: '冻结牌组' })).toBeInTheDocument();
    expect(screen.getByText('✓ 已准备')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ kind: 'deck', deckDraft: readDeckLibrary().drafts[0] });
    expect(screen.getByRole('heading', { name: '冻结牌组' })).toBeInTheDocument();
    expect(screen.getByText(/选择其他牌组后将取消准备/)).toBeInTheDocument();
  });

  it('disables the saved deck editor and selection while confirming an action', () => {
    const submit = vi.fn();
    render(<RoomLobby view={view} catalog={catalog} busy onAction={submit} />);
    fireEvent.click(screen.getByText('更换牌组'));
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toBeDisabled();
    const select = screen.getByRole('button', { name: '保存并选择此牌组' });
    expect(select).toBeDisabled();
    fireEvent.click(select);
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(['legacy catalog', 'missing frozen draft', 'no legal deck action'])('does not expose saved deck selection for %s', missing => {
    render(<RoomLobby view={{ ...view, ...(missing === 'missing frozen draft' ? { yourDeck: undefined } : {}), ...(missing === 'no legal deck action' ? { legalActions: testView.legalActions } : {}) }} catalog={missing === 'legacy catalog' ? testCatalog : catalog} busy={false} onAction={vi.fn()} />);
    fireEvent.click(screen.getByText('更换牌组'));
    expect(screen.queryByRole('heading', { name: '命名、编辑与选择牌组' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '保存并选择此牌组' })).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '冻结牌组' })).toBeInTheDocument();
  });
});

describe('playable table', () => {
  it('shows readable cards and only the selected card’s server-authored actions', () => {
    const submit = vi.fn(); const action = { id: 'deploy-a', kind: 'deploy', label: '派遣到地区一', cardId: 'instance-a', region: 0 };
    const regions = [{ id: 'region-one', index: 0, cardId: 'DQJC112', name: '纽约', threshold: 4, points: 4, influence: [0, 0], characters: [] }];
    render(<Table view={{ ...testView, status: 'playing', regions, hand: [testCard, { ...testCard, instanceId: 'another-card', name: '另一张牌' }], legalActions: [action, { id: 'unrelated', kind: 'asset', label: '不相关行动', cardId: 'another-card' }] }} catalog={testCatalog} busy={false} onAction={submit} />);
    expect(screen.queryByRole('button', { name: '派遣到地区一' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看无知路人' }));
    expect(screen.getAllByText('真实印刷文字').length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole('button', { name: '派遣到地区一' }));
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 纽约' }));
    fireEvent.click(screen.getByRole('button', { name: '确认 · 派遣到地区一 · 地区 1 · 纽约' }));
    expect(submit).toHaveBeenCalledWith(action);
    expect(screen.queryByRole('button', { name: '不相关行动' })).not.toBeInTheDocument();
  });
  it.each(['deploy', 'move'])('distinguishes two same-name destinations for %s and highlights only the intended region', kind => {
    const submit = vi.fn();
    const regions = [1, 2].map(index => ({ id: `new-york-${index}`, index, cardId: 'DQJC112', name: '纽约', threshold: 4, points: 4, influence: [0, 0], characters: [] }));
    const actions = regions.map(region => ({ id: `${kind}-${region.index}`, kind, cardId: testCard.instanceId, region: region.index, label: `${kind === 'deploy' ? '派遣' : '移动'} 无知路人 → 纽约` }));
    render(<Table view={{ ...testView, status: 'playing', regions, legalActions: actions }} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看无知路人' }));
    expect(screen.queryByRole('button', { name: `${kind === 'deploy' ? '派遣' : '移动'} 无知路人 → 地区 2 · 纽约` })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: `${kind === 'deploy' ? '派遣' : '移动'} 无知路人` }));
    const region2 = screen.getByRole('article', { name: '地区 2 · 纽约' });
    const region3 = screen.getByRole('article', { name: '地区 3 · 纽约' });
    expect(region2).toHaveAttribute('data-region-targeted', 'true');
    expect(region3).toHaveAttribute('data-region-targeted', 'true');
    fireEvent.click(screen.getByRole('button', { name: '查看地区3 纽约' }));
    const confirm = screen.getByRole('button', { name: `确认 · ${kind === 'deploy' ? '派遣' : '移动'} 无知路人 → 地区 3 · 纽约` });
    expect(confirm).toHaveAttribute('aria-controls', region3.id);
    expect(screen.queryByRole('button', { name: `${kind === 'deploy' ? '派遣' : '移动'} 无知路人 → 地区 2 · 纽约` })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[1]);
  });
  it('keeps region-scoped actions identifiable when the board has repeated world names', () => {
    const submit = vi.fn();
    const regions = [1, 2].map(index => ({ id: `new-york-${index}`, index, cardId: 'DQJC112', name: '纽约', threshold: 4, points: 4, influence: [0, 0], characters: [] }));
    const action = { id: 'select-region-2', kind: 'privilege', region: 2, label: '选择纽约地区' };
    render(<Table view={{ ...testView, status: 'playing', regions, legalActions: [action] }} catalog={testCatalog} busy={false} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看地区3 纽约' }));
    fireEvent.click(screen.getByRole('button', { name: '选择纽约地区 · 地区 3 · 纽约' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(action);
  });
  it('distinguishes waiting chooser from the priority holder', () => {
    render(<Table view={{ ...testView, status: 'playing', legalActions: [], waitingChoice: { playerId: 'p0', kind: 'investigation', title: '排列调查牌' } }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByText('等待 甲：排列调查牌')).toBeInTheDocument();
    expect(screen.getByText('优先权')).toBeInTheDocument();
  });
  it('keeps an opponent stack effect beside the dock, submits its server-authored response and shows an invalidated old target', () => {
    const submit = vi.fn();
    const opponent = { ...testView.players[0], id: 'p1', seat: 1, name: '乙', team: 1 };
    const source = { ...testCard, instanceId: 'response-source', name: '响应角色', region: 0 };
    const victim = { ...testCard, instanceId: 'old-target', name: '被指定角色', region: 0 };
    const fast = { id: 'fast-sacrifice', label: '发动快速响应', kind: 'activate', cardId: source.instanceId, abilityId: 'quick-response', costSelected: [victim.instanceId] };
    const effect = { id: 'opponent-effect', label: '对方指定一个角色', controller: 'p1', targetId: victim.instanceId, targets: [victim.instanceId], resolutionState: 'awaitingResponses' as const,
      targetSummaries: [{ instanceId: victim.instanceId, label: victim.name, kind: 'character', owner: 'p0', controller: 'p0', region: 0, valid: true, status: 'valid' as const }] };
    const view: View = { ...testView, status: 'playing', players: [testView.players[0], opponent], hand: [], stack: [effect], legalActions: [{ id: 'pass', kind: 'pass', label: '让过' }, fast],
      regions: [{ id: 'region', index: 0, cardId: 'world', name: '地区甲', threshold: 3, points: 3, influence: [0, 0], characters: [source, victim] }] };
    const { rerender } = render(<Table view={view} catalog={testCatalog} busy={false} connection="online" onAction={submit} />);
    const dock = screen.getByRole('region', { name: '固定手牌与下一步行动' });
    const response = within(dock).getByRole('region', { name: '当前待结算效果与响应' });
    expect(response).toHaveAttribute('data-response-state', 'respond');
    expect(within(response).getByText('对方指定一个角色')).toBeInTheDocument();
    expect(within(response).getByText('优先权：甲')).toBeInTheDocument();
    expect(within(response).getByText('被指定角色')).toBeInTheDocument();
    fireEvent.click(within(response).getByRole('button', { name: '查看响应牌' }));
    fireEvent.click(screen.getByRole('button', { name: '发动快速响应' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(fast);
    const updated: View = { ...view, version: 2, legalActions: [{ id: 'pass-new', kind: 'pass', label: '让过' }], regions: [{ ...view.regions[0], characters: [source] }],
      graveyard: [{ ...victim, instanceId: 'new-graveyard-instance', region: undefined }],
      stack: [{ ...effect, targetSummaries: [{ ...effect.targetSummaries[0], valid: false, status: 'missing', invalidReason: '原目标已离场' }] }] };
    rerender(<Table view={updated} catalog={testCatalog} busy={false} connection="online" onAction={submit} />);
    expect(response).toHaveAttribute('data-response-state', 'pass');
    expect(within(response).getByText('原目标已离场')).toBeInTheDocument();
    expect(response.querySelector('[data-target-instance="old-target"]')).toHaveAttribute('data-target-valid', 'false');
    expect(response.querySelector('[data-target-instance="new-graveyard-instance"]')).toBeNull();
  });
  it('keeps the viewing team below the world in every seat, identifies pieces and opens an owner-only readable card', () => {
    const players = Array.from({ length: 4 }, (_, seat) => ({ ...testView.players[0], id: `p${seat}`, seat, name: `玩家${seat + 1}`, team: Math.floor(seat / 2) }));
    const characters = players.map(player => ({ ...testCard, instanceId: `secret-${player.seat}`, name: `身份${player.seat + 1}`, text: `私密文字${player.seat + 1}`, owner: player.id, controller: player.id, faceDown: true, kind: 'hidden' }));
    const view = { ...testView, status: 'playing' as const, mode: 'teams' as const, players, hand: [], legalActions: [], regions: [{ id: 'region-a', index: 0, cardId: 'world', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters }] };
    const { container, rerender } = render(<Table view={view} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    for (const player of players) {
      rerender(<Table view={{ ...view, you: player.id }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
      const sides = container.querySelectorAll('.hg-region-side');
      expect(sides[0]).toHaveAttribute('data-side', 'opponent');
      expect(sides[1]).toHaveAttribute('data-side', 'mine');
      expect(sides[1]).toHaveAttribute('data-team', String(player.team));
      expect(screen.getByRole('button', { name: `查看身份${player.seat + 1}` })).toBeInTheDocument();
      for (const other of players.filter(other => other.id !== player.id)) expect(screen.queryByRole('button', { name: `查看身份${other.seat + 1}` })).not.toBeInTheDocument();
    }
    fireEvent.click(screen.getByRole('button', { name: '查看身份4' }));
    fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
    expect(screen.getByRole('dialog', { name: '放大阅读身份4' })).toBeInTheDocument();
    expect(screen.getAllByText('私密文字4').length).toBeGreaterThan(0);
    expect(screen.queryByText('私密文字1')).not.toBeInTheDocument();
    expect(container.querySelector('.hg-dock-action')).toHaveAttribute('data-next-step', 'wait');
    expect(screen.queryByRole('button', { name: '等待玩家 · 可浏览' })).not.toBeInTheDocument();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.getByRole('button', { name: '等待玩家 · 可浏览' })).toBeDisabled();
  });
  it('keeps teammate resources and hidden hands in separate mats on the same table side', () => {
    const players = Array.from({ length: 4 }, (_, seat) => ({ ...testView.players[0], id: `p${seat}`, seat, name: `玩家${seat}`, team: Math.floor(seat / 2), deckName: `独立牌组${seat}` }));
    const { container } = render(<Table view={{ ...testView, status: 'playing', mode: 'teams', players }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    const mine = screen.getByRole('region', { name: '我方玩家区' });
    const opponent = screen.getByRole('region', { name: '对方玩家区' });
    expect(within(mine).getByRole('region', { name: '玩家0的玩家区' })).toBeInTheDocument();
    expect(within(mine).getByRole('region', { name: '玩家1的玩家区' })).toBeInTheDocument();
    expect(within(opponent).getByRole('region', { name: '玩家2的玩家区' })).toBeInTheDocument();
    expect(within(opponent).getByRole('region', { name: '玩家3的玩家区' })).toBeInTheDocument();
    for (const player of players) {
      expect(screen.getByLabelText(`${player.name}的牌库，49张`)).toBeInTheDocument();
      expect(screen.getByText(`席位 ${player.seat + 1} · ${player.deckName}`)).toBeInTheDocument();
      expect(screen.getByLabelText(`${player.name}的秘社区，尚未实现`)).toBeInTheDocument();
    }
    expect(container.querySelector('[data-hand-owner="p1"]')).toHaveAttribute('data-hand-count', '1');
    expect(container.querySelectorAll('[data-pile-kind="graveyard"]')).toHaveLength(4);
  });
  it('highlights only server-authored card targets and keeps the source selected when a target is clicked', () => {
    const enemy = { ...testCard, instanceId: 'enemy', owner: 'p1', controller: 'p1', name: '合法目标' };
    const other = { ...enemy, instanceId: 'other', name: '非目标' };
    const opponent = { ...testView.players[0], id: 'p1', seat: 1, team: 1, name: '乙' };
    const action = { id: 'target-action', kind: 'play', cardId: testCard.instanceId, targetId: enemy.instanceId, label: '对合法目标发动' };
    const { container } = render(<Table view={{ ...testView, status: 'playing', players: [...testView.players, opponent], legalActions: [action], regions: [{ id: 'r', index: 0, cardId: 'w', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters: [enemy, other] }] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '查看无知路人' }));
    expect(container.querySelector('[data-card-instance="enemy"]')).toHaveAttribute('data-card-targeted', 'true');
    expect(container.querySelector('[data-card-instance="other"]')).toHaveAttribute('data-card-targeted', 'false');
    fireEvent.click(screen.getByRole('button', { name: '查看合法目标' }));
    expect(container.querySelector('[data-card-instance="instance-a"]')).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('button', { name: '确认 · 对合法目标发动' })).toBeInTheDocument();
  });
  it('reads authorized concealed print for its controller and sanitizes hover for its owner', () => {
    const secret = { ...testCard, instanceId: 'teammate-secret', owner: 'p1', controller: 'p0', faceDown: true, kind: 'hidden', name: '获授权身份', text: '获授权能力' };
    const players = [...testView.players, { ...testView.players[0], id: 'p1', seat: 1, team: 1, name: '乙' }];
    const view = { ...testView, status: 'playing' as const, players, hand: [], regions: [{ id: 'r', index: 0, cardId: 'w', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters: [secret] }] };
    const { rerender } = render(<Table view={view} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    fireEvent.mouseEnter(screen.getByRole('button', { name: '查看获授权身份' }));
    expect(within(screen.getByRole('complementary', { name: '悬停阅读获授权身份' })).getByText('获授权能力')).toBeInTheDocument();
    rerender(<Table view={{ ...view, you: 'p1' }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    fireEvent.mouseEnter(screen.getByRole('button', { name: '查看暗藏者' }));
    const preview = screen.getByRole('complementary', { name: '悬停阅读暗藏者' });
    expect(within(preview).getByLabelText('图标：调查0，战斗0，势力1')).toBeInTheDocument();
    expect(screen.queryByText('获授权身份')).not.toBeInTheDocument();
    expect(screen.queryByText('获授权能力')).not.toBeInTheDocument();
  });
  it('keeps pending choices reachable in the temporary table layer and submits the original legal choice', () => {
    const submit = vi.fn();
    const action = { id: 'choose', kind: 'choose', label: '确认', choiceId: testChoice.id };
    render(<Table view={{ ...testView, status: 'playing', pendingChoice: testChoice, legalActions: [action] }} catalog={testCatalog} busy={false} onAction={submit} />);
    const panel = screen.getByRole('dialog', { name: '待完成的选择' });
    fireEvent.click(within(panel).getByRole('button', { name: /角色甲/ }));
    fireEvent.click(within(panel).getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: testChoice.id, selected: ['a'] });
  });

  it('shows a world deck count only when the frozen room projection supplies it', () => {
    const { rerender } = render(<Table view={{ ...testView, status: 'playing' }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    const pile = screen.getByLabelText('世界牌库');
    expect(pile.querySelector('b')).toBeNull();
    rerender(<Table view={{ ...testView, status: 'playing', worldDeckCount: 7 }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(within(pile).getByText('7')).toBeInTheDocument();
  });

  it('uses the optional authoritative contest totals and labels accumulated influence as markers', () => {
    const region = { id: 'r', index: 0, cardId: 'w', name: '香港', threshold: 3, points: 3, influence: [2, 1], characters: [testCard] };
    const { rerender } = render(<Table view={{ ...testView, status: 'playing', regions: [{ ...region, iconsByTeam: [{ investigation: 7, combat: 6, influence: 5 }, { investigation: 3, combat: 2, influence: 1 }] }] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByRole('img', { name: '当前对抗图标：我方，调查7，战斗6，势力5' })).toBeInTheDocument();
    expect(screen.getByRole('img', { name: '当前对抗图标：对方，调查3，战斗2，势力1' })).toBeInTheDocument();
    expect(screen.getAllByText('势力标志')).toHaveLength(2);
    rerender(<Table view={{ ...testView, status: 'playing', regions: [region] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByRole('img', { name: /当前对抗图标/ })).not.toBeInTheDocument();
    expect(screen.getAllByText('势力标志')).toHaveLength(2);
  });

  it('shows the public skip-comparison flag and its reading explanation without inferring from a card ID', () => {
    const region = { id: 'r', index: 0, cardId: 'same-world-card', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters: [] };
    const { rerender } = render(<Table view={{ ...testView, status: 'playing', regions: [{ ...region, skipConfrontation: true }] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByText('本回合略过对抗比较')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 香港' }));
    expect(within(screen.getByRole('complementary', { name: '选牌行动' })).getByText('本回合略过对抗比较。')).toBeInTheDocument();
    rerender(<Table view={{ ...testView, status: 'playing', regions: [region] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.queryByText('本回合略过对抗比较')).not.toBeInTheDocument();
    expect(screen.queryByText('本回合略过对抗比较。')).not.toBeInTheDocument();
  });

});
