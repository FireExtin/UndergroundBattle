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
  { id: 'JC089', name: '毒血诅咒', cost: 2, kind: 'attachment', magic: '鲜血', subtypes: ['诅咒'], loyalty: 1, defense: undefined, permanentIcons: { investigation: 0, combat: 0, influence: 0 }, temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, hash: '83853a1a4c3e6ecd9644f7bb5c843e70585d421f1792a0e4a141ebbe867d12f7', text: '结附于目标角色。持续：受此结附的角色获得防御-1、永久战斗1和临时战斗1，并获得威名。' },
  { id: 'JZ51', name: '温迪戈', cost: 5, subtypes: ['不死生物', '食尸鬼'], subtitle: undefined, loyalty: 1, defense: 3, permanentIcons: { investigation: 0, combat: 2, influence: 0 }, temporaryIcons: { investigation: 1, combat: 0, influence: 0 }, hash: '1b0311913bbf4a26c0e9e2dc2c0d6b656bd802cc6e831350f6f2500edf577dc0', text: '进场触发：从你牌库顶的八张牌中寻找任意数量名为无名尸体的牌，将这些牌放置于本地区，然后将其余的牌洗回牌库。墓地行动：将本方墓地中三张名为无名尸体的牌封印在本方秘社上：将温迪戈从墓地中放置进场。' },
  { id: 'JZ52', name: '大维齐尔迈哈穆德', cost: 7, subtypes: ['人类', '法师'], subtitle: '魔特之子', loyalty: 4, defense: 5, permanentIcons: { investigation: 0, combat: 2, influence: 2 }, temporaryIcons: { investigation: 3, combat: 0, influence: 0 }, hash: '7f9ad8adb68bdd84877ebdead2bc13ed04983099977333d2106486405c6478a3', text: '领袖。持续：你不能从手中正面打出角色牌。持续：你可以从墓地中正面打出死亡领域的角色牌，就如同其在你手中一样。' },
  { id: 'JZ53', name: '尸舞舒拉密兹', cost: 4, subtypes: ['人类', '法师'], subtitle: '死灵术士', loyalty: 2, defense: 1, permanentIcons: { investigation: 0, combat: 0, influence: 1 }, temporaryIcons: { investigation: 1, combat: 0, influence: 1 }, hash: 'a84d896c1b552dabda05a1fc12c55224275a31b63a8408446508775235cc4623', text: '进场触发：将你墓地中的目标死亡领域角色牌置于你的手中。触发：每当一张角色牌离开你的墓地时，在目标地区生成一个行尸角色指示物。' },
];

describe.each(originals)('$id original card-face candidate', original => {
  const definition = { id: original.id, name: original.name, kind: original.kind || 'character', cost: original.cost, color: '黑', subtypes: original.subtypes, subtitle: original.subtitle, loyalty: Array(original.loyalty).fill('黑色'), loyaltyText: '黑色忠诚 ' + original.loyalty, magic: original.magic || '死亡', text: original.text, permanentIcons: original.permanentIcons, temporaryIcons: original.temporaryIcons, defense: original.defense };
  const card = { ...testCard, instanceId: 'LOCAL-' + original.id, cardId: original.id, name: original.name, kind: original.kind || 'character', cost: original.cost, color: '黑', magic: original.magic || '死亡', text: original.text, icons: definition.permanentIcons, defense: original.defense, owner: 'p1', controller: 'p0' };

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
    expect(dialog).toHaveTextContent('黑色忠诚 ' + original.loyalty);
    expect(dialog).toHaveTextContent(original.subtypes.join(' · '));
    if (original.defense !== undefined) expect(dialog).toHaveTextContent('防御 ' + original.defense);
    if (original.subtitle) expect(dialog).toHaveTextContent(original.subtitle);
    expect(screen.getByLabelText('费用 ' + original.cost)).toBeInTheDocument();
    const iconsLabel = (icons, prefix) => prefix + '：调查' + icons.investigation + '，战斗' + icons.combat + '，势力' + icons.influence;
    if (original.kind === 'attachment') {
      expect(screen.queryByLabelText(iconsLabel(original.permanentIcons, '图标'))).not.toBeInTheDocument();
      expect(screen.queryByLabelText(iconsLabel(original.temporaryIcons, '先手图标'))).not.toBeInTheDocument();
      expect(dialog).toHaveTextContent('永久战斗1和临时战斗1');
    } else {
      expect(screen.getByLabelText(iconsLabel(original.permanentIcons, '图标'))).toBeInTheDocument();
      expect(screen.getByLabelText(iconsLabel(original.temporaryIcons, '先手图标'))).toBeInTheDocument();
    }
    expect(dialog).toHaveTextContent(original.magic || '死亡');
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
