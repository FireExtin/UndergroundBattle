import {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {Table} from '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/Table';
import {testView,testCard,testCatalog} from '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/testFixtures';
import '/workspace/UndergroundBattle-v027-player-sessions/web/src/game/game.css';
function Probe(){
 const [submitted,setSubmitted]=useState<unknown[]>([]);
 const cards=Array.from({length:6},(_,i)=>({...testCard,instanceId:'synthetic-hand-'+i,name:i===0?'公路骑士':'本地卡牌'+i}));
 const choice={id:'local-mulligan',kind:'mulligan',title:'起手再调度',description:'明确的本地合成夹具，不连接房间。',playerId:'p0',min:0,max:6,allowDecline:true,options:cards.map(c=>({id:c.instanceId,label:c.name,card:c}))};
 return <div className="hg-app"><header className="hg-header"><span>本地起手诊断</span><button>仅本地按钮</button></header><Table view={{...testView,status:'playing',hand:cards,phase:'mulligan',step:'mulligan',pendingChoice:choice,legalActions:[{id:'local-choose',kind:'choose',choiceId:choice.id,label:'选择'}]}} catalog={testCatalog} busy={false} connection="online" onAction={a=>setSubmitted(x=>[...x,a])}/><output id="submissions">{JSON.stringify(submitted)}</output></div>;
}
createRoot(document.getElementById('root')!).render(<Probe/>);
