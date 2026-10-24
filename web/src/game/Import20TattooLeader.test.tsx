import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CardTile, visibleCard } from './CardTile';
import type { Card } from './types';

const card: Card = {
  instanceId: 'own-human', cardId: 'JC125', name: '无知路人', kind: 'character', owner: 'p0', controller: 'p0',
  exhausted: false, faceDown: false, region: 0, currentLeader: true, currentRenown: true,
  currentSubtypes: ['人类', '罪犯'], icons: { investigation: 1, combat: 2, influence: 1 }, defense: 3,
};
describe('server-derived imported status display', () => {
  it('shows a current Leader grant and removes it when the server grant ends', () => {
    const view = render(<CardTile card={card} viewerId="p0" />);
    expect(screen.getByText('领袖')).toBeInTheDocument();
    expect(screen.getByText('声望')).toBeInTheDocument();
    view.rerender(<CardTile card={{ ...card, currentLeader: undefined, currentRenown: undefined }} viewerId="p0" />);
    expect(screen.queryByText('领袖')).not.toBeInTheDocument();
    expect(screen.queryByText('声望')).not.toBeInTheDocument();
  });
  it('strips Leader from every hidden, asset or sealed display projection', () => {
    for (const viewer of ['p0', 'p1', 'p2', 'p3']) expect(visibleCard({ ...card, faceDown: true }, viewer).currentLeader).toBeUndefined();
    for (const kind of ['asset', 'sealed']) expect(visibleCard({ ...card, kind }, 'p0').currentLeader).toBeUndefined();
  });
  it('uses the conditional retreat label for server-approved Carmen state', () => {
    render(<CardTile card={{ ...card, cardId: 'CQ24', name: '卡门', currentLeader: undefined, currentRenown: undefined, currentRetreat: true }} viewerId="p0" />);
    expect(screen.getByText('隐秘撤回')).toBeInTheDocument();
    expect(screen.queryByText('本回合撤回')).not.toBeInTheDocument();
  });
});
