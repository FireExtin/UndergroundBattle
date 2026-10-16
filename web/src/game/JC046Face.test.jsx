// Complete printed-face rendering; this is not a deployed browser playtest.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {render, screen, fireEvent} from '@testing-library/react';
import {it, expect, vi} from 'vitest';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';

const record = JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'), 'utf8')).cards.find(card => card.id === 'JC046');
const definition = {...record, permanentIcons: record.icons.permanent, temporaryIcons: record.icons.temporary, abilities: []};

it('reads the complete nonunique JC046 face, permanent and temporary icons, mandatory reveal and original image', () => {
  expect(record).toMatchObject({
    name: '无面屠戮者', color: '红', kind: 'character', cost: 5, loyalty: ['红色'],
    magic: '鲜血', keywords: ['杀伤2'], defense: 3, subtypes: ['恶魔', '异界'],
    icons: {
      permanent: {investigation: 0, combat: 2, influence: 1},
      temporary: {investigation: 0, combat: 0, influence: 1},
    },
    text: '杀伤2\n【现身强制触发】每位玩家各牺牲一个角色。',
  });
  expect(record.unique ?? false).toBe(false);
  const original = readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC046 无面屠戮者.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('5d22023e0848d5d2e6566b2fdfe94ef62a80c7b97812f3f81066392aa97cc1ab');
  expect(readFileSync(resolve('public/cards/JC046.jpg')).equals(original)).toBe(true);
  expect(cardScanUrl('JC046')).toBe('/cards/JC046.jpg');

  render(<ReadModal card={{...definition, cardId: 'JC046', instanceId: 'source-jc046', owner: 'p0', controller: 'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  const dialog = screen.getByRole('dialog');
  expect(dialog).toHaveTextContent('无面屠戮者');
  expect(dialog).toHaveTextContent('恶魔 · 异界');
  expect(dialog).toHaveTextContent('红色 · 鲜血');
  expect(dialog).toHaveTextContent('防御 3');
  expect(dialog).toHaveTextContent('杀伤2 【现身强制触发】每位玩家各牺牲一个角色。');
  expect(screen.getByLabelText('费用 5')).toBeInTheDocument();
  expect(screen.getByLabelText('图标：调查0，战斗2，势力1')).toBeInTheDocument();
  expect(screen.getByLabelText('先手图标：调查0，战斗0，势力1')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', {name: '原始牌面'}));
  expect(screen.getByRole('img', {name: '无面屠戮者原始牌面'})).toHaveAttribute('src', '/cards/JC046.jpg');
});
