import {StrictMode,useState} from 'react';
import {createRoot} from 'react-dom/client';
import {Table} from '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/Table';
import {Help} from '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/Help';
import {testView,testCard,testCatalog} from '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/testFixtures';
import '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/game.css';
function Probe(){
 const [submitted,setSubmitted]=useState<unknown[]>([]);
 const [pending,setPending]=useState(false),[help,setHelp]=useState(false);
 const cards=Array.from({length:6},(_,i)=>({...testCard,instanceId:'synthetic-hand-'+i}));
 const choice={id:'local-mulligan',kind:'mulligan',title:'起手再调度',description:'本地合成组件夹具；不连接游戏房间。',playerId:'p0',min:0,max:6,allowDecline:true,options:cards.map(c=>({id:c.instanceId,label:c.name,card:c}))};
 return <div className="hg-app"><header className="hg-header"><span>本地模态交互验收</span><button onClick={()=>setTimeout(()=>setPending(true),700)}>打开本地选择</button><button onClick={()=>setHelp(true)}>上手指南</button></header><Table modalPaused={help} view={{...testView,status:'playing',hand:cards,phase:pending?'mulligan':'action',step:pending?'mulligan':'action',pendingChoice:pending?choice:null,legalActions:pending?[{id:'local-choose',kind:'choose',choiceId:choice.id,label:'选择'}]:[]}} catalog={testCatalog} busy={false} connection="online" onAction={a=>{setSubmitted(x=>[...x,a]);setPending(false)}}/><output id="submissions">{JSON.stringify(submitted)}</output>{help&&<Help catalog={testCatalog} onClose={()=>setHelp(false)}/>}</div>;
}
createRoot(document.getElementById('root')!).render(<StrictMode><Probe/></StrictMode>);
