import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { Table } from './Table';

const wasm = readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
const fixture = JSON.parse(readFileSync(resolve('src/game/currentLifecycle74.fixture.json'), 'utf8'));
expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
abi.initSync({ module: wasm });
const catalog = JSON.parse(abi.catalog());
const props = { catalog, busy: false, connection: 'online' };

function before(row) {
  const views = row.beforeViews.map((view, seat) => {
    expect(JSON.parse(abi.view(row.before, seat))).toEqual(view);
    return view;
  });
  return views[row.seat];
}
function replay(row, view, submitted) {
  expect(submitted).toHaveBeenCalledOnce();
  const action = JSON.parse(JSON.stringify(actionForRoom(view, submitted.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result = JSON.parse(abi.applyRoom(row.before, row.seat, JSON.stringify({ ...row.command, action }), '0'));
  expect(result).toEqual(row.expected);
  expect(result.state).toBe(row.expected.state); // Never parse/reserialize an opaque u64 state.
  row.afterViews.forEach((expected, seat) => expect(JSON.parse(abi.view(result.state, seat))).toEqual(expected));
  return result;
}

it.each(fixture.cases.filter(row => row.color !== 'red'))('$name restores the current private chooser and sends the exact Native selection', row => {
  const view = before(row), submit = vi.fn();
  const first = render(<Table {...props} view={view} onAction={submit} />);
  let dialog = screen.getByRole('dialog', { name: '待完成的选择' });
  const choice = view.pendingChoice;
  expect(within(dialog).queryByRole('button', { name: '跳过此选择' })).not.toBeInTheDocument();
  if (choice.previewCards?.length) {
    const read = within(dialog).getAllByRole('button', { name: /^放大阅读/ })[0];
    fireEvent.click(read);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '关闭放大阅读' }));
  }
  first.unmount(); // A fresh component models restored projection display, not browser seat persistence.
  const fresh = render(<Table {...props} view={JSON.parse(abi.view(row.before, row.seat))} onAction={submit} />);
  dialog = screen.getByRole('dialog', { name: '待完成的选择' });
  const confirm = within(dialog).getByRole('button', { name: '确认选择' });
  const selected = row.command.action.action.selected;
  expect(confirm.disabled).toBe(selected.length > 0);
  selected.forEach(id => fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`)));
  expect(confirm).toBeEnabled(); fireEvent.click(confirm);
  replay(row, view, submit);
  fresh.rerender(<Table {...props} view={row.afterViews[row.seat]} onAction={submit} />);
  expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
});

it.each(fixture.cases.filter(row => row.color === 'red'))('$name preserves private forecast order across a fresh mount and matches Native', row => {
  const view = before(row), submit = vi.fn();
  const first = render(<Table {...props} view={view} onAction={submit} />);
  expect(screen.getByRole('dialog', { name: '待完成的选择' })).toBeInTheDocument();
  first.unmount();
  const fresh = render(<Table {...props} view={JSON.parse(abi.view(row.before, row.seat))} onAction={submit} />);
  const dialog = screen.getByRole('dialog', { name: '待完成的选择' });
  for (const id of row.command.action.action.bottom) {
    const label = view.pendingChoice.options.find(o => o.id === id).label;
    const li = [...dialog.querySelectorAll('.hg-order-columns > div:first-child li')].find(el => el.querySelector('strong')?.textContent === label);
    fireEvent.click(within(li).getByRole('button', { name: '置于底' }));
  }
  fireEvent.click(within(dialog).getByRole('button', { name: '确认选择' }));
  replay(row, view, submit);
  fresh.rerender(<Table {...props} view={row.afterViews[row.seat]} onAction={submit} />);
  expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
});

it.each(fixture.cases)('$name grants only the real chooser a modal and never acts after waiting', row => {
  vi.useFakeTimers();
  try {
    row.beforeViews.forEach((view, seat) => {
      const submit = vi.fn();
      const ui = render(<Table {...props} view={view} onAction={submit} autoPassEnabled />);
      expect(!!screen.queryByRole('dialog', { name: '待完成的选择' })).toBe(seat === row.seat);
      expect(view.pendingChoice !== null).toBe(seat === row.seat);
      if (seat !== row.seat && row.color !== 'black') {
        const actorChoice = row.beforeViews[row.seat].pendingChoice;
        for (const option of actorChoice.previewCards || actorChoice.options.map(o => o.card)) {
          expect(JSON.stringify(view)).not.toContain(option.instanceId);
        }
      }
      act(() => vi.advanceTimersByTime(60_000));
      expect(submit).not.toHaveBeenCalled(); ui.unmount();
    });
  } finally { vi.useRealTimers(); }
});
