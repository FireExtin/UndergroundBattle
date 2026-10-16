// Current ABI78 projections and exact Native commands; component QA only.
import { render, screen, fireEvent, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { loadBq035Fixture, sha256 } from '../../../tools/bq035-oracle.mjs';
import { kernel as abi, moduleBytes, catalog } from '../../../rust-game-wasm/tests/current78-kernel.mjs';
import { Table } from './Table';
import { CardContent } from './CardTile';
import { actionForRoom } from './api';
const fixture = loadBq035Fixture();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);
expect(catalog).toEqual(fixture.catalog);
const choices = fixture.scenarios.flatMap(scenario => scenario.steps.flatMap((step, index) => {
  const views = index ? scenario.steps[index - 1].views : scenario.initial.views;
  const view = views[step.seat], action = step.command.action.action;
  return !step.expected.errorCode && action?.kind === 'choose' && view.pendingChoice?.kind === 'trigger'
    && view.pendingChoice.options.every(option => option.id.startsWith('region:'))
    ? [{ ...step, name: `${scenario.name}/${index}`, beforeViews: views }] : [];
}));
expect(choices.length).toBeGreaterThan(5);
beforeEach(() => sessionStorage.clear());
afterEach(() => sessionStorage.clear());
it.each(choices)('$name restores the real optional region choice and submits its exact Native command', row => {
  const view = row.beforeViews[row.seat], submit = vi.fn();
  expect(JSON.parse(abi.view(row.state, row.seat))).toEqual(view);
  const props = { catalog, view, busy: false, connection: 'online', onAction: submit };
  const first = render(<Table {...props} />);
  let dialog = screen.getByRole('dialog', { name: '待完成的选择' });
  expect(view.pendingChoice).toMatchObject({ min: 0, max: 1, allowDecline: true });
  const selected = row.command.action.action.selected;
  selected.forEach(id => fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`)));
  first.unmount();
  const fresh = render(<Table {...props} view={JSON.parse(abi.view(row.state, row.seat))} />);
  dialog = screen.getByRole('dialog', { name: '待完成的选择' });
  selected.forEach(id => expect(dialog.querySelector(`[data-choice-option="${id}"]`)).toHaveAttribute('aria-pressed', 'true'));
  fireEvent.click(within(dialog).getByRole('button', { name: selected.length ? '确认选择' : '跳过此选择' }));
  expect(submit).toHaveBeenCalledOnce();
  const action = JSON.parse(JSON.stringify(actionForRoom(view, submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const next = JSON.parse(abi.applyRoom(row.state, row.seat, JSON.stringify({ ...row.command, action }), row.serverNowMs));
  expect(next).toEqual(row.expected);
  row.views.forEach((expected, seat) => expect(JSON.parse(abi.view(next.state, seat))).toEqual(expected));
  fresh.rerender(<Table {...props} view={row.views[row.seat]} />);
  const nextChoice = row.views[row.seat].pendingChoice;
  if (nextChoice) {
    expect(nextChoice.id).not.toBe(view.pendingChoice.id);
    expect(screen.getByRole('dialog', { name: '待完成的选择' })).toBeInTheDocument();
  } else expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
});
it('only the frozen actor sees the real chooser; current granted mobility is public on the recipient', () => {
  const row = choices.find(row => row.beforeViews[row.seat].regions.some(region => region.characters.some(card => card.currentMobilitySource)));
  expect(row).toBeDefined();
  row.beforeViews.forEach((view, seat) => {
    const ui = render(<Table catalog={catalog} view={view} busy={false} connection="online" onAction={vi.fn()} />);
    expect(!!screen.queryByRole('dialog', { name: '待完成的选择' })).toBe(seat === row.seat);
    const cards = view.regions.flatMap(region => region.characters).filter(card => card.currentMobilitySource);
    expect(cards.length).toBeGreaterThan(0);
    ui.unmount();
    const content = render(<CardContent card={cards[0]} viewerId={view.you} />);
    expect(screen.getByText('机动 · 幽冥骑士授予')).toBeInTheDocument(); content.unmount();
  });
});
