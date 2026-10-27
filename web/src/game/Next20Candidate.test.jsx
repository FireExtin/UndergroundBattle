// Candidate evidence is opt-in; it never admits these cards to a public deck.
import {render,screen} from '@testing-library/react';
import {describe,expect,it,vi} from 'vitest';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {kernel,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {Table} from './Table';

const closed=['BQ033','BQ009','BQ001'];
const draft=()=>({id:'next20-public-reject',name:'Public admission control',description:'local synthetic test',societyId:null,
  cards:structuredClone(catalog.decks.find(d=>d.id==='watchers').cards),rulesVersion:catalog.rulesVersion,
  cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:'2026-10-25T00:00:00Z'});

it('public new/join reject all three closed WIP definitions while a normal Room creates, restores and resumes',()=>{
  for(const id of ['BQ034','CQ25','XG25'])expect(catalog.cards.find(card=>card.id===id).supported,id).toBe(true);
  const valid=draft();
  let result=JSON.parse(kernel.newGameWithDeck('a11200000000000000000001','N20PUB','duel','P0',JSON.stringify(valid),'18446744073709551503'));
  const created=result.state;
  for(const id of closed){
    expect(catalog.cards.find(c=>c.id===id).supported).toBe(false);
    const bad=draft();bad.cards.push({cardId:id,count:1});
    expect(()=>kernel.newGameWithDeck('a11200000000000000000002','N20BAD','duel','P0',JSON.stringify(bad),'7')).toThrow();
    expect(()=>kernel.joinGameWithDeck(created,'P1',JSON.stringify(bad))).toThrow();
    expect(JSON.parse(kernel.view(created,0)).players).toHaveLength(1);
  }
  result=JSON.parse(kernel.joinGameWithDeck(created,'P1',JSON.stringify(valid)));
  let state=result.state, sequence=0;
  const command=(seat,action)=>{
    const view=JSON.parse(kernel.view(state,seat));
    const cmd={commandId:`next20-public-${sequence++}`,expectedVersion:view.version,action};
    const out=JSON.parse(kernel.applyRoom(state,seat,JSON.stringify(cmd),'0'));
    expect(out.outcome).toBe('accepted');state=out.state;return out;
  };
  for(const seat of [0,1])command(seat,{kind:'game',action:{kind:'ready'}});
  command(0,{kind:'game',action:{kind:'start'}});
  for(let n=0;n<4;n++){
    const views=[0,1].map(seat=>JSON.parse(kernel.view(state,seat)));
    const seat=views.findIndex(v=>v.pendingChoice);if(seat<0)break;
    command(seat,{kind:'game',action:{kind:'choose',choiceId:views[seat].pendingChoice.id,selected:[]}});
  }
  command(0,{kind:'pauseRoom'});const saved=state;
  for(const seat of [0,1])expect(JSON.parse(kernel.view(saved,seat)).pause).toBeTruthy();
  const polled=JSON.parse(kernel.pollRoom(saved,0,'0'));expect(polled.state).toBe(saved);
  command(0,{kind:'resumeRoom'});expect(JSON.parse(kernel.view(state,0)).pause).toBeFalsy();
});

const candidateRoot=process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES;
const fixture=candidateRoot?loadCurrentFixture('rust-game-wasm/tests/current112/next20-candidate-room.json.gz'):null;
describe.skipIf(!candidateRoot)('internal closed-card Native candidates, not public reachability',()=>{
  it.each(fixture?.cases||[])('$name matches actual Room commands, restore, duplicate probes and private chooser rendering',scenario=>{
    expect(fixture.publicAdmission).toBe(false);
    let state=scenario.initial.state,views=scenario.initial.views;
    for(const row of scenario.steps){
      expect(row.state).toBe(state);
      views.forEach((expected,seat)=>expect(JSON.parse(kernel.view(state,seat))).toEqual(expected));
      if(views[row.seat].pendingChoice&&row.command.action.kind==='game'&&row.command.action.action.kind==='choose'){
        const shown=render(<Table view={views[row.seat]} catalog={catalog} busy={false} connection="online" onAction={vi.fn()}/>);
        expect(screen.getByRole('dialog',{name:'待完成的选择'})).toBeInTheDocument();shown.unmount();
        const other=(row.seat+1)%views.length;
        const hidden=render(<Table view={views[other]} catalog={catalog} busy={false} connection="online" onAction={vi.fn()}/>);
        expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();hidden.unmount();
      }
      const actual=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));
      expect(actual).toEqual(row.expected);state=actual.state;views=row.views;
      expect(JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs))).toEqual(row.duplicate);
      views.forEach((expected,seat)=>expect(JSON.parse(kernel.view(state,seat))).toEqual(expected));
    }
  });
});
