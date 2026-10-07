import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ArchivePresentation } from './ArchivePresentation';
import { CardTile } from './CardTile';
import { DeckLibrary } from './DeckLibraryPanel';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
import { testCard, testCatalog } from './testFixtures';

const scanHash = '208b2370433fb420ba91e1557ad6633589e43b1391d2008d63ebee2ed90fdf62';
const card = { ...testCard, cardId: 'JZ49', name: '蹒跚行尸', owner: 'p1', controller: 'p0' };

describe('JZ49 original card-face candidate', () => {
  it('binds the exact canonical original and preserves every previous scan', () => {
    const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));
    const previous = JSON.parse(readFileSync(resolve('src/game/cardScansV035.fixture.json'), 'utf8'));
    for (const [id, scan] of Object.entries(previous)) expect(scans[id]).toEqual(scan);
    expect(scans.JZ55.sha256).toBe('9155faa90be22aa66e56148dfacd0d517eb7b6f7b56fd1b45b71a6804719784a');
    expect(scans.JZ49).toEqual({ url: '/cards/JZ49.jpg', source: 'resource/ymsj-fun.github.io/cards/JZ49 蹒跚行尸.jpg', sha256: scanHash });
    expect(cardScanUrl('JZ49')).toBe('/cards/JZ49.jpg');
    for (const scan of Object.values(scans)) {
      expect(createHash('sha256').update(readFileSync(resolve('public' + scan.url))).digest('hex')).toBe(scan.sha256);
    }
  });

  it('makes the original readable while catalog selection still follows supplied gameplay definitions', () => {
    const library = render(<DeckLibrary catalog={testCatalog} />);
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: 'JZ49' } });
    expect(screen.getByText('当前开放目录没有符合筛选的卡牌。')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '添加 蹒跚行尸（JZ49）' })).not.toBeInTheDocument();
    library.unmount();
    render(<ReadModal card={card} viewerId="p0" onClose={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: '蹒跚行尸原始牌面' })).toHaveAttribute('src', '/cards/JZ49.jpg');
    expect(screen.getByRole('link', { name: '打开蹒跚行尸原始牌面全图' })).toHaveAttribute('href', '/cards/JZ49.jpg');
  });

  it('uses the original thumbnail for a visible card and keeps an asset resource token', () => {
    const { container, rerender } = render(<ArchivePresentation.Provider value><CardTile card={card} compact viewerId="p0" /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toHaveAttribute('src', '/cards/JZ49.jpg');
    rerender(<ArchivePresentation.Provider value><CardTile card={{ ...card, kind: 'asset' }} compact viewerId="p0" /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-art-card]')).toBeNull();
  });

  it.each(['p0', 'p1', 'p2', 'p3'])('keeps concealed thumbnails anonymous for seat %s', viewerId => {
    const { container } = render(<ArchivePresentation.Provider value><CardTile card={{ ...card, faceDown: true }} compact viewerId={viewerId} /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-art-card]')).toBeNull();
    expect(screen.queryByText('蹒跚行尸')).not.toBeInTheDocument();
  });

  it.each(['p1', 'p2', 'p3'])('withholds the original reader from noncontroller seat %s', viewerId => {
    render(<ReadModal card={{ ...card, faceDown: true }} viewerId={viewerId} onClose={vi.fn()} />);
    expect(screen.getByRole('dialog', { name: '放大阅读暗藏者' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '原始牌面' })).not.toBeInTheDocument();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
  });
});
