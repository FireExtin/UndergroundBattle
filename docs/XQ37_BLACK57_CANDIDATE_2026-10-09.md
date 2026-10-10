# XQ37 夜总会看门人：黑色有限候选

本批从已审并发布 Site49 的 `7e51625cf30074adfc4f8923ce45ea455c788c71` 接续，仅加入原排期 B13 黑色 XQ37。当前120普通卡（含10地区），另有8秘社；候选身份为 `rust-v0.2.57-black-entry-influence-candidate` / `limited-v2.52-black-entry-influence-candidate`。中立新增、JC037领袖跨名及JZ25/JZ26费用未决内容不在本次范围内。既有证据和他人工作保留。

## 原卡与已确认规则

实际查看完整原图 `resource/ymsj-fun.github.io/cards/XQ37 夜总会看门人.jpg`，SHA256 `93778bc4ffa69e5b79ad08738e3f31a78a669a049c42a43e68d04bd6398856b9`，印刷37/52。费用2、黑色忠诚1、角色／人类／罪犯、战斗1、防御1，没有魔法领域或关键词。可选正文为：【进场触发】若夜总会看门人位于有本方势力标志的地区，则在该地区上放置一个本方势力标志。两份图索引登记实际原图，ReadModal可打开。

2026-10-10 00:17 UTC，父会话转交用户“OK”，凭据 `Sentinel_6bc93af86df08191ac34716faccec289`，明确确认方案B：

- 在实际Enter事件发生时，来源所处地区的冻结行动者所属队伍势力须大于0，才产生这个可选印刷触发。
- 结算时查找同一原来源实例当前所处的真实地区；当前该队伍势力仍大于0，才在当前地区放置1点。
- 移动到零势力地区或离场，已声明能力照常结算，但不放置势力。进场0点后再因响应或获授奖励得到势力，不补发旧进场触发。
- 暗藏属于离场，使用现有fresh身份；旧能力不追随新的暗藏者或再次现身后的新实例。
- 行动者、队伍与原声明冻结值保持一致，控制权变化不迁移旧能力。

例：A/B各有本方1点，来源从A移到B，结算后A1/B2；B起初0点则A1/B0。来源回手、死亡或暗藏时，A保持1点。此用户裁定解决此前IR-01；原 `783dca7` 和 `c94c007` 的冻结原地区方案A及其运行证据仅作历史记录，不能作为新规则B的通过证据。

实际查看规则手册印刷5：“角色牌被翻面到背面也是离场。”玩家指南印刷9说明暗藏者不具有正面信息，并将潜伏视作角色离场。没有把JC052专门释疑推作XQ37的通用来源位置规则。

## 最小实现

保留无参数 `XQ37EntryInfluenceIfPresent`，复用已有source snapshot、卡牌实例ID、`board`和`place_influence`。仅在既有emit_event的XQ37印刷 `entry-existing-influence` 声明处检查实际Enter势力；不影响获授声望／JC089奖励程序。程序结算查找原实例ID对应的明置XQ37及其当前地区，再按冻结actor所属队伍检查势力并放1点。进场地区快照继续保留作帧准入，不再作为本印刷程序的结算目标。没有新增身份、队列、通用绑定框架。

保留c94c007的P2修复：仅XQ37待选声明校验冻结actor的seat/playerId、Accept阶段、trigger种类、0到1可选数量、allowDecline=true、空amount/preview、唯一accept且card=None；标题、描述和显示标签可变。Game、RoomEnvelope及帧执行仍拒绝程序移植、别名、重复或嵌套执行、伪造actor/地区/目标/支付/游标与暗藏根卡。秘密派遣不触发，正常派遣和付费现身依既有Enter事件触发，单纯移动不产生进场触发。

## 本轮验证及实际限制（2026-10-10 更新）

已完成规则B的实际完整生产验证，早期方案A/c94c007运行结果不计入本轮：

- Native workspace：776项通过、0失败、0跳过，包含原hegemony-server完整lib-test二进制及18项XQ37测试。GREEN导出及其真实Room begin/submit、存读、回放断言全部开启。
- 新编译WASM：433个场景、24,403次转换、96,975个视图与独立Native输出完全一致；319非法命令、42非法构筑拒绝，1次quote一致。最大u64种子保持原opaque字节。
- XQ37专项：706份原Native输入（494快照、212真实Room命令）及2,824个四席视图一致；64非法定义在view/applyRoom两入口共128次拒绝。
- React全量：75个文件、667项通过。7份当前UI材料直接取自原Native生产函数，旧51–56材料仍保持原字节并用于旧版本拒绝；另逐项比对全部24快照、9命令、1quote和132视图。
- Worker：实际构建后21个文件、68项通过，包含真实workerd、临时本地D1、并发、鉴权、暂停重开、私有选择、重复命令、回滚、40个历史tuple拒绝；MSJC09自然Room链2346 HTTP调用、461个接受命令通过。12份当前Worker材料直接复制原Native导出，旧材料字节不改。一次首轮失败仅为陈旧的109卡/engine50夹具；将精确断言与已有session协议夹具更新至120卡/engine57后，全量通过。

本次Native、WASM、UI和Worker成功运行均无OOM/kill；完整Native/WASM运行触及cgroup max计数，不能描述为所有memory.events均为0。当前生产WASM为2,406,075字节，SHA256 `6dd02fc245292ebf0d7a0a233f9402db37dc85e3610d0e8d368a5de2e6116915`，pkg、Worker generated及部署产物三处实读一致。原行为断言未删减；Native和UI混色牌组各自固定到其原引入时有限卡集，恢复原50卡场景而非随全卡池增长溢出。

早期两次rustc SIGKILL/OOM失败的命令、日志和旧控制运行材料均保留。资源核查确认/tmp构建及/dev/shm材料累计约16.44GB shmem被计入16GiB cgroup，并非证明rustc本身需要16GiB。按用户授权移至普通磁盘、保全源码/Git/在途改动和历史原字节后，33个闲置target目录已清理，只保留一个实际复用且受限的普通磁盘Cargo target。验证导出在逐成员SHA校验后压缩保存；verify.sh退出即清理其自身临时大包。构建入口加入普通磁盘/可用余量校验与单进程配置，后续清理规则写于rust-game-wasm/README.md；未加入通用缓存管理框架。

完整原Native原始材料、源码差异、命令/日志/资源采样和独审报告位于 `/workspace/game-publication-evidence/xq37-black57-B-rebuild-20261010`；清理和迁移凭据位于 `/workspace/game-build-storage-recovery-20261010`。历史36份Native/WASM原字节（399,300,603字节）保留为原文件或经校验的压缩成员；没有重建历史核作为替代。

Sites官方包167文件及site-workflow.mjs校验通过，原生Sites owner权限和同一云端task归属已核，官方source helper已打开现有公开Site49。用户授权在固定候选独审通过后正常推送协作分支并更新该现有Sites；发布结果另记独立交付凭据。GitHub/main不改，不强推、不建worktree、不换执行器。Library401和官方包旧403保持原失败记录；无Library容量不足证据，未删除Library文件。当前没有可用浏览器控制插件，以上React/Worker验证不声称自然浏览器或在线D1验收。
