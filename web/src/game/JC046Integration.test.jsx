// Current source-built ABI77 only. Prepared Native layouts are not live gameplay.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { Table } from './Table';

const bytes = readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({ module: bytes });
const catalog = JSON.parse(abi.catalog());
const fixture = JSON.parse(readFileSync(resolve('src/game/jc046Native77.fixture.json'), 'utf8'));
const props = { catalog, busy: false, connection: 'online' };
const scenario = name => fixture.scenarios.find(row => row.name === name);
const acceptedChoices = name => scenario(name).steps.filter(step =>
  step.command.action.action?.kind === 'choose' && !step.expected.errorCode);

beforeEach(() => sessionStorage.clear());
afterEach(() => { sessionStorage.clear(); vi.useRealTimers(); });

it('binds the integrated chooser to the exact source-built JC046 ABI77 and unchanged Native traces', () => {
  expect(catalog).toEqual(fixture.catalog);
  expect(catalog.engineVersion).toBe('rust-v0.2.77-jc046-mandatory-sacrifice-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.70-jc046-mandatory-sacrifice-candidate');
  expect(createHash('sha256').update(bytes).digest('hex')).toBe(fixture.wasmSha256);
  expect(fixture.scenarios).toHaveLength(37);
});

it.each([
  'paid-seat0', 'self-sacrifice-continues', 'prepared-leader-paused-continuation',
  'two-real-reveals-independent-frames',
])('%s preserves each mandatory draft and exact command across a fresh table mount', name => {
  const choices = acceptedChoices(name);
  expect(choices.length).toBeGreaterThan(1);
  for (const step of choices) {
    const view = JSON.parse(abi.view(step.state, step.seat));
    const choice = view.pendingChoice;
    expect(choice).toMatchObject({ min: 1, max: 1, allowDecline: false });
    const submit = vi.fn();
    let ui = render(<Table {...props} view={view} onAction={submit} />);
    let dialog = screen.getByRole('dialog', { name: '待完成的选择' });
    expect(within(dialog).queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
    expect(within(dialog).getByRole('button', { name: '确认选择' })).toBeDisabled();
    const wanted = step.command.action.action.selected;
    const option = id => Array.from(dialog.querySelectorAll('[data-choice-option]'))
      .find(button => button.getAttribute('data-choice-option') === id);
    const alternative = choice.options.find(item => !wanted.includes(item.id));
    if (alternative) fireEvent.click(option(alternative.id));
    wanted.forEach(id => fireEvent.click(option(id)));
    if (alternative) expect(option(alternative.id)).toHaveAttribute('aria-pressed', 'false');
    ui.unmount();
    ui = render(<Table {...props} view={JSON.parse(abi.view(step.state, step.seat))} onAction={submit} />);
    dialog = screen.getByRole('dialog', { name: '待完成的选择' });
    wanted.forEach(id => expect(option(id)).toHaveAttribute('aria-pressed', 'true'));
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledOnce();
    const action = JSON.parse(JSON.stringify(actionForRoom(view, submit.mock.calls[0][0])));
    expect(action).toEqual(step.command.action);
    const actual = JSON.parse(abi.applyRoom(step.state, step.seat,
      JSON.stringify({ ...step.command, action }), step.serverNowMs));
    expect(actual).toEqual(step.expected);
    step.views.forEach((expected, seat) => expect(JSON.parse(abi.view(actual.state, seat))).toEqual(expected));
    ui.rerender(<Table {...props} view={step.views[step.seat]} onAction={submit} />);
    expect(submit).toHaveBeenCalledOnce();
    ui.unmount();
  }
});

it('requires all four real choosers to act while other seats and automatic pass leave the choice untouched', () => {
  const choices = acceptedChoices('paid-seat0');
  expect(choices.map(step => step.seat)).toEqual([0, 1, 2, 3]);
  vi.useFakeTimers();
  for (const step of choices) {
    for (let seat = 0; seat < 4; seat++) {
      const view = JSON.parse(abi.view(step.state, seat));
      const submit = vi.fn();
      const ui = render(<Table {...props} view={view} onAction={submit} autoPassEnabled />);
      expect(!!screen.queryByRole('dialog', { name: '待完成的选择' })).toBe(seat === step.seat);
      if (seat !== step.seat) expect(ui.container.querySelector('[data-choice-option]')).toBeNull();
      else {
        expect(screen.queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
        expect(screen.getByRole('button', { name: '确认选择' })).toBeDisabled();
      }
      act(() => vi.advanceTimersByTime(60_000));
      expect(submit).not.toHaveBeenCalled();
      ui.unmount();
    }
  }
});
