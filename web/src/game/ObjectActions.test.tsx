import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Card, LegalAction, View } from './types';

const opponent = { ...testView.players[0], id: 'p1', name: '乙', seat: 1, team: 1 };
const graves: Card[] = Array.from({ length: 7 }, (_, index) => ({ ...testCard, instanceId: `grave-${index}`, name: `墓地角色${index + 1}`, owner: index < 3 ? 'p0' : 'p1', controller: index < 3 ? 'p0' : 'p1' }));
const funeral = { ...testCard, kind: 'spell', cardId: 'XQ49', name: '葬礼' };
const actions: LegalAction[] = graves.map(card => ({ id: `funeral-${card.instanceId}`, kind: 'play', cardId: funeral.instanceId, abilityId: 'funeral', targetId: card.instanceId, label: `葬礼：发动 → ${card.name}` }));
const view: View = { ...testView, status: 'playing', players: [...testView.players, opponent], hand: [funeral], graveyard: graves, legalActions: [...actions, { id: 'pass', kind: 'pass', label: '让过' }] };
const props = { catalog: testCatalog, busy: false, connection: 'online' as const };
const open = () => { fireEvent.click(screen.getByRole('button', { name: '查看葬礼' })); fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' })); };

describe('object actions on a synthetic table', () => {
  it('shows description-only distinctions as separately named object actions', () => {
    const variants = [{ ...actions[0], description: '方式甲' }, { ...actions[1], description: '方式乙' }];
    const { container } = render(<Table {...props} view={{ ...view, legalActions: variants }} onAction={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    expect(screen.getByRole('button', { name: '葬礼：发动 · 方式甲' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动 · 方式乙' }));
    expect(container.querySelector('[data-card-instance="grave-1"]')).toHaveAttribute('data-card-targeted', 'true');
    expect(container.querySelector('[data-card-instance="grave-0"]')).toBeNull();
  });

  it('keeps ability, mode, and explicit sacrifice alternatives distinct while narrowing target highlights to the chosen action', () => {
    const variants = [
      { ...actions[0], costSelected: ['cost-a'], label: '葬礼：模式甲 → 墓地角色1（费用：牺牲甲）' },
      { ...actions[1], costSelected: ['cost-b'], label: '葬礼：模式甲 → 墓地角色2（费用：牺牲乙）' },
      { ...actions[2], abilityId: 'another', option: 'other-mode', label: '葬礼：模式乙 → 墓地角色3' },
    ];
    const { container } = render(<Table {...props} view={{ ...view, legalActions: variants }} onAction={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    expect(container.querySelectorAll('[data-object-action]')).toHaveLength(3);
    fireEvent.click(screen.getByRole('button', { name: '葬礼：模式甲（费用：牺牲乙）' }));
    expect(container.querySelector('[data-card-instance="grave-1"]')).toHaveAttribute('data-card-targeted', 'true');
    expect(container.querySelector('[data-card-instance="grave-0"]')).toBeNull();
    expect(container.querySelector('[data-card-instance="grave-2"]')).toBeNull();
  });

  it('selects a player directly from their highlighted mat and leaves a target-only action on its own card', () => {
    const submit = vi.fn();
    const playerAction = { ...actions[0], targetId: 'p1', label: '葬礼：选玩家 → 乙' };
    const ownAction = { id: 'own', kind: 'reveal', targetId: funeral.instanceId, label: '此对象自身动作' };
    render(<Table {...props} view={{ ...view, legalActions: [playerAction, ownAction] }} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    expect(screen.getByRole('button', { name: ownAction.label })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '葬礼：选玩家' }));
    fireEvent.click(screen.getByRole('button', { name: '选择玩家乙' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · ${playerAction.label}` }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(playerAction);
  });

  it('keeps old-room actions with an unavailable source reachable globally without guessing an object identity', () => {
    const submit = vi.fn();
    const orphan = { id: 'orphan', kind: 'activate', cardId: 'unprojected-source', abilityId: 'legacy', label: '旧房间行动' };
    render(<Table {...props} view={{ ...view, attachments: undefined, legalActions: [orphan] }} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: orphan.label }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(orphan);
  });

  it('blocks an unprojected target without offering numbered blind selections', () => {
    const submit = vi.fn();
    render(<Table {...props} view={{ ...view, graveyard: [], legalActions: [actions[0]] }} onAction={submit} />);
    open();
    expect(screen.getByText(/部分目标当前未显示，无法选择这些目标/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /服务端目标/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '点选高亮对象后确认' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '取消选目标' }));
    expect(submit).not.toHaveBeenCalled();
  });

  it('blocks a selected target removed from the projection even if revision and legal action stay unchanged', () => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    open(); fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    rerender(<Table {...props} view={{ ...view, graveyard: graves.slice(1) }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByText(/部分目标当前未显示，无法选择这些目标/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看墓地角色2' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · ${actions[1].label}` }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[1]);
  });

  it('blocks an old-room global action whose destination is not projected', () => {
    const submit = vi.fn();
    const orphan = { ...actions[0], cardId: 'unprojected-source', label: '旧房间缺失目标行动' };
    render(<Table {...props} view={{ ...view, graveyard: [], legalActions: [orphan] }} onAction={submit} />);
    const button = screen.getByRole('button', { name: orphan.label });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('title', '目标当前未显示，请等待牌桌更新。');
    fireEvent.click(button);
    expect(submit).not.toHaveBeenCalled();
  });

  it('replaces seven funeral combination buttons with player graveyard targets and confirms the untouched legal action once', () => {
    const submit = vi.fn();
    const { container } = render(<Table {...props} view={view} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    expect(container.querySelectorAll('[data-object-action]')).toHaveLength(1);
    for (const action of actions) expect(screen.queryByRole('button', { name: action.label })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
    for (const owner of ['p0', 'p1']) {
      const pile = container.querySelector(`[data-pile-owner="${owner}"][data-pile-kind="graveyard"]`)!;
      expect(pile).toHaveAttribute('open'); expect(pile).toHaveAttribute('data-pile-targeted', 'true');
      expect(pile.querySelectorAll('[data-card-targeted="true"]')).toHaveLength(owner === 'p0' ? 3 : 4);
    }
    fireEvent.click(screen.getByRole('button', { name: '前往高亮对象' }));
    expect(document.activeElement).toHaveAttribute('data-card-targeted', 'true');
    fireEvent.click(screen.getByRole('button', { name: '查看墓地角色5' }));
    expect(screen.getByText('目标：乙的墓地角色5')).toBeInTheDocument();
    expect(container.querySelector('[data-card-instance="instance-a"]')).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
    const confirm = screen.getByRole('button', { name: `确认 · ${actions[4].label}` });
    fireEvent.click(confirm); fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[4]);
  });

  it('only opens legal graveyard candidates and never reconstructs a concealed target from the catalog', () => {
    const concealed = { ...graves[0], faceDown: true, name: '不得显示的身份', text: '不得显示的文字', controller: 'p1' };
    const unrelated = { ...testCard, instanceId: 'not-legal', name: '无合法目标资格' };
    const { container } = render(<Table {...props} view={{ ...view, graveyard: [concealed, unrelated], legalActions: [actions[0]] }} onAction={vi.fn()} />);
    open();
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toHaveAttribute('data-card-targeted', 'true');
    expect(screen.queryByText('无合法目标资格')).not.toBeInTheDocument();
    expect(container.textContent).not.toContain('不得显示的身份');
    expect(container.textContent).not.toContain('不得显示的文字');
    fireEvent.click(screen.getByRole('button', { name: '查看暗藏者' }));
    fireEvent.click(screen.getByRole('button', { name: '放大阅读所选目标' }));
    expect(screen.getByRole('dialog', { name: '放大阅读暗藏者' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '原始牌面' })).not.toBeInTheDocument();
  });

  it('cancels locally by button or Escape, while Escape in the reader closes only the reader', () => {
    const submit = vi.fn();
    render(<Table {...props} view={view} onAction={submit} />);
    open(); fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    fireEvent.click(screen.getByRole('button', { name: '放大阅读所选目标' }));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(screen.getByRole('region', { name: '点选合法目标' })).toBeInTheDocument();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '葬礼：发动' })).toHaveFocus();
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
    fireEvent.click(screen.getByRole('button', { name: '取消选目标' }));
    expect(submit).not.toHaveBeenCalled();
  });

  it('rejects non-target clicks, switches legal targets, and blocks busy or uncertain confirmation', () => {
    const submit = vi.fn();
    const content = (busy = false, uncertain = false) => <Table {...props} view={{ ...view, hand: [funeral, { ...testCard, instanceId: 'other', name: '其他手牌' }] }} busy={busy} uncertain={uncertain} onAction={submit} />;
    const { rerender } = render(content());
    open(); fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    fireEvent.click(screen.getByRole('button', { name: '查看其他手牌' }));
    expect(screen.getByText(/此对象不是当前动作的合法目标/)).toBeInTheDocument();
    expect(screen.getByText('目标：甲的墓地角色1')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看墓地角色2' }));
    expect(screen.getByText('目标：甲的墓地角色2')).toBeInTheDocument();
    for (const lock of [[true, false], [false, true]]) {
      rerender(content(...lock));
      const confirm = screen.getByRole('button', { name: `确认 · ${actions[1].label}` });
      expect(confirm).toBeDisabled(); fireEvent.click(confirm);
    }
    expect(submit).not.toHaveBeenCalled();
  });

  it('invalidates an expired target even when the replacement has the same printed card', () => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    open(); fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    const oldConfirm = screen.getByRole('button', { name: `确认 · ${actions[0].label}` });
    rerender(<Table {...props} view={{ ...view, version: 2, graveyard: [{ ...graves[0], instanceId: 'replacement' }], legalActions: [view.legalActions.at(-1)!] }} onAction={submit} />);
    fireEvent.click(oldConfirm);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(screen.getByText('牌桌已更新，请重新选择动作与目标。')).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(['room', 'viewer'])('drops all local selections on a %s switch with colliding versions and instance IDs', kind => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    open(); fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    rerender(<Table {...props} view={{ ...view, ...(kind === 'room' ? { roomId: 'other-room' } : { you: 'p1' }) }} onAction={submit} />);
    expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it('keeps a private search in its modal and resets expired options without exposing them in player zones', () => {
    const submit = vi.fn();
    const privateCard = { ...testCard, instanceId: 'private', name: '仅本人检索牌' };
    const choice = { id: 'search', kind: 'search', title: '检索', description: '本人可见', playerId: 'p0', min: 1, max: 1, options: [{ id: 'private', label: privateCard.name, card: privateCard }] };
    const choosing = { ...view, pendingChoice: choice, legalActions: [{ id: 'choose', kind: 'choose', label: '确认选择', choiceId: 'search' }] };
    const { container, rerender } = render(<Table {...props} view={view} onAction={submit} />);
    open();
    rerender(<Table {...props} view={choosing} onAction={submit} />);
    const dialog = screen.getByRole('dialog', { name: '待完成的选择' });
    fireEvent.click(within(dialog).getByRole('button', { name: /仅本人检索牌.*真实印刷文字/ }));
    expect(within(dialog).getByRole('button', { name: '确认选择' })).toBeEnabled();
    expect(container.querySelector('.hg-team-edge')?.textContent).not.toContain(privateCard.name);
    rerender(<Table {...props} view={{ ...choosing, version: 2, pendingChoice: { ...choice, options: [{ id: 'new-private', label: '新选择', card: { ...privateCard, instanceId: 'new-private', name: '新选择' } }] } }} onAction={submit} />);
    expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
    expect(screen.queryByText(privateCard.name)).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each([false, true])('preserves old/new response envelopes and explicit sacrifice costs (modern=%s)', modern => {
    const source = { ...testCard, region: 0 };
    const target = { ...graves[0], region: 0 };
    const fast = { ...actions[0], kind: 'activate', cardId: source.instanceId, targetId: target.instanceId, abilityId: 'fast', option: 'mode-a', costSelected: ['sacrifice'], label: '发动快速响应 → 墓地角色1' };
    const response: View = { ...view, hand: [], graveyard: [], regions: [{ id: 'r0', index: 0, cardId: 'world', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters: [source, target] }],
      stack: [{ id: 'top', label: '等待连锁', controller: 'p1' }], legalActions: [fast, { id: 'pass', kind: 'pass', label: '让过' }],
      ...(modern ? { serverNowMs: 1000, responseWindow: { id: 'window', stackTopId: 'top', holderTeam: 0, canBegin: true, myIntentId: 'original-intent', members: [{ playerId: 'p0', status: 'composing' as const }] } } : {}) };
    const submit = vi.fn();
    render(<Table {...props} view={response} onAction={action => submit(actionForRoom(response, action))} />);
    fireEvent.click(screen.getByRole('button', { name: '查看响应牌' }));
    fireEvent.click(screen.getByRole('button', { name: '发动快速响应' }));
    fireEvent.click(screen.getByRole('button', { name: '查看墓地角色1' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · ${fast.label}` }));
    const payload = { kind: 'activate', cardId: source.instanceId, targetId: target.instanceId, abilityId: 'fast', option: 'mode-a', costSelected: ['sacrifice'] };
    expect(JSON.parse(JSON.stringify(submit.mock.calls[0][0]))).toEqual(modern ? { kind: 'submitResponse', windowId: 'window', intentId: 'original-intent', action: payload } : payload);
  });

  it('exposes mounted attachment actions through their own object, and keeps unrelated deployments out of region actions', () => {
    const submit = vi.fn();
    const host = { ...testCard, instanceId: 'host', region: 0, name: '宿主' };
    const attachment = { ...testCard, instanceId: 'attachment', kind: 'attachment', hostId: host.instanceId, region: 0, name: '附属' };
    const activate = { id: 'attached-activate', kind: 'activate', cardId: attachment.instanceId, abilityId: 'attached', label: '发动附属能力' };
    const region = { id: 'r0', index: 0, cardId: 'world', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters: [host] };
    render(<Table {...props} view={{ ...view, attachments: [attachment], regions: [region], legalActions: [activate, { id: 'deploy', kind: 'deploy', cardId: funeral.instanceId, region: 0, label: '来源牌派遣' }, { id: 'region', kind: 'privilege', region: 0, label: '地区动作' }] }} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看附属附属' }));
    fireEvent.click(screen.getByRole('button', { name: activate.label }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(activate);
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 香港' }));
    expect(screen.getByRole('button', { name: '地区动作 · 地区 1 · 香港' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /来源牌派遣/ })).not.toBeInTheDocument();
  });
});
