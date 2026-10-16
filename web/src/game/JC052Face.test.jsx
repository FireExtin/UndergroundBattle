// Source face check, independent of runtime ABI inputs.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {render,screen,fireEvent} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
const record=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='JC052');
const definition={...record,permanentIcons:record.icons.permanent,temporaryIcons:record.icons.temporary,abilities:[]};
it('shows the entire Leader/entry face, four Red loyalty and the exact original',()=>{
  expect(record).toMatchObject({name:'终焉的诗寇蒂',subtitle:'剪断命运者',color:'红',kind:'character',cost:7,loyalty:['红色','红色','红色','红色'],magic:'星辰',keywords:['领袖'],unique:true,defense:1});
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC052 终焉的诗寇蒂.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('3b42cde3d48132e0ecbf73dd7ee78c6379c2319a0b83cb7304f24033ab361d33');
  expect(readFileSync(resolve('public/cards/JC052.jpg')).equals(original)).toBe(true);expect(cardScanUrl('JC052')).toBe('/cards/JC052.jpg');
  render(<ReadModal card={{...definition,cardId:'JC052',instanceId:'source-jc052',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent(record.text.replace(/\s+/g,' '));
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'终焉的诗寇蒂原始牌面'})).toHaveAttribute('src','/cards/JC052.jpg');
});
