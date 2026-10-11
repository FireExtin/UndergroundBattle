import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { ChoicePanel } from './ChoicePanel';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { cardScanUrl } from './cardScans';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const fixtures = JSON.parse(readFileSync(resolve('src/game/sevenCardUIV062.fixture.json'), 'utf8'));
const definitions = new Map(catalog.cards.map(card => [card.id, card]));
const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));

it.each(['JZ30', 'BQ028', 'BQ040', 'WM059', 'BQ078', 'JZ44', 'JZ45', 'XQ18'])('reads the admitted %s original and preserves its reviewed source bytes', id => {
  const definition = definitions.get(id); expect(definition).toBeDefined();
  const registered = scans[id]; expect(cardScanUrl(id)).toBe(registered.url);
  expect(createHash('sha256').update(readFileSync(resolve('public' + registered.url))).digest('hex')).toBe(registered.sha256);
  render(<ReadModal card={{ ...definition, cardId: id, instanceId: `read-${id}`, owner: 'p0', controller: 'p0' }} definition={definition} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${definition.name}原始牌面` })).toHaveAttribute('src', registered.url);
});
it('uses engine60/pool55 and admits the reviewed XQ18', () => {
  expect(catalog.engineVersion).toBe('rust-v0.2.62-fixed-empty-slots-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.55-xq18-carrier-candidate');
  expect(catalog.cards).toHaveLength(128); expect(definitions.has('XQ18')).toBe(true);
});
it('rejects the preserved engine57 opaque room without upgrading its state', () => {
  const previous = JSON.parse(readFileSync(resolve('src/game/xq37Native57Test.fixture.json'), 'utf8')).accept;
  expect(() => kernel.view(previous.state, 0)).toThrow();
  expect(() => kernel.applyRoom(previous.state, previous.seat, JSON.stringify(previous.command), '0')).toThrow();
});
it.each(fixtures)('submits the exact inspected-hand choice for seat=$targetSeat attachment=$attachment discard=$discard and matches Native', fixture => {
  const view = JSON.parse(kernel.view(fixture.before, 0)); expect(view).toEqual(fixture.views[0]);
  const choice = view.pendingChoice; expect(choice.kind).toBe('bq028-hand-inspect');
  expect(choice.min).toBe(0); expect(choice.max).toBe(fixture.attachment ? 1 : 0);
  const action = view.legalActions.find(item => item.kind === 'choose'); const submitted = vi.fn();
  render(<ChoicePanel choice={choice} action={action} definitions={definitions} busy={false} onSubmit={submitted} viewerId="p0" />);
  expect(screen.getByRole('heading', { name: '你检视的手牌' })).toBeInTheDocument();
  expect(screen.queryByText('你查看的对手手牌')).not.toBeInTheDocument();
  if (fixture.targetSeat === 2) {
    expect(view.mode).toBe('teams');
    expect(choice.previewCards.every(card => card.owner === 'p2')).toBe(true);
  }
  expect(screen.queryByText('你查看的牌库顶牌')).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: '不弃牌并确认' })).toBeEnabled();
  expect(screen.getByRole('button', { name: '确认选择' })).toBeEnabled();
  expect(choice.options.every(option => option.card.kind === 'attachment')).toBe(true);
  if (fixture.discard) {
    fireEvent.click(document.querySelector(`[data-choice-option="${choice.options[0].id}"]`));
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  } else fireEvent.click(screen.getByRole('button', { name: '不弃牌并确认' }));
  expect(submitted).toHaveBeenCalledOnce();
  const command = { ...fixture.command, action: actionForRoom(view, submitted.mock.calls[0][0]) };
  const transition = JSON.parse(kernel.applyRoom(fixture.before, 0, JSON.stringify(command), '1000'));
  expect(transition.outcome).toBe('accepted'); expect(transition.state).toBe(fixture.after);
  for (let seat = 0; seat < 4; seat++) expect(JSON.parse(kernel.view(transition.state, seat))).toEqual(fixture.afterViews[seat]);
});
it.each([1, 2, 3])('withholds the inspected hand from real private view seat %i', seat => {
  const fixture = fixtures[1]; const view = JSON.parse(kernel.view(fixture.before, seat));
  expect(view).toEqual(fixture.views[seat]); expect(view.pendingChoice).toBeNull();
  const visible = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(visible.container.querySelector('[data-choice-id]')).toBeNull();
  expect(screen.queryByText('你检视的手牌')).not.toBeInTheDocument();
});
it('closes an actor hand reader when the inspected choice finishes', () => {
  const fixture = fixtures[1]; const view = JSON.parse(kernel.view(fixture.before, 0));
  const shown = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
  const choice = screen.getByRole('dialog', { name: '待完成的选择' });
  fireEvent.click(within(choice).getByRole('button', { name: '放大阅读无知路人' }));
  expect(screen.getByRole('dialog', { name: '放大阅读无知路人' })).toBeInTheDocument();
  shown.rerender(<Table view={fixture.afterViews[0]} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.queryByRole('dialog', { name: '放大阅读无知路人' })).not.toBeInTheDocument();
});
it('drops the inspected-hand reader when the viewer changes to an unauthorized seat', () => {
  const fixture = fixtures[1];
  const shown = render(<Table view={fixture.views[0]} catalog={catalog} busy={false} onAction={vi.fn()} />);
  fireEvent.click(within(screen.getByRole('dialog', { name: '待完成的选择' })).getByRole('button', { name: '放大阅读无知路人' }));
  expect(screen.getByRole('dialog', { name: '放大阅读无知路人' })).toBeInTheDocument();
  shown.rerender(<Table view={fixture.views[1]} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.queryByRole('dialog', { name: '放大阅读无知路人' })).not.toBeInTheDocument();
  expect(screen.queryByText('你检视的手牌')).not.toBeInTheDocument();
});
