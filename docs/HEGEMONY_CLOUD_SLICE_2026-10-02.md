# 霸权云端可玩切片：边界与接口

输入：用户授权的 Rust 服务/规则核 + React 客户端；原私有仓库 HEAD a09f14d。旧 Go 保留。
本阶段只开放逐卡实现与测试过的真实卡牌，使用受限卡池自组预组，不能标为官方四套预组。

## 规则真相源

`resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` 扫描页码=印刷页码+1。

- 印刷 P6：初始手牌6，一次再调度：暂放任意手牌、抓等量、然后原牌洗回；忠诚来自资产派系/领域；白底图标只在操控者团队持先手时生效。
- P8–9：开始/行动/对抗/结束；横置角色不参加；每地区调查→战斗→势力，奖励为对应图标差值。调查排序顶X后抓1；战斗分配X并同时伤害；势力抵消敌方标志后放差值。
- P9：赢区前快速行动窗口；地区上牌按各拥有者选择的顺序置牌库底；地区入计分区、补地区、可选赢区触发入栈。整理版“回墓地”和“奖励固定1”错误。
- P13：构筑至少50张、同名最多3张（无知路人按其印刷能力无限制）；秘社可选。
- P17–18：杀伤/护卫/撤回/屏障/灵体与护盾；护盾为强制移除并终止来源效果。
- P21：2V2五地区、团队10分、同队同侧；座位0/2只可正常派遣至地区0..2，1/3至2..4；队伍共用行动/优先权，全队让过才交出；任一队员行动重置该队让过记录。调查/伤害执行者仅从实际参与该对抗的队员选择。先手特权每队每回合1次，发动者必须贡献至少1图标。
- 1V1 三地区、8分；经典2V2(12分、顺时针个人步骤)不混入本包。

## 第一批卡池（随后逐卡测试）

共用：JC125 无知路人；LC19 外科医生；LC20 心理咨询师；LC21 称职的保镖；LC22 退役军人；LC23 安格鲁；LC24 林中女巫；XQ49 葬礼；JC118 停战协议。
主题：JC002/JC003/XQ03（调查控制）；JC014/JC016/JZ08（机动战斗）；JC056/JC059/JC063（区域防守）；JC086/XQ12/JC092（墓地回收）。
从仓库 cards.json 导入完整印刷文字、类型、费用、忠诚、永久/临时图标和防御，不允许忽略开放牌的能力。
世界牌第一批限定为完全实现的 DQJC107/112/113/114/116 各2张；这是明确标示的受限世界卡池，不是官方十地区组合。未实现牌不可选。

## 协作 HTTP 契约

服务端监听默认8090，可同源托管 web/dist；开发 Vite /api 代理8090。
房间/座位令牌由服务端安全随机生成；客户端令牌本机存储，邀请仅能占空座。

- GET /api/health
- GET /api/catalog → { rulesVersion, cardPoolVersion, engineVersion, decks:[{id,name,description,cardCount,cards:[{cardId,count}]}], cards:[CardDefinition] }
- POST /api/rooms {name,mode:"duel"|"teams",deckId} → Session
- POST /api/rooms/join {inviteCode,name,deckId} → Session
- GET /api/rooms/:roomId/state（Authorization: Bearer token）→ View
- GET /api/rooms/:roomId/events（同 Bearer；SSE，data为本人View）
- POST /api/rooms/:roomId/commands（同 Bearer）{commandId,expectedVersion,action:Action} → View；冲突409+{error,view}，拒绝400+{error,message}；重试相同commandId只返回原提交结果。
- Session={roomId,inviteCode,token,seat,view}
- View={roomId,inviteCode,version,mode,status:"lobby"|"playing"|"finished",you,players:[{id,seat,name,team,deckId,ready,eliminated,handCount,deckCount,score}],firstTeam,activeTeam,priorityTeam,turn,phase,step,winScore,winnerTeam?,regions:[{id,index,cardId,name,threshold,points,influence:[number,number],characters:[CardView]}],hand:[CardView],assets:[CardView],graveyard:[CardView],scoreCards:[CardView],stack:[{id,label,controller,cardId?,targetId?}],pendingChoice:Choice|null,legalActions:[Action & {id,label,description?}],log:[{version,text}],versions:{rules,cardPool,engine},waitingChoice:{playerId,kind,title}|null}
- CardView={instanceId,cardId?,name,owner,controller,kind,region?,exhausted,faceDown,cost?,text?,icons?:{investigation,combat,influence},defense?,damage?,shield?,wounds?,color?,magic?}；对手/队友暗牌不得包含cardId/name/text/印刷属性，名称只能“暗藏者”。己方手牌只出现在自己的hand；其他玩家手牌只给数量。
- Action={kind,cardId?,targetId?,region?:number,option?,choiceId?,selected?:string[],top?:string[],bottom?:string[],allocations?:{[instanceId]:number}}
- Choice={id,kind,title,description,playerId,options:[{id,label,card?:CardView}],min?,max?,amount?,allowDecline?}，可选kind为mulligan/discard/order/investigation/damage/recipient/target/trigger/search/region_return；order/investigation使用top/bottom；damage使用allocations；其他使用selected，空selected代表允许的拒绝。
- 房主start/restart、选牌组、准备等都经合法Action。所有动作由令牌确定actor，expectedVersion验证；等待选择时只有选择者有choose动作，选择权与priority分离。

每局串行锁；SQLite事务先持久化完整状态、命令去重响应与日志，成功后广播/确认；固定版本和seed+命令可确定性回放。跨手牌/牌库/墓地/场上的区域变化和翻面更新instance身份；地区间移动保留实体身份与状态；旧引用不可打到翻面后的新实体。

## 验收

测试规则奖励、团队让过/重置、区域距离、实例身份、隐藏投影、命令去重、拒绝无变更、恢复待选、完整结束与重开。
云端Chromium真实2与4独立上下文试玩；验证另一端即时收到、自选调查排序/伤害/弃牌、刷新/断线/服务重启后续接。保留复现脚本和截图；不依赖Mac或隧道、不上传私有卡图到第三方。
