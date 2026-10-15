// Source-card rendering only; not a substitute for deployed browser acceptance.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {render,screen,fireEvent} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
const record=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='JC060');
const definition={...record,permanentIcons:record.icons.permanent,temporaryIcons:record.icons.temporary,abilities:[]};
it('reads the complete JC060 printed face and its exact original image',()=>{
  expect(record).toMatchObject({name:'神秘学顾问',color:'灰',kind:'character',cost:3,loyalty:['灰色'],magic:'心灵',keywords:[],defense:1,subtypes:['人类','学者']});
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC060 神秘学顾问.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('6a879cc4c380b2a79b9fc3f60acf57c408768a9f8fc4dfa07682c9db63b2a4ae');
  expect(readFileSync(resolve('public/cards/JC060.jpg'))).toEqual(original);
  expect(cardScanUrl('JC060')).toBe('/cards/JC060.jpg');
  render(<ReadModal card={{...definition,cardId:'JC060',instanceId:'source-jc060',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent(record.text.replace(/\s+/g,' '));
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'神秘学顾问原始牌面'})).toHaveAttribute('src','/cards/JC060.jpg');
});
