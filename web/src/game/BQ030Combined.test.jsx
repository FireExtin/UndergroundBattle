// Actual current Native choices through React controls and the production WASM.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import fixtures from './bq030Native63.fixture.json';
import { Table } from './Table';

abi.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(abi.catalog());
afterEach(cleanup);

it.each(fixtures.choices)('$name: sends the unchanged Native choice and matches all real seat views', fixture => {
  const step = fixture.step, view = JSON.parse(abi.view(step.state, step.seat));
  expect(view).toEqual(step.beforeViews[step.seat]);
  expect(view.pendingChoice.allowDecline).toBe(true);
  let result;
  const { container } = render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={action => {
    const { id, label, ...payload } = action;
    expect(id).toBe(`choose:${view.pendingChoice.id}`);
    expect(label).toBe('确认选择');
    expect(payload).toEqual(step.command.action.action);
    result = JSON.parse(abi.applyRoom(step.state, step.seat, JSON.stringify({ ...step.command, action: { kind: 'game', action } }), '0'));
  }} />);
  const selected = step.command.action.action.selected;
  if (!selected.length) fireEvent.click(screen.getByRole('button', { name: '跳过此选择' }));
  else {
    const option = [...container.querySelectorAll('[data-choice-option]')]
      .find(element => element.getAttribute('data-choice-option') === selected[0]);
    expect(option).toBeDefined(); fireEvent.click(option);
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  }
  expect(result).toEqual(step.expected);
  for (let seat = 0; seat < step.views.length; seat++) expect(JSON.parse(abi.view(result.state, seat))).toEqual(step.views[seat]);
});
