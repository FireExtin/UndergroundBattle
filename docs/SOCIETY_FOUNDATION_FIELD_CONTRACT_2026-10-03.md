# 秘社区基础机制：前后端最小契约草案

私有候选基线：Site17 `8f700cdcb90cfb5847d75b33bd02a69fa65d8664`。本契约不声明真实秘社卡已上线，不改变web；父负责前端拆分和复核。

## 目录与构筑

新增 `Catalog.societies: SocietyDefinition[]`，与普通 `cards` 数组分开。普通48定义仍原样；生产候选 `societies=[]`、`deckBuildRules.societySupported=false`。测试专用feature提供明确FIXTURE前缀的条目，独立版本身份，不能作为玩家牌上线。

每个SocietyDefinition平铺原CardDefinition字段（`kind:"society"`），新增 `startingHand:number`、`deckConstraints:[{kind:"minimumColor",color:string,count:number}]`、`unresolvedAbilities:{[abilityId]:issueId}`。后者命中的能力服务器拒绝，不自行决定未裁定条件。首批仅实现有实际规则需求的至少某颜色数量约束；无其他构筑规则抽象。

DeckDraft已有 `societyId:string|null`。仅允许该版本societies内已注册的ID，单独保存，不放入cards计数；cards仍至少50。未知、未开放秘社及把秘社塞普通cards均拒绝。准备前其他席的选定societyId不公开；`yourDeck`只供本席，准备开始一次原子公开全部已选秘社。无秘社起手6，选定时按startingHand。

## 牌桌授权投影

新增 `View.societyZones:[{id:"society:p0",playerId:"p0",card:null|CardView}, …]`，每个已占席位始终一项，没选择秘社也有空区。区域ID跨刷新稳定，不属于任何Region。

CardView沿用 `instanceId/cardId/name/owner/controller/kind/exhausted/faceDown/text` 等；秘社 `kind:"society"`，region缺省，不参与地区图标/伤害/赢区回底。普通卡牌或能力不能使其离場。公开后所有席能看秘社正面；手牌、再调度等仍仅本人有牌面，队友不共享。entity状态来源以服务器最新授权投影为准。

## 合法动作与响应

沿用 `{kind:"activate",cardId:秘社instanceId,abilityId:能力key}`；新增只读LegalAction `sourceZoneId:"society:p0"`，前端将动作归于该区的卡。命令不增加zone参数，不采信客户端来源标签，由Rust按实体查找与controller核验。动作无默认region；有目标时沿现有目标字段，不把秘社伪装成地区。

费用、堆叠、响应意图、原命令ID重试和选择协议沿用现有接口。服务器拓展来源定位来横置秘社，付款一次后使用已有持久SourceSnapshot/ResolutionFrame；下一回合重置秘社。MSJC09 U13仍未开放整牌、未选择任何条件语义；不能用fixture-draw能力替代其原文效果。

## 持久化与版本

Player内部新增 `society_zone:{card:Card|null}`，默认空用于该候选字段兼容；新命令的societyId已由DeckDraft序列化并进入房间快照/原意图收据。候选和fixture采用不同engineVersion，已发布v028及更早房间必须走原冻结核，不能静默升级。本契约仍待父复核，发布前版本号由父决定。
