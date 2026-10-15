// Complete source-card display; deployed browser acceptance remains separate.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {render,screen,fireEvent} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
const record=JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='JC095');
const definition={...record,permanentIcons:record.icons.permanent,temporaryIcons:record.icons.temporary,abilities:[]};
it('reads the complete unique JC095 face, subtitle, Leader, Renown and original image',()=>{
 expect(record).toMatchObject({name:'唐·格鲁尼',subtitle:'波士顿之主',color:'黑',kind:'character',cost:7,loyalty:['黑色','黑色','黑色'],magic:'',keywords:['领袖','声望'],unique:true,defense:2,subtypes:['人类','罪犯']});
 const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC095 唐·格鲁尼.jpg'));
 expect(createHash('sha256').update(original).digest('hex')).toBe('32dbf8c8af063f8b680bbba87c82920ac5a7dd4a18bcdb33e0d861894e05c020');
 expect(readFileSync(resolve('public/cards/JC095.jpg'))).toEqual(original);
 expect(cardScanUrl('JC095')).toBe('/cards/JC095.jpg');
 render(<ReadModal card={{...definition,cardId:'JC095',instanceId:'source-jc095',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent(record.text.replace(/\s+/g,' '));
 expect(screen.getByRole('dialog')).toHaveTextContent('波士顿之主');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'唐·格鲁尼原始牌面'})).toHaveAttribute('src','/cards/JC095.jpg');
});
