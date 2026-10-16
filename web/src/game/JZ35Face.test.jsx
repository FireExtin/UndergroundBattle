// Source-face regression only. This does not prove production ABI or Room actions.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { render,screen,fireEvent } from '@testing-library/react';
import { it,expect,vi } from 'vitest';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
const record=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='JZ35');
const definition={...record,permanentIcons:record.icons.permanent,temporaryIcons:record.icons.temporary,defense:null,abilities:[]};
it('shows the complete JZ35 source face, Mind identity and unchanged original image',()=>{
  expect(record).toMatchObject({name:'思维湮灭',kind:'spell',color:'红',cost:2,magic:'心灵',loyalty:['红色'],subtypes:['法术','心灵'],unique:false,keywords:[]});
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JZ35 思维湮灭.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('5b84ed6442f88c2406d7a0955ef225e1a8061fe1951da58818567234c193c1dc');
  expect(readFileSync(resolve('public/cards/JZ35.jpg')).equals(original)).toBe(true);expect(cardScanUrl('JZ35')).toBe('/cards/JZ35.jpg');
  render(<ReadModal card={{...definition,cardId:'JZ35',instanceId:'source-face',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent(record.text);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'思维湮灭原始牌面'})).toHaveAttribute('src','/cards/JZ35.jpg');
});
