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

const originals = [
  { id: 'JZ48', name: '街头劫匪', cost: 1, subtypes: ['人类', '罪犯'], magic: '', hash: 'a9a5da75938c22872cca524776356b10f2a0250370d3ba3ed3b31deb332e5b4f', text: '持续：若本地区有其他本方罪犯角色，则街头劫匪获得永久势力1和防御力+1。' },
  { id: 'JZ50', name: '墓穴食尸鬼', cost: 2, subtypes: ['不死生物', '食尸鬼'], magic: '死亡', hash: '403e44bb5f7f0ff8715acb267a8369c3715b2dc752d7fe612de085da29b523e7', text: '现身触发：从你的牌库中寻找一张死亡领域的角色牌，将该牌置于你的墓地，然后洗牌。' },
];

describe.each(originals)('$id original card-face candidate', original => {
  const definition = { id: original.id, name: original.name, kind: 'character', cost: original.cost, color: '黑', subtypes: original.subtypes, loyalty: ['黑色'], loyaltyText: '黑色忠诚 1', magic: original.magic, text: original.text, permanentIcons: { investigation: 0, combat: 1, influence: 0 }, temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, defense: 1 };
  const card = { ...testCard, instanceId: 'LOCAL-' + original.id, cardId: original.id, name: original.name, cost: original.cost, color: '黑', magic: original.magic, text: original.text, icons: definition.permanentIcons, owner: 'p1', controller: 'p0' };

  it('binds the exact canonical original without populating a supplied gameplay catalog', () => {
    const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));
    expect(scans[original.id]).toEqual({ url: '/cards/' + original.id + '.jpg', source: 'resource/ymsj-fun.github.io/cards/' + original.id + ' ' + original.name + '.jpg', sha256: original.hash });
    expect(cardScanUrl(original.id)).toBe('/cards/' + original.id + '.jpg');
    expect(createHash('sha256').update(readFileSync(resolve('public' + scans[original.id].url))).digest('hex')).toBe(original.hash);
    render(<DeckLibrary catalog={testCatalog} />);
    fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: original.id } });
    expect(screen.getByText('当前开放目录没有符合筛选的卡牌。')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '添加 ' + original.name + '（' + original.id + '）' })).not.toBeInTheDocument();
  });

  it('reads the pinned printed fields and original while retaining asset resource presentation', () => {
    const reader = render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()} />);
    const dialog = screen.getByRole('dialog', { name: '放大阅读' + original.name });
    expect(dialog).toHaveTextContent(original.text);
    expect(dialog).toHaveTextContent('黑色忠诚 1');
    expect(dialog).toHaveTextContent(original.subtypes.join(' · '));
    expect(dialog).toHaveTextContent('防御 1');
    expect(screen.getByLabelText('费用 ' + original.cost)).toBeInTheDocument();
    expect(screen.getByLabelText('图标：调查0，战斗1，势力0')).toBeInTheDocument();
    expect(screen.queryByLabelText('先手图标：调查0，战斗0，势力0')).not.toBeInTheDocument();
    if (original.magic) expect(dialog).toHaveTextContent('死亡');
    else expect(dialog).not.toHaveTextContent('死亡');
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: original.name + '原始牌面' })).toHaveAttribute('src', '/cards/' + original.id + '.jpg');
    expect(screen.getByRole('link', { name: '打开' + original.name + '原始牌面全图' })).toHaveAttribute('href', '/cards/' + original.id + '.jpg');
    reader.unmount();
    const tile = render(<ArchivePresentation.Provider value><CardTile card={card} compact viewerId="p0" /></ArchivePresentation.Provider>);
    expect(tile.container.querySelector('img')).toHaveAttribute('src', '/cards/' + original.id + '.jpg');
    tile.rerender(<ArchivePresentation.Provider value><CardTile card={{ ...card, kind: 'asset' }} compact viewerId="p0" /></ArchivePresentation.Provider>);
    expect(tile.container.querySelector('img')).toBeNull();
    expect(tile.container.querySelector('[data-art-card]')).toBeNull();
  });

  it.each(['p0', 'p1', 'p2', 'p3'])('keeps concealed compact print anonymous for seat %s', viewerId => {
    const { container } = render(<ArchivePresentation.Provider value><CardTile card={{ ...card, faceDown: true }} definition={definition} compact viewerId={viewerId} /></ArchivePresentation.Provider>);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('[data-art-card]')).toBeNull();
    expect(screen.queryByText(original.name)).not.toBeInTheDocument();
    expect(container).not.toHaveTextContent(original.text);
  });

  it.each(['p1', 'p2', 'p3'])('withholds concealed print from noncontroller seat %s, including the owner', viewerId => {
    render(<ReadModal card={{ ...card, faceDown: true }} definition={definition} viewerId={viewerId} onClose={vi.fn()} />);
    const dialog = screen.getByRole('dialog', { name: '放大阅读暗藏者' });
    expect(dialog).not.toHaveTextContent(original.text);
    expect(screen.queryByRole('button', { name: '原始牌面' })).not.toBeInTheDocument();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
  });

  it('authorizes concealed reading by the current controller when owner differs', () => {
    render(<ReadModal card={{ ...card, faceDown: true }} definition={definition} viewerId="p0" onClose={vi.fn()} />);
    expect(screen.getByRole('dialog', { name: '放大阅读' + original.name })).toHaveTextContent(original.text);
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: original.name + '原始牌面' })).toHaveAttribute('src', '/cards/' + original.id + '.jpg');
  });
});
