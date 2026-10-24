import { render, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';

describe('server-authored step resources', () => {
  it('shows temporary money separately from real assets and updates when it expires', () => {
    const assets = [
      { ...testCard, instanceId: 'asset-ready', controller: 'p0', exhausted: false },
      { ...testCard, instanceId: 'asset-tapped', controller: 'p0', exhausted: true },
    ];
    const makeView = (n: number) => ({ ...testView, status: 'playing', assets, players: testView.players.map(p => ({ ...p, stepResources: p.id === 'p0' ? n : 0 })) });
    const ui = render(<Table view={makeView(2)} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    const mat = () => ui.container.querySelector('[data-player-zone="p0"]')!;
    expect(mat().querySelector('[data-step-resources="2"]')).toHaveTextContent('费用 3');
    expect(mat()).toHaveTextContent('资产 1 / 2 · 本步骤费用 +2');
    ui.rerender(<Table view={makeView(0)} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(mat().querySelector('[data-step-resources="0"]')).toHaveTextContent('费用 1 / 2');
    expect(within(mat() as HTMLElement).queryByText(/本步骤费用/)).not.toBeInTheDocument();
  });

  it('does not derive other players temporary money from card names or grant them the local amount', () => {
    const own = { ...testView.players[0], id: 'p0', stepResources: 1 };
    const other = { ...own, id: 'p2', seat: 2, team: 1, name: '对手', stepResources: 0 };
    const { container } = render(<Table view={{ ...testView, status: 'playing', assets: [], players: [own, other] }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(container.querySelector('[data-player-zone="p0"] [data-step-resources="1"]')).toHaveTextContent('费用 1');
    expect(container.querySelector('[data-player-zone="p2"] [data-step-resources="0"]')).toHaveTextContent('费用 0 / 0');
  });
});
