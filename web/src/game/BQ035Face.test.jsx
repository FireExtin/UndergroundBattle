// Component/source verification only; never counted as online gameplay.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { render, screen, fireEvent } from '@testing-library/react';
import { it, expect, vi } from 'vitest';
import { ReadModal } from './ReadModal';
import { CardContent, visibleCard } from './CardTile';
import { cardScanUrl } from './cardScans';

const record = JSON.parse(readFileSync(resolve('../rust-game/data/cards.json'),'utf8')).cards.find(c=>c.id==='BQ035');
const definition = {...record, permanentIcons:record.icons.permanent, temporaryIcons:record.icons.temporary, abilities:[]};
it('reads the whole pinned BQ035 print, including corrected blue loyalty and both continuous clauses',()=>{
  expect(record).toMatchObject({name:'幽冥骑士',cost:5,loyalty:['蓝色'],color:'蓝',magic:'鲜血',defense:3,subtypes:['吸血鬼','猎手'],keywords:['机动','威名']});
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/BQ035 幽冥骑士.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('52500e1a2eac4bb3ab1e9dc56c971669cea058b48f328ff09c42ebd24b7761b5');
  expect(readFileSync(resolve('public/cards/BQ035.jpg'))).toEqual(original);expect(cardScanUrl('BQ035')).toBe('/cards/BQ035.jpg');
  render(<ReadModal card={{...definition,cardId:'BQ035',instanceId:'knight',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent(record.text.replace(/\s+/g,' '));
  expect(screen.getByLabelText('图标：调查0，战斗2，势力1')).toBeInTheDocument();
  expect(screen.getByLabelText('先手图标：调查1，战斗0，势力0')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'幽冥骑士原始牌面'})).toHaveAttribute('src','/cards/BQ035.jpg');
});
const live={instanceId:'recipient',cardId:'JC029',name:'新生血族',owner:'p0',controller:'p0',kind:'character',region:0,faceDown:false,exhausted:false,currentMobility:true,currentMobilitySource:'knight-source',currentCombatGlory:true};
it('shows authoritative current granted mobility and glory without changing the printed face',()=>{
  render(<CardContent card={live} viewerId="p0"/>);
  expect(screen.getByText('机动 · 幽冥骑士授予')).toHaveAttribute('title',expect.stringContaining('source'));
  expect(screen.getByText('威名')).toBeInTheDocument();
});
it('redacts mobility grant facts for either viewer of a concealed card and omits them off board',()=>{
  for(const viewer of ['p0','p1']) expect(visibleCard({...live,faceDown:true},viewer)).toMatchObject({currentMobility:undefined,currentMobilitySource:undefined});
  const app=render(<CardContent card={{...live,region:undefined}} viewerId="p0"/>);
  expect(screen.queryByText(/幽冥骑士授予/)).not.toBeInTheDocument();
  app.rerender(<CardContent card={{...live,kind:'asset'}} viewerId="p0"/>);expect(screen.queryByText(/幽冥骑士授予/)).not.toBeInTheDocument();
});
