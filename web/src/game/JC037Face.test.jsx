// Source face check, independent of runtime ABI inputs.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {render,screen,fireEvent} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
const record=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='JC037');
const definition={...record,permanentIcons:record.icons.permanent,temporaryIcons:record.icons.temporary,abilities:[]};
it('shows the entire continuous Leader face, three Blue loyalty and the exact original',()=>{
  expect(record).toMatchObject({name:'克里斯蒂娜·爱弗罗德',subtitle:'寂静女士',color:'蓝',kind:'character',cost:6,loyalty:['蓝色','蓝色','蓝色'],magic:'鲜血',keywords:['领袖'],unique:true,defense:3});
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC037 克里斯蒂娜·爱弗罗德.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('6d4199bd890373683c100b78cebca1cb6e291346b3c8a4fc7b25beea1f2feece');
  expect(readFileSync(resolve('public/cards/JC037.jpg')).equals(original)).toBe(true);expect(cardScanUrl('JC037')).toBe('/cards/JC037.jpg');
  render(<ReadModal card={{...definition,cardId:'JC037',instanceId:'source-jc052',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent(record.text.replace(/\s+/g,' '));
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'克里斯蒂娜·爱弗罗德原始牌面'})).toHaveAttribute('src','/cards/JC037.jpg');
});
