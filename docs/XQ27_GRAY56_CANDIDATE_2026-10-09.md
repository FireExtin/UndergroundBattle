# XQ27 定点清除行动：灰色有限候选 2026-10-09

来源为已实际查看的完整原图 `resource/ymsj-fun.github.io/cards/XQ27 定点清除行动.jpg`，SHA256 `78f2d5e2422b611bbf159a33083d70c75bf2ac83103325550430a394e8a803b5`，与 `docs/factions/card-specifications.json` B13 原图规格一致。费用4、灰色忠诚2、事务／阴谋，没有魔法领域、关键词或目标要求。完整正文为标准行动“消灭所有暗藏者”。

基线为 `68547fce82bc5dbd8877fb4dac7685af703eed24`，加入一张完整卡，当前119普通卡含10地区、8秘社。候选版本为 engine56/pool51：`rust-v0.2.56-gray-hidden-sweep-candidate` / `limited-v2.51-gray-hidden-sweep-candidate`。

实现仅增加无参数 `XQ27DestroyAllHidden`，在响应完成后的结算时冻结所有当前地区的暗藏者，再复用既有 `remove_death_batch`。不预先选择地区、不指定目标、不新增队列或绑定框架。暗藏者无论正面类型均消灭，墓地归拥有者，暗藏者移除不产生角色死亡触发；正面角色、地区、秘社和资产区不属于本效果。既有离场、控制释放、结附清理及级联逻辑继续适用。真实付费现身事务仍使用原有空程序，不等于打出其标准行动。

`xq27.rs` 只准入该完整程序和真实空现身帧，Game及RoomEnvelope存读入口一致拒绝操作移植、重复执行、伪造操控者／地区／目标／现身标记；打出入口在付费前拒绝预声明地区或目标。原118张定义、牌组、世界牌数据不变。原图与两份运行时图索引均登记，UI不添加新动作框架。

验证落点：`/dev/shm/xq27-gray56-evidence-20261009`；独审：`/dev/shm/xq27-gray56-independent-review`。使用明确准备的Native布局、真实Room命令、React与WASM、当地Worker/D1。没有自然浏览器整局或随机长局声明，本批不发布Sites、不改访问策略、不触碰GitHub/main。Site48匿名首页403已交父且停止该请求，不在本批重试。

最终测试、独审和协作分支提交结果在完成后追加。本任务旧Native54输入与可执行字节无损ZIP逐项SHA核验后移除展开副本；Native53、54、55历史版本字节保留，未删除Library资料或他人WIP。

验证已通过：Native新卡10项、同时移除复用15项及卡池断言1项；WASM逐字节215输入（137检查点、62成功命令、4拒绝、12非法存档）和812个席位视图一致；React5文件81项；两份TypeScript配置、Vite、Wrangler dry-run；本地workerd/D1及current-only路由4项。Native使用无默认特性的生产确定性核心／RoomEnvelope，未声称重跑Native HTTP整套。已停止该旧批全量回归。

本批试验中修正了测试资产ID、响应现身费用及触发隔离布局，UI隐私断言按实际省略cardId修正；构建提高原192MB Node上限至384MB后通过。以上是测试／执行资源问题，不伪称为已修复产品缺陷。历史53/54/55共33份引擎原字节再次逐项SHA核验一致（54、55部分为无损ZIP，不再使用旧展开路径）。独审与提交结果待最终裁定登记。
