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
