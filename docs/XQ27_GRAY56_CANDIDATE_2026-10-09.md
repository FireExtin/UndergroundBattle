# XQ27 定点清除行动：灰色有限候选 2026-10-09

本批基于 `68547fce82bc5dbd8877fb4dac7685af703eed24` 加入一张完整灰色事务，当前119普通卡含10地区、8秘社。候选为 `rust-v0.2.56-gray-hidden-sweep-candidate` / `limited-v2.51-gray-hidden-sweep-candidate`。实现与测试源码固定在 `4d63b07608e473c828d16b403227654c5267e5ac`，已独审通过；后续收束提交仅增加文档证据，并对19个非docs改动路径做精确字节映射。

真相源为已实际查看的完整原图 `resource/ymsj-fun.github.io/cards/XQ27 定点清除行动.jpg`，SHA256 `78f2d5e2422b611bbf159a33083d70c75bf2ac83103325550430a394e8a803b5`，与排期B13完整规格一致：费用4、灰色忠诚2、事务／阴谋，没有魔法领域、关键词或目标要求。正文为标准行动“消灭所有暗藏者”。原图已登记在两份运行时图索引，ReadModal可打开真实牌面。

实现只增加无参数 `XQ27DestroyAllHidden`，在响应结束后的结算时冻结所有当前地区暗藏者，再复用既有 `remove_death_batch`，不增加队列、身份或绑定框架。暗藏者无论正面类型均消灭，墓地归拥有者；暗藏移除不产生角色死亡触发。已在响应中真实付费现身的角色逃离本次消灭。正面角色、地区、秘社与资产区不属于效果，既有离场／结附／控制清理继续适用。事务的真实付费现身仍使用原空程序，不等于打出标准行动。

有限准入在Game和RoomEnvelope存读入口一致拒绝程序移植、重复执行、伪造操控者、地区、目标及现身标记；打出入口在付费前拒绝预声明地区或目标。独审实际发现并关闭P2问题 `XQ27-IR-01`：伪造root堆栈卡暗藏标记会经原Bury保留为暗藏墓地牌。修复只在XQ27原root卡准入拒绝该标记，并以Rust完整Room单字段反例及实际WASM双入口证明拒绝；未改通用Bury。最终独审100项有限动态检查全部通过，无开放阻断。

验证全部通过：Native新卡10项、同时移除复用15项及卡池断言1项；WASM逐字节216份原始Native输入（137检查点、62成功命令、4拒绝、13非法存档）和812个席位视图一致；React5文件81项；两份TypeScript配置、Vite、Wrangler dry-run；本地workerd/D1及current-only路由6项，包括持久化重开、四席视图、重复命令原回执、40历史身份、5篡改身份和精确engine55旧Room拒绝。

原118张定义、牌组和世界牌数据逐项不变；独审再次独立核算33份历史53/54/55引擎原始字节SHA，全部一致。部分旧可执行与输入已在逐项SHA核验后无损ZIP保存，旧展开路径不再作为当前入口；没有删除Library资料或他人WIP。

证据索引为本目录 `XQ27_GRAY56_EVIDENCE_2026-10-09.json`，独审原文为 `XQ27_GRAY56_INDEPENDENT_REVIEW_2026-10-09.md`。完整Native输入、编译WASM、可执行、增量bundle、完整diff及本地构建包保存在云端 `/dev/shm/xq27-gray56-evidence-20261009`；独审脚本、反例与结果位于 `/dev/shm/xq27-gray56-independent-review`。

范围说明：测试使用明确准备的Native布局、真实Room命令、React/WASM及当地Worker/D1，未声称自然浏览器整局。Native使用无默认特性的生产确定性核心／RoomEnvelope，未重跑Native HTTP整套或旧批长回归。本批不发布Sites、不改访问策略、不重试Site48匿名首页403、不触碰GitHub/main、不使用worktree或force。
