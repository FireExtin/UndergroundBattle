// Actual Engine62 Native inputs and WASM projections, without edited view fields.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import fixtures from './fixedSlotsNative62.fixture.json';
import { Table } from './Table';

abi.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(abi.catalog());
afterEach(cleanup);

it.each(fixtures.captures)('$name: actual capture keeps every slot, closes the retired inspector and reads the fresh scored instance', fixture => {
  const step = fixture.step, seat = step.seat;
  const before = JSON.parse(abi.view(fixture.beforeState, seat));
  expect(before).toEqual(fixture.beforeViews[seat]);
  const result = JSON.parse(abi.applyRoom(fixture.beforeState, seat, JSON.stringify(step.command), step.serverNowMs));
  expect(result).toEqual(step.expected);
  const after = JSON.parse(abi.view(result.state, seat));
  expect(after).toEqual(step.views[seat]);
  const retired = before.regions[fixture.emptyIndex], submit = vi.fn();
  const { container, rerender } = render(<Table view={before} catalog={catalog} busy={false} connection="online" onAction={submit} />);
  fireEvent.click(screen.getByRole('button', { name: `查看地区${retired.index + 1} ${retired.name}` }));
  expect(screen.getByRole('complementary', { name: '选牌行动' })).toBeInTheDocument();
  rerender(<Table view={after} catalog={catalog} busy={false} connection="online" onAction={submit} />);
  const slots = within(container.querySelector('.hg-board')).getAllByRole('article');
  expect(slots.map(slot => slot.id)).toEqual(after.regions.map(r => `hg-region-${r.index}`));
  expect(slots).toHaveLength(after.mode === 'duel' ? 3 : 5);
  const empty = slots[fixture.emptyIndex];
  expect(empty).toHaveAccessibleName(`地区 ${fixture.emptyIndex + 1} · 空位`);
  expect(empty.querySelector('button,img,[data-card-instance]')).toBeNull();
  expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
  expect(after.legalActions.every(a => a.region !== fixture.emptyIndex)).toBe(true);
  fireEvent.click(empty); expect(submit).not.toHaveBeenCalled();
  const scored = after.scoreCards.find(c => c.cardId === retired.cardId);
  expect(scored).toBeDefined(); expect(scored.instanceId).not.toBe(retired.id);
  fireEvent.click(container.querySelector(`[data-card-instance="${scored.instanceId}"]`));
  fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
  expect(screen.getByRole('dialog', { name: /阅读/ })).toHaveTextContent(scored.name);
});

it('JC050 renders the exact sparse original indices and sends the real current choice', () => {
  const fixture = fixtures.sparseChoice, step = fixture.step, seat = step.seat;
  const view = JSON.parse(abi.view(step.state, seat));
  expect(view).toEqual(fixture.beforeViews[seat]);
  expect(view.pendingChoice.options.map(o => o.id)).toEqual(['region:0', 'region:2', 'region:3', 'region:4']);
  let result;
  const { container } = render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={action => {
    result = JSON.parse(abi.applyRoom(step.state, seat, JSON.stringify({ ...step.command, action: { kind: 'game', action } }), step.serverNowMs));
  }} />);
  const selected = step.command.action.action.selected[0];
  const options = [...container.querySelectorAll('[data-choice-option]')];
  expect(options.map(o => o.getAttribute('data-choice-option'))).toEqual(['region:0', 'region:2', 'region:3', 'region:4']);
  fireEvent.click(options.find(o => o.getAttribute('data-choice-option') === selected));
  fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  expect(result).toEqual(step.expected);
});
