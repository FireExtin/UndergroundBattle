// Historical catalog/scan assertions use frozen32; current36 is covered independently by JZ55UniqueDestroy.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/legacy-v0.2.32/hegemony_wasm.js';
import { actionForRoom, actionPayload } from './api';
import { CardContent } from './CardTile';
import { ReadModal } from './ReadModal';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/legacy-v0.2.32/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const definition = catalog.cards.find(c => c.id === 'JC008');
const sha256 = '31340e2535e0909359860a090186b7a5032e84d6da94c8181baf32565b5d4f45';

// Local normal Room commands only: no initial-state edits, hidden-state reads,
// injected resolved effects or public-room credentials. This is not browser QA.
function localRoom() {
  const draft = { id: 'jc008-reader', name: 'JC008 本地投影', description: '', societyId: null,
    cards: [{ cardId: 'JC008', count: 3 }, { cardId: 'JC002', count: 3 }, { cardId: 'JC125', count: 44 }],
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion,
    engineVersion: catalog.engineVersion, updatedAt: '' };
  let room = JSON.parse(kernel.newGameWithDeck('jc008-reader', 'LOCAL', 'duel', 'P0', JSON.stringify(draft), '12'));
  room = JSON.parse(kernel.joinGameWithDeck(room.state, 'P1', JSON.stringify(draft)));
  let count = 0;
  const view = seat => JSON.parse(kernel.view(room.state, seat));
  const session = (seat, action) => {
    const result = JSON.parse(kernel.applyRoom(room.state, seat, JSON.stringify({ commandId: `jc008-reader-${count++}`,
      expectedVersion: room.version, action }), String(1000 + count * 10)));
    expect(result.outcome, result.errorMessage).toBe('accepted');
    room = result;
  };
  const game = (seat, action) => {
    let v = view(seat);
    if (v.responseWindow && !v.pendingChoice && !v.waitingChoice && action.kind !== 'pass') {
      const member = v.responseWindow.members.find(m => m.playerId === v.you);
      if (member?.status === 'undecided') {
        session(seat, { kind: 'beginResponse', windowId: v.responseWindow.id, intentId: `intent-${count}` });
        v = view(seat);
      }
    }
    session(seat, actionForRoom(v, actionPayload(action)));
  };
  const advance = done => {
    for (let n = 0; n < 180; n++) {
      const views = [view(0), view(1)];
      if (done(views)) return;
      const seat = views.findIndex(v => v.pendingChoice);
      if (seat >= 0) {
        const p = views[seat].pendingChoice;
        game(seat, { kind: 'choose', choiceId: p.id,
          selected: p.allowDecline ? [] : p.options.slice(0, p.min ?? 1).map(o => o.id) });
      } else {
        const passer = views.findIndex(v => v.legalActions.some(a => a.kind === 'pass'));
        expect(passer, 'bounded normal-command progress needs a legal passer').toBeGreaterThanOrEqual(0);
        game(passer, { kind: 'pass' });
      }
    }
    throw new Error('bounded normal-command progress did not reach the desired action');
  };
  for (const [seat, kind] of [[0, 'ready'], [1, 'ready'], [0, 'start']]) game(seat, { kind });
  return { view, game, advance };
}

it('admits only JC008 and retains its actual printed metadata and original JPG bytes', () => {
  expect(catalog.engineVersion).toBe('rust-v0.2.32-jc030-blue-bat-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.29-jc030-blue-bat-candidate');
  expect(catalog.cards).toHaveLength(94);
  expect(catalog.cards.some(c => c.id === 'MSJC01')).toBe(false);
  expect(catalog.societies.map(c => c.id)).toEqual(['MSJC09', 'MSJC01', 'MSJC07', 'MSJC06', 'MSJC08', 'MSJC11', 'MSJC02']);
  expect(definition).toMatchObject({ name: '灵能激发', kind: 'spell', type: '法术/心灵',
    cost: 2, loyalty: ['黄色'], magic: '心灵', defense: null, unique: false,
    permanentIcons: { investigation: 0, combat: 0, influence: 0 },
    temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, keywords: [] });
  expect(definition.text).toBe('【快速行动】本回合中，目标角色获得+1防御和1个普通战斗图标。');
  const scans = JSON.parse(readFileSync(resolve('src/game/cardScansV032.fixture.json'), 'utf8'));
  expect(Object.keys(scans)).toHaveLength(101);
  expect(scans.JC008).toEqual({ url: '/cards/JC008.jpg', source: 'resource/ymsj-fun.github.io/cards/JC008 灵能激发.jpg', sha256 });
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC008.jpg'))).digest('hex')).toBe(sha256);
});

it('reads the actual own-hand WASM spell projection and opens its original without inventing defense', () => {
  const local = localRoom();
  const card = local.view(0).hand.find(c => c.cardId === 'JC008');
  expect(card).toMatchObject({ owner: 'p0', controller: 'p0', kind: 'spell', cost: 2 });
  expect(Object.hasOwn(card, 'defense')).toBe(false);
  const rendered = render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()} />);
  expect(screen.getByRole('dialog', { name: '放大阅读灵能激发' })).toHaveTextContent('普通战斗图标');
  expect(screen.queryByText(/^防御 /)).not.toBeInTheDocument();
  expect(screen.getByRole('dialog')).not.toHaveTextContent('null');
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: '灵能激发原始牌面' })).toHaveAttribute('src', '/cards/JC008.jpg');
  rendered.unmount();
});

it('renders current versus printed stats from paid, resolved JC008 using normal Room commands and expires them normally', () => {
  const local = localRoom();
  const perform = predicate => {
    local.advance(views => views[0].legalActions.some(a => predicate(a, views[0])));
    const v = local.view(0);
    const a = v.legalActions.find(a => predicate(a, v));
    local.game(0, a);
    return a;
  };
  perform((a, v) => a.kind === 'asset' && v.hand.find(c => c.instanceId === a.cardId)?.cardId === 'JC002');
  perform((a, v) => a.kind === 'deploy' && a.region === 0 && v.hand.find(c => c.instanceId === a.cardId)?.cardId === 'JC125');
  local.advance(views => views[0].regions[0].characters.some(c => c.cardId === 'JC125'));
  const target = local.view(0).regions[0].characters.find(c => c.cardId === 'JC125');
  perform((a, v) => a.kind === 'asset' && v.hand.find(c => c.instanceId === a.cardId)?.cardId === 'JC125');
  perform((a, v) => a.kind === 'play' && a.targetId === target.instanceId && v.hand.find(c => c.instanceId === a.cardId)?.cardId === 'JC008');
  local.advance(views => views[0].regions[0].characters.find(c => c.instanceId === target.instanceId)?.defense === 2);
  const boosted = local.view(0).regions[0].characters.find(c => c.instanceId === target.instanceId);
  expect(boosted.icons.combat).toBe(1);
  expect(local.view(0).assets.filter(c => c.controller === 'p0' && c.exhausted)).toHaveLength(2);
  expect(local.view(0).graveyard.some(c => c.cardId === 'JC008')).toBe(true);
  expect(local.view(1).regions[0].characters.find(c => c.instanceId === target.instanceId)).toEqual(boosted);
  const printed = catalog.cards.find(c => c.id === 'JC125');
  expect(printed.defense).toBe(1);
  expect(printed.permanentIcons.combat).toBe(0);
  let rendered = render(<CardContent card={boosted} definition={printed} viewerId="p0" />);
  expect(screen.getByText('当前防御 2 · 印刷防御 1')).toBeInTheDocument();
  expect(screen.getByLabelText('当前有效图标：调查0，战斗1，势力0')).toBeInTheDocument();
  expect(screen.getByLabelText('印刷图标：调查0，战斗0，势力0')).toBeInTheDocument();
  rendered.unmount();
  const castTurn = local.view(0).turn;
  local.advance(views => views[0].turn > castTurn);
  const expired = local.view(0).regions[0].characters.find(c => c.instanceId === target.instanceId);
  expect(expired.defense).toBe(1);
  expect(expired.icons.combat).toBe(0);
  rendered = render(<CardContent card={expired} definition={printed} viewerId="p0" />);
  expect(screen.getByText('当前防御 1 · 印刷防御 1')).toBeInTheDocument();
  rendered.unmount();
});
