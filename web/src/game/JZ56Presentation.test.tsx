import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, expect, it } from 'vitest';
import { CardTile, visibleCard } from './CardTile';
import { testCard } from './testFixtures';
import type { Card } from './types';

afterEach(cleanup);
const night: Card = { ...testCard, instanceId: 'night', cardId: 'JZ56', name: '亡者之夜',
  kind: 'attachment', faceDown: false, region: undefined, chosenSubtype: '行尸' };

it.each(['食尸鬼', '行尸', '骷髅'])('shows the public chosen subtype %s on the attachment face', chosenSubtype => {
  render(<CardTile card={{ ...night, chosenSubtype }} viewerId="p1" />);
  expect(screen.getByText(`选择类别：${chosenSubtype}`)).toBeInTheDocument();
});

it.each(['asset', 'sealed', 'hidden'])('strips the choice from a blank or hidden %s projection for every seat', kind => {
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    const card = visibleCard({ ...night, kind, faceDown: kind === 'hidden' }, viewer);
    expect(card.chosenSubtype).toBeUndefined();
  }
});

it('shows continuous wound separately from kill for all four seats', () => {
  for (const viewerId of ['p0', 'p1', 'p2', 'p3']) {
    const view = render(<CardTile card={{ ...testCard, kind: 'character', faceDown: false, region: 0, currentWound: 1 }} viewerId={viewerId} />);
    expect(screen.getByText('创伤 1')).toBeInTheDocument();
    expect(screen.queryByText('本回合杀伤 1')).not.toBeInTheDocument();
    view.unmount();
  }
});

it.each(['asset', 'sealed', 'hidden'])('strips wound from the %s face for every seat', kind => {
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    const card = visibleCard({ ...testCard, currentWound: 1, kind, faceDown: kind === 'hidden' }, viewer);
    expect(card.currentWound).toBeUndefined();
  }
});
