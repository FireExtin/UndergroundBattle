# JZ55「传奇落幕」待审候选（云端，2026-10-06）

基于父会话已审 `b533ef25318585bd5d2b1c2a9225ea891108b339`，在独立工作树 `/workspace/jz55-game`、分支 `codex/jz55-cloud-candidate` 完成。本候选未发布、未推送 GitHub/main。已有 JZ31 Site 候选 `d42613af53cb091d6907bf6d6a3d5181cab993a8` 保留；其发布仍因官方 Sites 工具包下载 HTTP 403 暂停，不重试或经其他执行器搬运工具包。

实际查看了 `resource/ymsj-fun.github.io/cards/JZ55 传奇落幕.jpg`：费用 2、黑色忠诚 1、事务·命运、无魔法领域要求，文字为「传奇落幕不能被响应。快速行动：消灭目标独有角色。」JPEG SHA256 `9155faa90be22aa66e56148dfacd0d517eb7b6f7b56fd1b45b71a6804719784a`。独有依据既有印刷定义的 `unique`，隐藏牌不视为可指定的角色。规则手册的快速行动/堆栈/独有说明和既有 Immediate 实现已检查；采用父会话指定的有限实现。

实现仅增加无参数枚举 `TargetPredicate::JZ55UniqueCharacter`，将已有 Board/Character/Any/Anywhere 目标规格、`Destroy(Target(0))`、`ResponsePolicy::Immediate` 绑定在一条完整 JZ55 能力上。目标谓词检查真实源定义为 JZ55、目标正面朝上、印刷类型为角色、印刷独有标记为真；既有可指定检查继续处理依当前操控者判断的屏障。保持已有费用、忠诚、强制盾替代和墓地归属行为。整张 JZ55 定义及其能力只允许规范形状；移植到其他卡、模式/附件守卫和改变范围、费用、响应策略等均拒绝。

JZ55 在普通行动或真实 BeginResponse/SubmitResponse 中立即结算，不产生自身可响应堆栈对象。摧毁后产生的其他死亡效果沿用原流程，仍可选择、响应和存读。没有修改 engine、room、service、死亡快照、队列或实例身份机制。源事务通过现有 Bury 进入 owner 墓地，保留既有记录的 controller；角色死亡通过原离场路径归 owner 并重置，未为了测试改动该行为。

新内核身份为 `rust-v0.2.36-jz55-unique-destroy-candidate`，卡池为 `limited-v2.33-jz55-unique-destroy-candidate`，100 张普通卡含 10 地区、另有 8 会社和 5 预组。engine/pool 编号不是 Sites 平台版本。新 WASM 2,181,427 字节，SHA256 `0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036`；JS、两份声明和 package.json 与已审 engine35 字节相同。pkg 是原仓库忽略生成物，审查包另附当前全部 5 文件。

冻结 engine35 到 `rust-game-wasm/legacy-v0.2.35`，来自已验证云端 b533 工作树的生成包，WASM SHA256 `4ceb4f410ff96796888f2568b426a56866f8db743a8cebf41f41d9e47e98ee7b`。逐文件校验旧 172 个历史文件；旧 107 项原图映射及图像 SHA、旧 99 张定义、世界牌组、会社、预组和构筑规则均保持一致。元数据测试的当前卡数从 99 改为 100、黑牌数 12 改为 13、当前身份改为 engine36/pool33。

验证为定向范围，不是全仓通过声明：

- Native：最终 JZ55 过滤运行 10/10。另有 8 项既有规则准入、4 项 JC015 谓词、2 项 Immediate、14 项受卡池身份影响的原字段/构筑元数据检查通过，合计 38 项不同测试。元数据批次先 12/13，黑牌总数断言修正后该项单独通过。没有重跑既有 JZ31 大包或全 workspace。
- Native/WASM：49 个独立命令，含 22 次拒绝；57 个序列化检查点。两条分别连续的 RoomEnvelope 命令链为 9 步与 12 步；每步完整 transition、journal 回放、存读及四席视图对齐。508 次命令/快照视图比较、21 个 UI 快照四席合计 84 次、冻结 engine35 的 40 个历史快照四席合计 160 次均精确一致。另比较两种 SQLite 最终 receipt 命令。
- 第一条链：JC063 建堆栈，双席让过，真实 BeginResponse/SubmitResponse 施放 JZ55；立即摧毁独有目标并重开原堆栈窗口，旧窗口命令被拒绝，之后正常让过完成。
- 第二条链：前缀夹具中通过实际支付揭示 JZ27 并操控 JZ31，实际付费 JC102 造成伤害；连续链从明确存档边界开始，JZ55 立即摧毁 JZ27，JZ31 控制归还、失去光环后真实级联死亡。死亡时 actor/owner/controller 为 2，原地区实例被冻结。选择发动后再由席 0 实际 BeginResponse/SubmitResponse 用 JC063 响应死亡效果，先完成响应，再完成 JZ31；原死亡 frame 不变，地区影响力最终 `[0,1]`。
- SQLite：普通 Game 与真实 SubmitResponse 均通过实际 Store 命令完成；关闭并重开 Store 后重复 commandId 返回原 receipt，不重复费用或死亡。冲突 commandId、旧目标非法命令不产生数据库副作用。重开的是数据库 Store 连接，未宣称操作系统进程重启。
- UI：新增 8/8，既有 MixedDeckConstruction 5/5，合计 13 项不同测试。测试读牌原图、真实唯一目标选择和确认、composing/立即响应后的原堆栈、级联死亡可选发动与响应、50 张构筑存读/3 与 4 张限制。使用真实 WASM/native 夹具；未声称公网或完整浏览器对局。`npm run build` 类型检查及 Vite 构建成功。

已保存的早期失败日志全部保留；开发过程的夹具修正包括：数据插入因 pkg 尚未物化未执行、灰色费用被误设为白色、附件反例起初用错卡、Bury 的既有 controller 被错误要求归 owner、SQLite 离线 fixture 未同步 revision/误用 version 列、UI 对非独立卡片的附件节点作 HTMLElement 断言、空响应手牌时错误期待 canBegin；均修正夹具/断言并有对应绿日志。增强级联链的第一版有 Pending 字段名和 Option 类型编译错误，已修正。卡池元数据批次的黑色数量断言也单列前后记录。未把失败日志当成通过证据。

重现：使用仓库 Rust 1.90 工具链构建 `cargo build -p hegemony-wasm --target wasm32-unknown-unknown --release`，wasm-bindgen 0.2.104 输出 web pkg；运行 `cargo test -p hegemony-server --lib jz55`，设置 GREEN_EVIDENCE_DIR/JZ55_FRONTEND_DIR/JZ55_RESPONSE_EVIDENCE_DIR/JZ55_RECEIPT_EVIDENCE_DIR 可导出夹具、命令链和 SQLite receipts。审查包含确定导出的 oracle-clean/response-final/receipts/frontend-final，执行 `node rust-game-wasm/tests/jz55_compare.mjs <证据根目录>` 可重现一致性；web 中运行 `npm test -- src/game/JZ55UniqueDestroy.test.jsx` 和 `npm run build`。历史 UI 测试仍使用各自旧版本快照；本次未对全部历史 UI 测试改绑或声称全套运行。

待父会话独立审查源码、原图、两条链和候选内核。JZ55 尚未制作可发布 Site 更新，发布流程与任何 main 推送均未执行。
