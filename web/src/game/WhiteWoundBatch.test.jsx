import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { CardTile } from './CardTile';
import { Table } from './Table';
import { testView } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const definition = catalog.cards.find(c => c.id === 'LC06');
const card = { ...definition, instanceId: 'lc06-controller2', cardId: 'LC06', owner: 'p3', controller: 'p2', region: 0, faceDown: false, exhausted: true, wounds: 1, damage: 0, defense: 1 };
it('reads the actual full LC06 fields including its printed holy domain and exact original bytes', () => {
  expect(definition).toMatchObject({ name: '瓦尔德修士', subtitle: '苦行先知', color: '白', kind: 'character', cost: 3, loyalty: ['白色', '白色'], magic: '神圣', unique: true, subtypes: ['人类', '僧侣'], defense: 2, permanentIcons: { investigation: 1, combat: 0, influence: 1 }, temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, keywords: [], deckCopyLimit: 3, abilities: [{ key: 'wound-draw-two', timing: 'fast', costs: [], triggered: true }] });
  const hash = '46bcfd890d040fc0b9e558ba601867f402bcc1443851e0ccf7f7a9e1579154b6';
  const scan = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8')).LC06;
  expect(scan).toEqual({ url: '/cards/LC06.jpg', source: 'resource/ymsj-fun.github.io/cards/LC06 瓦尔德修士.jpg', sha256: hash });
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/LC06.jpg'))).digest('hex')).toBe(hash);
  const m = render(<ReadModal card={card} definition={definition} viewerId="p2" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: '瓦尔德修士原始牌面' })).toHaveAttribute('src', '/cards/LC06.jpg'); m.unmount();
});
it.each(['p0', 'p1', 'p2', 'p3'])('shows the public wounded character to %s without mixing damage or enabling a manual trigger', you => {
  const m = render(<CardTile card={card} definition={definition} viewerId={you} />);
  expect(screen.getByText('创伤 1')).toBeInTheDocument();
  expect(screen.getByText('当前防御 1 · 印刷防御 2')).toBeInTheDocument();
  expect(m.container.querySelector('.hg-damage-marker')).toBeNull();
  expect(screen.queryByRole('button', { name: '瓦尔德修士：受到创伤触发' })).not.toBeInTheDocument(); m.unmount();
});
const players = [0,1,2,3].map(s => ({...testView.players[0],id:`p${s}`,seat:s,name:`玩家${s}`,team:Math.floor(s/2)}));
const choice = { id: 'wound-trigger-choice', kind: 'trigger', title: '瓦尔德修士：受到创伤触发', description: '选择是否发动', playerId: 'p2', min: 0, max: 1, allowDecline: true, options: [{ id: 'accept', label: '发动触发能力' }] };
const action = { id: 'choose-lc06-trigger', kind: 'choose', choiceId: choice.id, label: '确认选择' };
const base = { ...testView, mode: 'teams', status: 'playing', players, hand: [], regions: [], legalActions: [] };
it('submits the optional source-controller trigger acceptance and decline using server choice IDs', () => {
  const submit = vi.fn(); const m = render(<Table view={{...base,you:'p2',pendingChoice:choice,legalActions:[action]}} catalog={catalog} busy={false} onAction={submit} />);
  fireEvent.click(m.container.querySelector('[data-choice-option="accept"]'));
  fireEvent.click(screen.getByRole('button', {name:'确认选择'}));
  expect(submit).toHaveBeenLastCalledWith({...action,choiceId:choice.id,selected:['accept']});
  fireEvent.click(screen.getByRole('button', {name:'跳过此选择'}));
  expect(submit).toHaveBeenLastCalledWith({...action,choiceId:choice.id,selected:[]}); m.unmount();
});
it.each(['p0','p1','p3'])('keeps the source controller choice private from %s including the owner', you => {
  const submit=vi.fn();const m=render(<Table view={{...base,you,pendingChoice:null,waitingChoice:{playerId:'p2',title:choice.title,kind:'trigger'}}} catalog={catalog} busy={false} onAction={submit}/>);
  expect(m.container.querySelector('[data-choice-option]')).toBeNull(); expect(screen.queryByRole('button',{name:'确认选择'})).not.toBeInTheDocument(); expect(submit).not.toHaveBeenCalled(); m.unmount();
});
