#!/usr/bin/env python3
"""Join every archive ID to pinned source specs and current finite code coverage; never unlock cards."""
from pathlib import Path
from collections import Counter
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[2]
def read(p): return json.loads((ROOT/p).read_text())
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 locator=read('resource/ymsj-fun.github.io/cards/cards.json');spec=read('docs/factions/card-specifications.json');index=read('rust-game/data/card-research-index.json');pool=read('rust-game/data/cards.json')
 current={c['id']:c for c in pool['cards']};roles={r['id']:r for r in index['records']}
 closed={
  'JC058':{'question':'faqRevealTerminology','basis':'pinned 058/135 Reveal text + stack-hidden exclusion; cross-edition FAQ never broadens trigger','evidence':['docs/factions/B02-evidence-package.md','rust-game/src/rules.rs','rust-game/src/engine.rs#detective_targets_all_sides_same_region_without_private_leaks_or_hidden_death']},
  'JC063':{'question':'faqRevealTerminology','basis':'separate face-up hide and hidden return modes; an already revealing stack entity is not a board hidden target','evidence':['docs/factions/B02-evidence-package.md','rust-game/src/rules.rs','rust-game/src/resolution.rs']},
  'DQJC108':{'question':'freeRevealLoyalty','basis':'documented user-approved archived FAQ for this version, not a general printed-text inference','evidence':['docs/RELEASE_V025_ACCEPTANCE_2026-10-02.md','rust-game/src/world.rs#florence_rotates_independent_players_and_free_reveal_ignores_loyalty_under_adopted_faq']},
  'DQJC111':{'question':'sacrificeDefenseSnapshot','basis':'documented user-approved pre-sacrifice current defense snapshot, including continuous modifiers','evidence':['docs/RELEASE_V025_ACCEPTANCE_2026-10-02.md','rust-game/src/world.rs#moscow_resumes_seat_order_and_uses_the_pre_sacrifice_defense_snapshot']},
 }
 rows=[]
 for cid,v in sorted(locator.items()):
  if cid=='TK007':continue
  s=spec['cards'].get(cid);f=(s or {}).get('fields',{});r=roles[cid];source=(s or {}).get('sourceVerification',{})
  pending=[]
  if not s:pending.append('missingPinnedSourceSpecification')
  else:pending=[q for q in s.get('openQuestionIds',[]) if q != closed.get(cid,{}).get('question')]
  if source.get('blockingFields'):pending+=['blockingSourceField:'+k for k in source['blockingFields']]
  if cid in ('JC016','BQ022'):pending.append('JC016_BQ022_WinReturnRetreatRecyclePriority')
  status='needsRuling' if pending else 'implementedBounded' if cid in current else 'unimplemented'
  rows.append({'id':cid,'name':f.get('name') or v.get('name'), 'role':r['role'],'color':f.get('colorKey') or v.get('color') or '无颜色','cardType':f.get('basicType') or v.get('basic-type'),
   'sourceStatus':source.get('status','missing'),'imagePath':source.get('imagePath'),'imageSha256':source.get('imageSha256'),
   'status':status,'inCurrentCatalog':cid in current,'serverSelectionEnabled':cid in current,
   'finiteEngineBehavior':'partialPendingInteraction' if cid in current and pending else 'completeForCurrentBoundedDefinition' if cid in current else 'absent',
   'openQuestions':pending,'closedVersionQuestion':closed.get(cid),'independentAcceptance':'notEstablishedByThisMatrix',
   'printedRules':f.get('printedRuleTextLines',[]),'confirmedMechanisms':s.get('confirmedMechanismIds',[]) if s else [],
   'evidence': ['rust-game/data/cards.json','rust-game/src/rules.rs'] if cid in current else ['docs/factions/card-specifications.json']})
 assert len(rows)==684 and len({x['id'] for x in rows})==684
 # Check numerical/icon fields against prior pinned review, not locator text.
 comparisons=[]
 for cid,c in current.items():
  f=spec['cards'][cid]['fields']
  for k,v in [('name',f['name']),('cost',f.get('printedCost')),('defense',f.get('defense'))]:
   if v is not None:assert c.get(k)==v,(cid,k,c.get(k),v)
  for k in ('permanent','temporary'):assert c['icons'][k]==f[k+'Icons'],(cid,k)
  comparisons.append(cid)
 old=read('docs/architecture/full-mechanics-design/mechanism-matrix.json');mechanisms=[]
 for m in old['mechanisms']+old['addenda']:
  cid=m['id'];status={'有限支持':'implementedBounded','部分支持':'partial','未实现':'unimplemented'}[m['current_status']]
  scope=m['verified_scope'];missing=m['missing'];evidence=['rust-game/src/rules.rs','rust-game/src/engine.rs','rust-game/src/resolution.rs']
  if cid=='gravePlay':status='implementedBounded';scope='JC085 graveyard face-up play, ordinary assets/loyalty payment, new instance, stack, four projections and reload';missing='Other cards/zones/permissions and cost exemptions absent';evidence+=['rust-game/src/play_sources.rs','docs/SITE13_SOURCE_CLOSURE_2026-10-03.md']
  if cid=='attachment':status='partial';scope='BQ022 typed host, attachment lifecycle, normal recycling, privacy, batch return and persisted ordering';missing='JC016+BQ022 Win return/recycle priority pending; broader host kinds, dual types and cards absent';evidence+=['rust-game/src/attachment.rs']
  if cid=='priority':scope='Atomic declared payment, LIFO, target guards, persisted frames, server five-second decision/composing protocol';missing='Forced trigger and multi-target generalized declarations, card-specific event arbitration absent'
  if cid=='continuous':scope='JC016/JC059 and asset-domain/region-influence modifiers including team friendliness';missing='Layered type/ability/control dependency graph absent'
  if cid=='deckChoice':scope='Draw/discard/forecast/search/order, independent world choices and simultaneous Hong Kong attachment search';missing='General reveal/peek grants and arbitrary card search criteria absent'
  if cid=='hidden':scope='Deploy/reveal/hide identity changes, loyalty/payments, controller privacy and normal attachment cleanup';missing='Seal, plans and generalized permission grants absent'
  if cid in ('crisis','eternal'):status='needsRuling';scope='Source term exists; no production playable mechanism';missing='Pinned rules do not establish the complete term/model; do not infer effects'
  mechanisms.append({'id':cid,'name':m['name'],'status':status,'scope':scope,'missing':missing,'ruleGate':m['rule_gate'],'evidence':evidence,'historicalDesignSource':old['reviewedCode']})
 assert len(mechanisms)==52
 ui=[
 ('roomEntry','创建/加入/原席恢复','implementedBounded','Lobby.tsx,useGame.ts','2/4席、历史席与同邀请码恢复；独立标签方案只在8584753单独补丁'),
 ('legalActions','合法动作按钮','implementedBounded','Table.tsx','仅执行有限卡池服务端提供的动作'),
 ('privateCards','四席私有牌视图','implementedBounded','CardTile.tsx','controller可见；服务端投影；合成控制布局不算自然试玩'),
 ('targetChoice','单目标与地区选择','implementedBounded','ChoicePanel.tsx,Table.tsx','目标资格由核守卫，非通用多目标'),
 ('orderedChoices','排序/预测/赢区回底','implementedBounded','ChoicePanel.tsx','逐拥有者排序、恢复待选；A默认回归已覆盖'),
 ('paidRecovery','付款后/未确认命令刷新','implementedBounded','api.ts,useGame.ts','原命令ID重试及409同步，不自动改版本重付'),
 ('responseIntent','响应意图界面','implementedBounded','ResponseWindow.tsx','5秒决定、无限时composing、取消与服务器截止'),
 ('deckLibrary','50张自组与固定预组','implementedBounded','DeckLibrary.tsx,deckLibrary.ts','有限池、版本与构筑限制；无实际秘社选择'),
 ('originalImages','原图与卡牌阅读','implementedBounded','ReadModal.tsx,CardTile.tsx','已登记卡的原图；不凭研究字段解锁'),
 ('combatModes','2人/4人对抗流程','partial','Table.tsx','霸权有限流程可用，非全部模式/任务/遭遇'),
 ('keywordPresentation','关键词策略与持续数值','partial','Table.tsx,CardTile.tsx','已实现卡可见；完整创伤/护盾/扩展关键词缺'),
 ('simultaneousChoices','同时私密承诺','partial','ChoicePanel.tsx','香港等有限路径可用；任意多对象承诺缺'),
 ('mobileReading','布局与可读性','partial','game.css,table-desktop.css','有现有布局/放大；未完成全部屏幕独立体验验收'),
 ('societyCore','秘社/核心实体操作','unimplemented','types.ts','研究展示不是可玩实体/规则入口'),
 ('sealShape','封印/变化/双面操作','unimplemented','types.ts','缺生产实体与合法动作'),
 ('xMultiTarget','任意X费/多目标组装','unimplemented','Table.tsx','有限单目标/现有费用表不覆盖通用声明'),
 ('persistentPeek','持续私密检视权限','unimplemented','CardTile.tsx','不能以卡名/已有背面视图推断授权'),
 ('pendingReturn','JC016+BQ022赢区策略','needsRuling','Table.tsx','合法组合存在；未禁止也未全面验收，等待优先关系裁定'),
 ]
 uis=[{'id':i,'name':n,'status':st,'source':'web/src/game/'+p,'scope':scope} for i,n,st,p,scope in ui]
 inputs=['resource/ymsj-fun.github.io/cards/cards.json','docs/factions/card-specifications.json','rust-game/data/card-research-index.json','rust-game/data/cards.json','rust-game/src/rules.rs','rust-game/src/catalog.rs','rust-game/src/attachment.rs','rust-game/src/play_sources.rs','rust-game/src/world.rs']
 result={'schemaVersion':1,'sourceAncestor':'cdf85ce0e574e7e5c0a840b1993057e034dafabf','sourceHashes':{p:sha(p) for p in inputs},
 'scope':'Archive face IDs excluding TK007 first-player marker; bounded code implementation is separate from independent UI acceptance',
 'catalogVersions':{key:re.search('pub const '+const+r': &str = "([^"]+)"',(ROOT/'rust-game/src/catalog.rs').read_text())[1] for key,const in [('rules','RULES_VERSION'),('pool','POOL_VERSION'),('engine','ENGINE_VERSION')]},
 'totals':{'archiveRecords':len(locator),'faceIds':len(rows),'sourceComplete':sum(x['sourceStatus']=='completeGameplayFieldsForPinnedImage' for x in rows),'sourceBlocked':sum(x['sourceStatus']=='blocked' for x in rows),'sourceMissing':sum(x['sourceStatus']=='missing' for x in rows),'runtimeDefinitions':len(current),'runtimeNonRegions':sum(c['kind']!='region' for c in current.values()),'cardStatus':{key:sum(x['status']==key for x in rows) for key in ['implementedBounded','partial','unimplemented','needsRuling']},'mechanismStatus':dict(Counter(x['status'] for x in mechanisms)),'uiStatus':dict(Counter(x['status'] for x in uis))},
 'currentPrintedFieldsMatched':comparisons,'cards':rows,'mechanisms':mechanisms,'ui':uis,'markerExcluded':'TK007'}
 out=ROOT/'docs/factions/current-implementation-matrix-2026-10-03.json';out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 assert result['totals']['sourceComplete']+result['totals']['sourceBlocked']+result['totals']['sourceMissing']==684
 print(json.dumps(result['totals'],ensure_ascii=False));return result
if __name__=='__main__':main()
