// Assemble only fresh current-source Native outputs. Opaque states are never rewritten.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {isDeepStrictEqual} from 'node:util';
import {pathToFileURL} from 'node:url';
const root=path.dirname(process.argv[1]);
const source=process.cwd(),env=JSON.parse(fs.readFileSync(path.join(root,'native-env.json'))),pin=JSON.parse(fs.readFileSync(path.join(root,'frozen-source.json'))).source_commit;
const {kernel,catalog}=await import(pathToFileURL(path.join(source,'rust-game-wasm/tests/current-kernel.mjs')));
const {writeCurrentFixture,sha256,currentWasmSha256}=await import(pathToFileURL(path.join(source,'tools/current-native-source.mjs')));
const read=p=>JSON.parse(fs.readFileSync(p,'utf8'));
const inputs=[],outputs=new Map(),delayedRaw=[];
function queue(relative,value){assert(!outputs.has(relative));outputs.set(relative,value);}
function raw(p){const bytes=fs.readFileSync(p);inputs.push({path:p,sha256:sha256(bytes),bytes:bytes.length});return JSON.parse(bytes);}
const list=(dir,filter=n=>n.endsWith('.json'))=>fs.readdirSync(dir).filter(filter).sort().map(n=>path.join(dir,n));
const snapshots=[];
for(const card of ['jc061','jc087','jc094','wm056','wm023','xq25','wm038','jz35','jc052'])for(const file of list(env[card.toUpperCase()+'_EVIDENCE_DIR'])){
 if(card==='jc094'&&!['public-target.json','hidden-target.json','self-target.json','target-is-cost.json'].includes(path.basename(file)))continue;
 snapshots.push({card,file,snap:raw(file),matches:[]});
}
const lifecycle=[];
for(const [color,key]of [['blue','BLUE_FRONTEND_DIR'],['black','BLACK_EXPANSION_EVIDENCE_DIR'],['red','RED_TIME_EVIDENCE_DIR']])for(const file of list(env[key])){
 const name=path.basename(file);if(!['jc032-positive.json','jc032-zero-hit.json'].includes(name)&&!name.includes('corpse-choice-')&&!name.includes('private-forecast'))continue;
 lifecycle.push({color,file,snap:raw(file),matches:[]});
}
const all=[...snapshots,...lifecycle],byState=new Map();for(const row of all){const a=byState.get(row.snap.state)||[];a.push(row);byState.set(row.snap.state,a);}
for(const file of list(env.GREEN_EVIDENCE_DIR,n=>/^step-\d+\.json$/.test(n))){
 const bytes=fs.readFileSync(file),r=JSON.parse(bytes);const a=byState.get(r.state);if(!a)continue;
 const action=r.command.action.action;if(r.command.action.kind!=='game'||action?.kind!=='choose'&&a.every(x=>x.card!=='jc094'&&x.card!=='jz35'&&x.card!=='wm023'&&x.card!=='wm038'&&x.card!=='wm056'&&x.card!=='xq25'&&x.card!=='jc052'))continue;
 if(r.expected.outcome!=='accepted')continue;
 for(const row of a){let ok;
  if(row.color)ok=action?.kind==='choose';
  else if(row.card==='jc061')ok=action?.kind==='choose'&&Boolean(action.selected?.length)!==path.basename(row.file).includes('decline');
  else if(row.card==='jc087')ok=action?.kind==='choose'&&isDeepStrictEqual(action.selected,row.snap.selected);
  else ok=isDeepStrictEqual(action,row.snap.action);
  if(ok)row.matches.push({file,row:r,sha:sha256(bytes)});
 }
}
function chosen(row){assert(row.matches.length,row.file+' has no exact accepted current Native command');const first=row.matches[0];assert(row.matches.every(x=>isDeepStrictEqual(x.row.command.action,first.row.command.action)),row.file+' ambiguous accepted Native payload');return first;}
function value(row){const m=chosen(row),r=m.row;return{name:path.basename(row.file,'.json'),seat:r.seat,before:row.snap.state,beforeViews:row.snap.views,command:r.command,expected:r.expected,afterViews:r.views,provenance:{snapshot:row.file,snapshotSha256:sha256(fs.readFileSync(row.file)),acceptedNativeStep:m.file,acceptedNativeStepSha256:m.sha}};}
const scope='Explicit fresh Native layouts and original accepted Room commands; no natural play, migration or public admission.';
for(const card of ['jc061','jc087','jc094','wm056','wm023','xq25','wm038','jz35','jc052']){
 const cases=snapshots.filter(r=>r.card===card).map(value);assert.equal(cases.length,4,card+' complete case set');
 const result={scope,sourceBaseline:pin,wasmSha256:currentWasmSha256,cases};
 queue(`rust-game-wasm/tests/current112/${card}.json.gz`,result);
 if(card==='jc052'){
  const out=path.join(root,'raw-selectors/web/src/game/jc052Native112Current.fixture.json');delayedRaw.push([out,JSON.stringify(result)+'\n']);
 }
}
assert.equal(lifecycle.length,10);queue('rust-game-wasm/tests/current112/lifecycle.json.gz',{scope,sourceBaseline:pin,wasmSha256:currentWasmSha256,cases:lifecycle.map(row=>({...value(row),name:row.color+':'+path.basename(row.file,'.json'),color:row.color}))});
// Original Native win-flow checkpoints supply all private before-views.
const win=env.WIN_FLOW_EVIDENCE_DIR;
function steps(name){const files=list(win,n=>n.startsWith(name+'-')&&new RegExp('^'+name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+'-\\d+-step\\.json$').test(n));files.sort((a,b)=>Number(path.basename(a).slice(name.length+1).split('-')[0])-Number(path.basename(b).slice(name.length+1).split('-')[0]));const rows=files.map(file=>({...raw(file),inputName:path.basename(file),inputSha256:sha256(fs.readFileSync(file))}));assert(rows.length,name);for(let n=1;n<rows.length;n++)assert.equal(rows[n-1].expected.state,rows[n].state);return rows;}
function beforeViews(row){const p=path.join(win,row.inputName.replace('-step.json','-checkpoint.json')),checkpoint=raw(p);assert.equal(checkpoint.state,row.state);return checkpoint.views;}
const names=['entry-XQ37-0-first-0','nested-reveal-Before(1, 0)','privilege-renown-true','privilege-already-wins','world-empty-nested-2-1','world-empty-jc050','world-empty-conservation','world-empty-duel'];
queue('rust-game-wasm/tests/current112/win-flow.json.gz',{scenarios:names.map(name=>({name,steps:steps(name)}))});
const captures=[];for(const[name,emptyIndex]of[['world-empty-duel',1],['world-empty-slot-2',2]]){const rows=steps(name).filter(row=>beforeViews(row)[0].regions[emptyIndex].cardId&&!row.views[0].regions[emptyIndex].cardId);assert.equal(rows.length,1);captures.push({name,emptyIndex,beforeState:rows[0].state,beforeViews:beforeViews(rows[0]),step:rows[0]});}
const sparse=steps('world-empty-jc050').filter(row=>{const v=beforeViews(row)[row.seat];return v.pendingChoice&&JSON.stringify(v.pendingChoice.options.map(o=>o.id))===JSON.stringify(['region:0','region:2','region:3','region:4'])&&!row.expected.errorCode;});assert.equal(sparse.length,1);queue('rust-game-wasm/tests/current112/fixed-slots.json.gz',{captures,sparseChoice:{beforeViews:beforeViews(sparse[0]),step:sparse[0]}});
// Fresh seven-card Native example; not the old layout-relabeling example.
queue('rust-game-wasm/tests/current112/seven-card.json.gz',raw(path.join(root,'raw-selectors/web/src/game/sevenCardUIV0112.fixture.json')));
const xq24=list(env.XQ24_TRACE_DIR).map(raw);assert.equal(xq24.length,12);xq24.forEach(r=>assert.deepEqual(r.catalog,catalog));queue('rust-game-wasm/tests/current112/xq24.json.gz',{catalog,scenarios:xq24});queue('rust-game-wasm/tests/current112/xq24-invalid.json.gz',raw(env.XQ24_CONTAINER_EVIDENCE_FILE));
for(const[relative,value]of outputs){assert(!fs.existsSync(path.join(root,'fixtures',relative)),relative);}
for(const[file,data]of delayedRaw){assert(!fs.existsSync(file),file);}
for(const[relative,value]of outputs)writeCurrentFixture(relative,value);
for(const[file,data]of delayedRaw)fs.writeFileSync(file,data,{flag:'wx'});
fs.writeFileSync(path.join(root,'selector-input-manifest.json'),JSON.stringify({source_commit:pin,scope,inputs},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cards:9,cardCases:snapshots.length,lifecycleCases:lifecycle.length,winScenarios:names.length,xq24Scenarios:xq24.length,rawInputs:inputs.length,source_commit:pin}));
