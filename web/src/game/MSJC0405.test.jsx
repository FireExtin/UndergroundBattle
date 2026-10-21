import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,within} from '@testing-library/react';
import {catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {Lobby} from './Lobby';
import {ReadModal} from './ReadModal';
import {createDeckDraft,publicDeckDraft,readDeckLibrary,saveDeckDraft,validateDeckDraft} from './deckLibrary';
import {readLobbySelection} from './deckWorkspace';
import {saveSession} from './api';

const specs=[
  {id:'MSJC04',name:'鸣钟教派',color:'红',subtype:'秘密教派',subtitle:'末日钟的侍奉者',sha256:'a6ba66c7fb73acdbe4cab001e6330d0f339c46e12b6d3e714307196e9fd23cf3'},
  {id:'MSJC05',name:'国家机构',color:'灰',subtype:'机构',subtitle:'政府权力',sha256:'d71e7c3df37a7c857cfb74c9988261ad722c5f3012ca49c75151fd407dab59f3'},
];
function draft(spec,count=25) {
  let left=count;const cards=[];
  for(const card of catalog.cards.filter(c=>c.color===spec.color&&['character','spell','attachment'].includes(c.kind))) {
    const n=Math.min(left,card.deckCopyLimit??left);
    if(n)cards.push({cardId:card.id,count:n});left-=n;
  }
  expect(left).toBe(0);cards.push({cardId:'JC125',count:50-count});
  return {...createDeckDraft(catalog),name:`${spec.name} · 本地保存测试`,cards};
}
const mount=(props={})=>render(<Lobby catalog={catalog} busy={false} onCreate={vi.fn()} onJoin={vi.fn()} onCreateDraft={vi.fn()} onJoinDraft={vi.fn()} retry={vi.fn()} {...props}/>);
beforeEach(()=>{localStorage.clear();sessionStorage.clear();history.replaceState({},'','/#play');});
afterEach(()=>{cleanup();vi.restoreAllMocks();localStorage.clear();sessionStorage.clear();history.replaceState({},'','/');});

describe('complete real red and gray society entry',()=>{
  for(const spec of specs) {
    it(`${spec.id} exposes its complete printed card and exact original scan`,()=>{
      const society=catalog.societies.find(c=>c.id===spec.id);
      expect(society).toMatchObject({name:spec.name,color:spec.color,subtypes:[spec.subtype],unique:true,supported:true,printedCost:null,startingHand:6,subtitle:spec.subtitle,deckConstraints:[{kind:'minimumColor',color:spec.color,count:25}],unresolvedAbilities:{}});
      expect(society.abilities.map(a=>a.key)).toEqual(['drawWithInitiative',spec.id==='MSJC04'?'search-red-unique':'search-gray-unique']);
      const original=readFileSync(resolve(`../resource/ymsj-fun.github.io/cards/${spec.id} ${spec.name}.jpg`));
      expect(createHash('sha256').update(original).digest('hex')).toBe(spec.sha256);
      expect(readFileSync(resolve(`public/cards/${spec.id}.jpg`))).toEqual(original);
      render(<ReadModal card={{...society,cardId:spec.id,instanceId:'reader',owner:'p0',controller:'p0',faceDown:false,exhausted:false}} definition={society} viewerId="p0" onClose={vi.fn()}/>);
      expect(screen.getByRole('dialog')).toHaveTextContent(spec.subtitle);
      expect(screen.getByRole('dialog')).toHaveTextContent('起手 6 张');
      fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
      expect(screen.getByRole('img',{name:`${spec.name}原始牌面`})).toHaveAttribute('src',`/cards/${spec.id}.jpg`);
    });
    it(`${spec.id} is selectable at initial entry, rejects24 and submits25 real color cards`,()=>{
      const invalid=draft(spec,24),valid=draft(spec,25);valid.id+='-valid';
      saveDeckDraft(invalid);saveDeckDraft(valid);const create=vi.fn();
      const ui=mount({onCreateDraft:create});
      const select=()=>screen.getByRole('combobox',{name:'入席秘社（可选）'});
      expect(within(select()).getByRole('option',{name:new RegExp(spec.name)})).toBeInTheDocument();
      fireEvent.change(screen.getByRole('combobox',{name:'已保存牌组'}),{target:{value:`saved:${invalid.id}`}});
      fireEvent.change(select(),{target:{value:spec.id}});
      expect(screen.getByRole('alert')).toHaveTextContent(`当前 24 张`);
      expect(screen.getByRole('button',{name:'创建牌桌 →'})).toBeDisabled();
      expect(create).not.toHaveBeenCalled();
      fireEvent.change(screen.getByRole('combobox',{name:'已保存牌组'}),{target:{value:`saved:${valid.id}`}});
      fireEvent.change(select(),{target:{value:spec.id}});
      expect(screen.getByRole('button',{name:'创建牌桌 →'})).toBeEnabled();
      const selected=readLobbySelection();expect(selected.draft.societyId).toBe(spec.id);
      expect(readDeckLibrary().drafts.find(d=>d.id===valid.id).societyId).toBeNull();
      ui.unmount();mount({onCreateDraft:create});
      expect(select()).toHaveValue(spec.id);
      fireEvent.click(screen.getByRole('button',{name:'创建牌桌 →'}));
      expect(create).toHaveBeenCalledWith(expect.any(String),'duel',expect.objectContaining({name:valid.name,societyId:spec.id,cards:valid.cards}));
      expect(validateDeckDraft(create.mock.calls[0][2],catalog).valid).toBe(true);
    });
    it(`${spec.id} persists named society decks and joins with the complete saved snapshot`,()=>{
      const saved={...draft(spec),societyId:spec.id};saveDeckDraft(saved);const join=vi.fn();
      history.replaceState({},'','/?invite=NEWTABLE');mount({onJoinDraft:join});
      fireEvent.change(screen.getByRole('combobox',{name:'已保存牌组'}),{target:{value:`saved:${saved.id}`}});
      expect(screen.getByRole('combobox',{name:'入席秘社（可选）'})).toHaveValue(spec.id);
      expect(screen.getByLabelText('所选牌组')).toHaveTextContent(saved.name);
      fireEvent.click(screen.getByRole('button',{name:'加入牌桌 →'}));
      expect(join).toHaveBeenCalledWith('NEWTABLE',expect.any(String),publicDeckDraft(saved));
      expect(readDeckLibrary().drafts.find(d=>d.id===saved.id)).toEqual(saved);
    });
  }
  it('converts a preset to a session snapshot without silently saving and reports the real color deficit',()=>{
    const create=vi.fn();mount({onCreateDraft:create});
    fireEvent.change(screen.getByRole('combobox',{name:'入席秘社（可选）'}),{target:{value:'MSJC04'}});
    expect(screen.getByRole('alert')).toHaveTextContent('当前 0 张');
    expect(readDeckLibrary().drafts).toHaveLength(0);
    expect(readLobbySelection().draft.societyId).toBe('MSJC04');
    fireEvent.change(screen.getByRole('combobox',{name:'入席秘社（可选）'}),{target:{value:''}});
    fireEvent.click(screen.getByRole('button',{name:'创建牌桌 →'}));
    expect(create).toHaveBeenCalledWith(expect.any(String),'duel',expect.objectContaining({societyId:null,cards:catalog.decks[0].cards}));
  });
  it('locks society editing while entering and while restoring an existing seat',()=>{
    let ui=mount({busy:true});expect(screen.getByRole('combobox',{name:'入席秘社（可选）'})).toBeDisabled();ui.unmount();
    saveSession({roomId:'existing',inviteCode:'OLDROOM',token:'synthetic-local-seat',seat:0});
    history.replaceState({},'','/?invite=OLDROOM');const join=vi.fn(),joinDraft=vi.fn();mount({onJoin:join,onJoinDraft:joinDraft});
    expect(screen.getByRole('combobox',{name:'入席秘社（可选）'})).toBeDisabled();
    expect(screen.getByText(/保留桌内秘社/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button',{name:'加入牌桌 →'}));
    expect(join).toHaveBeenCalledWith('OLDROOM',expect.any(String),catalog.decks[0].id);expect(joinDraft).not.toHaveBeenCalled();
  });
  it('explicitly loads the entry snapshot into the existing editor and preserves unsaved edits before another load',()=>{
    mount();
    fireEvent.change(screen.getByRole('combobox',{name:'入席秘社（可选）'}),{target:{value:'MSJC04'}});
    fireEvent.click(screen.getByRole('button',{name:'收藏与组牌 →'}));
    fireEvent.click(screen.getByRole('button',{name:'载入当前入席快照'}));
    fireEvent.change(screen.getByRole('textbox',{name:'牌组名称'}),{target:{value:'仍未保存的红色构筑'}});
    fireEvent.click(screen.getByRole('button',{name:'牌组设置与模板'}));
    expect(screen.getByRole('combobox',{name:'秘社（可选）'})).toHaveValue('MSJC04');
    fireEvent.click(within(screen.getByRole('navigation',{name:'大厅导航'})).getByRole('button',{name:'入席'}));
    fireEvent.change(screen.getByRole('combobox',{name:'入席秘社（可选）'}),{target:{value:'MSJC05'}});
    fireEvent.click(screen.getByRole('button',{name:'收藏与组牌 →'}));
    expect(screen.getByRole('textbox',{name:'牌组名称'})).toHaveValue('仍未保存的红色构筑');
    expect(screen.getByRole('button',{name:'载入当前入席快照'})).toBeDisabled();
    expect(readDeckLibrary().drafts).toHaveLength(0);
    fireEvent.click(screen.getByRole('button',{name:'保存牌组'}));
    expect(readDeckLibrary().drafts[0]).toMatchObject({name:'仍未保存的红色构筑',societyId:'MSJC04'});
    fireEvent.click(screen.getByRole('button',{name:'载入当前入席快照'}));
    expect(screen.getByRole('combobox',{name:'秘社（可选）'})).toHaveValue('MSJC05');
    expect(readDeckLibrary().drafts[0]).toMatchObject({name:'仍未保存的红色构筑',societyId:'MSJC04'});
  },20_000);
});
