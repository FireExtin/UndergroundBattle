# 秘社区基础机制私有候选与验证交接

输入：父明确的首批秘社区基础范围、Site17实现源码 `8f700cdcb90cfb5847d75b33bd02a69fa65d8664`、已有秘社原规则/原图核验；继承的 `16a8d93` 仅为审计文档提交。输出在私有分支 `codex/hegemony-society-foundation-20261003`，实现diff、源文件、原始测试日志、目录与WASM/旧核哈希将打包Library给父复核。本轮未推送、部署、修改web或进入任何真实房间。

## 实现范围

- Player有独立 `society_zone:{card:null|Card}`，每个已占席的授权View都有稳定 `societyZones` 条目。该区不属于Region，无秘社也存在；秘社实体不进入普通board target、资产/秘密派遣/现身或普通离场路径。
- DeckDraft的可选societyId单独准入并保存；不占玩家牌组50张，仍受基础同名/数量及秘社具体构筑条件验证。本批只增加已有规则需求对应的“至少指定颜色数量”谓词，复用现有不可变房间牌组快照。
- Start一次原子创建全部秘社并同时公开，随后各席按其startingHand抓牌、按原流程再调度；无秘社为6。准备期其他席不会从societyZones获得已选秘社身份，yourDeck仍只供本人。
- 复用activate/cardId/abilityId和AbilitySpec；增加只读LegalAction.sourceZoneId。来源定位独立拓展到秘社区，SourceSnapshot.region为None；费用横置通过该来源定位，使用既有Assets支付、原子回滚、响应帧与SourceSnapshot序列化。下一回合重置原秘社instance，不重新生成。
- 未决能力经unresolvedAbilities小接口直接拒绝，未把条件解释成发动门槛或付费无收益。MSJC09及所有真实秘社均未注册；U13未选择语义。无封印、计划、核心或MSJC16能力实现。

前端最小字段契约见 [SOCIETY_FOUNDATION_FIELD_CONTRACT_2026-10-03.md](SOCIETY_FOUNDATION_FIELD_CONTRACT_2026-10-03.md)，已先保存Library给父转交。现有web的actionPayload白名单不会把sourceZoneId等只读元数据当命令传入。前端仍由独立执行线负责。

## 生产候选与fixture严格分开

生产候选身份 `rust-v0.2.8-society-candidate`；目录普通cards仍48、societies为空，societySupported为false，没有公开不完整卡。用于机制验证的显式Cargo feature `society-fixtures` 身份为 `rust-v0.2.8-society-fixture`，单独生成在忽略目录 `pkg-society-fixtures/`：三个FIXTURE条目分别验证起手6、起手4且至少25中立牌、未决条件。它们不是印刷卡；测试不能记作真实秘社或自然UI上线验收。两身份不能互读存档，正式发布版本号由父后续确定。

已将原发布v028包按字节冻存为legacy-v0.2.8：WASM 1,619,119字节、SHA256 `6bbd2ce9ffdefdb3c24b7c0ca1786440ce087e4be34cff6af7f6c4af5e7bbc51`。原v021–v027未改；本地候选Worker路由八个冻结核加候选共九核。原Site17仍是八核，未被本轮私有构建替换。不同engineVersion继续按完整规则/卡池/引擎tuple路由，未进行旧房迁移。原生服务继续沿既有约束要求旧存档使用旧binary，不能把Worker冻结路由声称为原生旧库迁移。

## 已实际执行的验证

| 检查 | 结果 | 证明范围 |
| --- | --- | --- |
| 默认 `cargo test --locked --workspace` | 101库＋32集成＝133通过 | 生产候选、空秘社区、真实卡仍关闭及原有回归 |
| `cargo test --locked --workspace --features society-fixtures` | 106库＋32集成＝138通过 | 额外5项基础纵向fixture；两统计重叠，不累加为271 |
| 默认native/WASM compare | 26场景、678迁移、2237投影、1报价、6拒绝，完整状态/日志/各席投影一致 | 原有有限卡行为及新增空区字段；保留默认完整场景 |
| fixture native/WASM compare | 1场景、163迁移、646投影、1拒绝，一致 | 正常创建/加入/准备/开局/建立资产/付费秘社能力/响应/下一回合重置，无初始状态布局注入；使用合成卡目录，不是浏览器自然UI |
| 默认本地Worker/workerd/D1测试 | 22通过 | 生产候选及全部冻结核的原有恢复/响应/存储测试、九核身份路由；本地模拟非公网 |
| 独立fixture Worker/workerd/D1 | 1通过；823 HTTP调用、159接受命令 | 四席从创建到正常下一回合，付款后重开D1恢复原响应窗口和五秒截止、原命令重试、异意图拒绝、手牌/选择私密、原instance重置；无API状态注入 |

所有构建只有本地Rust/WASM与Worker `--dry-run`；backend-only构建脚本不调用前端构建，也不提供发布路径。WASM fixture验证支持显式独立ABI路径，普通verify默认仍使用生产pkg。

首轮fixture全套测试发现旧“rule definitions数＝普通cards数”断言忽略新增独立societies；已改为两注册表总数，普通玩家池仍另断言48。Worker fixture的初轮还发现外部模块目录需modulesRoot，以及无合法响应牌不能请求composing；只修测试前置，保留原协议。最终恢复测试保存正常undecided窗口与截止，允许真实时间已到期时一次合法结算；本次实际响应在截止前成功恢复（日志responseRestoredBeforeExpiry:true）。没有为让测试通过加入额外fixture能力或修改生产响应规则。相关失败日志保留供审阅。

## 审阅边界与待办

秘社区基础候选可纵向验证；完整真实秘社卡准入、具体条件/能力、前端交互、独立四人自然UI及公网刷新验收均未完成。起手4/中立25是机制fixture，不替代MSJC01或MSJC16原文。待父复核本diff与U13、安排前端分支及后续代表卡；父批准后再由本执行者串行集成测试和现有Site发布。本轮不更新父已接手的52项优先级或把基础切片升级为完整“秘社机制通过”。
