import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CardContent } from './CardTile';
import { testCard, testCatalog } from './testFixtures';

describe('concealed character presentation', () => {
  it('shows damage immunity on own concealed board cards while preserving printed data in inspection', () => {
    const card = { ...testCard, kind: 'hidden', faceDown: true, damage: 2 };
    const definition = { ...testCatalog.cards[0], defense: 1 };
    const { rerender } = render(<CardContent card={card} definition={definition} compact />);
    expect(screen.getByText('暗藏者 · 不受角色伤害')).toBeInTheDocument();
    expect(screen.queryByText('防御 1')).not.toBeInTheDocument();
    expect(screen.queryByText('伤害 2')).not.toBeInTheDocument();
    rerender(<CardContent card={card} definition={definition} />);
    expect(screen.getByText('防御 1')).toBeInTheDocument();
    expect(screen.getByText('真实印刷文字')).toBeInTheDocument();
  });
});
