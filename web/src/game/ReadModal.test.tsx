import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ReadModal } from './ReadModal';
import { testCard } from './testFixtures';

describe('original card scan reading', () => {
  it('opens an admitted original face and keeps keyboard focus within the reader', () => {
    render(<ReadModal card={{ ...testCard, cardId: 'BQ022', name: '合金指虎', owner: 'p1', controller: 'p1' }} viewerId="p0" onClose={vi.fn()} />);
    fireEvent.keyDown(document, { key: 'Tab' });
    expect(screen.getByRole('button', { name: '当前状态与文字' })).toHaveFocus();
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: '合金指虎原始牌面' })).toHaveAttribute('src', '/cards/BQ022.jpg');
    expect(screen.getByRole('link', { name: '打开合金指虎原始牌面全图' })).toHaveAttribute('href', '/cards/BQ022.jpg');
  });

  it('does not request a face or expose the scan control for another player’s hidden card', () => {
    render(<ReadModal card={{ ...testCard, cardId: 'BQ022', name: '合金指虎', owner: 'p1', controller: 'p1', faceDown: true }} viewerId="p0" onClose={vi.fn()} />);
    expect(screen.getByRole('dialog', { name: '放大阅读暗藏者' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '原始牌面' })).not.toBeInTheDocument();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(screen.queryByText('合金指虎')).not.toBeInTheDocument();
  });
});
