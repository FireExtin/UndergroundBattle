// Prepared Native layouts followed by actual Room commands; no browser acceptance.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { ChoicePanel } from './ChoicePanel';
import { Table } from './Table';
import native from './xq18Native63Test.fixture.json';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const definitions = new Map(catalog.cards.map(card => [card.id, card]));

it.each(native.carriers)('distinguishes equal-name equal-owner carriers and submits Native selected=$selected', fixture => {
  const view = JSON.parse(kernel.view(fixture.state, fixture.seat));
  expect(view).toEqual(fixture.beforeViews[fixture.seat]);
  const choice = view.pendingChoice;
  expect(choice.kind).toBe('xq18-time-carrier');
  expect([choice.min, choice.max, choice.allowDecline]).toEqual([1, 1, false]);
  expect(choice.options).toHaveLength(2);
  expect(new Set(choice.options.map(option => option.label)).size).toBe(2);
  const submitted = vi.fn();
  const action = view.legalActions.find(item => item.kind === 'choose');
  render(<ChoicePanel choice={choice} action={action} definitions={definitions} busy={false} onSubmit={submitted} viewerId={view.viewerId} />);
  const confirm = screen.getByRole('button', { name: '确认选择' });
  expect(confirm).toBeDisabled();
  expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /放大文字|放大阅读/ })).not.toBeInTheDocument();
  for (const option of choice.options) {
    expect(option).not.toHaveProperty('card');
    expect(option.label).toContain(`#${option.id}`);
    expect(option.label).toContain('1号席拥有');
    expect(option.label).toContain('3个时间标志');
    expect(document.querySelector(`[data-choice-option="${option.id}"]`)).toHaveTextContent(option.label);
  }
  fireEvent.click(document.querySelector(`[data-choice-option="${fixture.selected}"]`));
  expect(confirm).toBeEnabled();
  fireEvent.click(confirm);
  const command = { ...fixture.command, action: actionForRoom(view, submitted.mock.calls[0][0]) };
  expect(command).toEqual(fixture.command);
  const actual = JSON.parse(kernel.applyRoom(fixture.state, fixture.seat, JSON.stringify(command), '0'));
  expect(actual).toEqual(fixture.expected);
  for (let seat = 0; seat < 4; seat++) expect(JSON.parse(kernel.view(actual.state, seat))).toEqual(fixture.views[seat]);
});

it.each([1, 2, 3])('keeps the carrier choice private from seat %i', seat => {
  const fixture = native.carriers[0];
  const view = JSON.parse(kernel.view(fixture.state, seat));
  expect(view).toEqual(fixture.beforeViews[seat]);
  expect(view.pendingChoice).toBeNull();
  render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
  for (const option of fixture.beforeViews[0].pendingChoice.options) expect(screen.queryByText(option.label)).not.toBeInTheDocument();
});
