import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ArchivePresentation } from './ArchivePresentation';
import { CardTile } from './CardTile';
import { ReadModal } from './ReadModal';
import { testCard, testCatalog } from './testFixtures';

describe('archival table artwork authorization', () => {
  it('keeps components outside the table presentation unchanged', () => {
    const { container } = render(<CardTile card={testCard} compact viewerId="p0" />);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-card-color]')).toBeNull();
  });

  it('uses a resource token for assets even if stale input retains an underlying card ID', () => {
    const { container } = render(<ArchivePresentation.Provider value><CardTile card={{ ...testCard, kind: 'asset' }} compact viewerId="p0" /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-art-card]')).toBeNull();
    expect(container.querySelector('.hg-archive-asset-art')).not.toBeNull();
  });

  it('uses the exact admitted card ID, falls back on load failure, and resets on a replacement card', () => {
    const content = (cardId: string) => <ArchivePresentation.Provider value><CardTile card={{ ...testCard, cardId }} viewerId="p0" compact /></ArchivePresentation.Provider>;
    const { container, rerender } = render(content('LC24'));
    expect(container.querySelector('img')).toHaveAttribute('src', '/cards/LC24.jpg');
    fireEvent.error(container.querySelector('img')!);
    expect(container.querySelector('img')).toBeNull();
    rerender(content('JC058'));
    expect(container.querySelector('img')).toHaveAttribute('src', '/cards/JC058.jpg');
    rerender(content('NOT-ADMITTED'));
    expect(container.querySelector('img')).toBeNull();
  });

  it.each(['p0', 'p1', 'p2', 'p3'])('shows only a back on concealed thumbnails for %s, including stale owner-visible print', viewerId => {
    const card = { ...testCard, cardId: 'LC22', name: '退役军人', kind: 'hidden', owner: 'p1', controller: 'p0', faceDown: true, color: '红' };
    const { container } = render(<ArchivePresentation.Provider value><CardTile card={card} definition={testCatalog.cards[0]} viewerId={viewerId} compact /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-art-card]')).toBeNull();
    expect(container.querySelector('.hg-archive-back')).not.toBeNull();
    expect(container.querySelector('[data-card-color]')).toBeNull();
    expect(container.querySelector('.hg-card')).toHaveClass('hg-card-neutral');
    expect(screen.queryByText(card.name)).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: viewerId === card.controller ? `查看${card.name}` : '查看暗藏者' })).toBeInTheDocument();
  });

  it('preserves the explicit original-reader permission on current controller rather than owner', () => {
    const card = { ...testCard, cardId: 'LC22', name: '退役军人', owner: 'p1', controller: 'p0', faceDown: true };
    const content = (viewerId: string) => <ArchivePresentation.Provider value><ReadModal key={viewerId} card={card} viewerId={viewerId} onClose={vi.fn()} /></ArchivePresentation.Provider>;
    const { container, rerender } = render(content('p0'));
    expect(container.querySelector('img')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: '退役军人原始牌面' })).toHaveAttribute('src', '/cards/LC22.jpg');
    rerender(content('p1'));
    expect(container.querySelector('img')).toBeNull();
    expect(screen.queryByRole('button', { name: '原始牌面' })).not.toBeInTheDocument();
  });

  it('uses the registered MSJC09 original in the existing explicit reader', () => {
    render(<ArchivePresentation.Provider value><ReadModal card={{ ...testCard, cardId: 'MSJC09', name: '秘社', kind: 'society' }} viewerId="p0" onClose={vi.fn()} /></ArchivePresentation.Provider>);
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: '秘社原始牌面' })).toHaveAttribute('src', '/cards/MSJC09.jpg');
  });

  it('keeps distinct projected damage, wound and shield counts alongside attachment counts', () => {
    const card = { ...testCard, cardId: 'LC22', damage: 2, wounds: 1, shield: 3 };
    const { container } = render(<ArchivePresentation.Provider value><CardTile card={card} compact viewerId="p0" attachmentCount={2} /></ArchivePresentation.Provider>);
    expect(container.querySelector('.hg-damage-marker')).toHaveTextContent('伤害 2');
    expect(container.querySelector('.hg-wound-marker')).toHaveTextContent('创伤 1');
    expect(container.querySelector('.hg-shield-marker')).toHaveTextContent('护盾 3');
    expect(screen.getByLabelText('附属 2 张')).toBeInTheDocument();
  });
});
