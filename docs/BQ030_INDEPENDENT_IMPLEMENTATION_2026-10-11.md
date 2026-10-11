# BQ030 独立机制实现与待接线边界

> 本文保存本地 `18f2a78` 的未接线历史阶段。后续共享所有权、用户致死裁定和实际纳入结果见 [BQ030_CARD_ADMISSION_2026-10-11.md](BQ030_CARD_ADMISSION_2026-10-11.md)，本文的“待接线／待裁定”不是当前状态。

本批仅裁定官 BQ030。父端指定候选基线为 `14b7b5e06a0b1a3649f2a3f947ad515236ebdfd9`；本次本地起点为 `f98d1e07f28fe4fa9f48f9f3d75d131f186f652a`。不发布、不推送、不合并 main，保留此前提交。按父端“若需要修改共享文件，先列出文件确认所有权”的要求，共享接线文件尚未修改。

## 已写入的独立成果

- `rust-game/src/bq030.rs`：有限结附事件、冻结宿主及合格来源、可选触发、逐项选择入栈、放弃本事件剩余来源、来源离场后的独立抓牌，以及持久状态校验。上下文保存于专用 Op，避免给 SourceSnapshot、Declaration、ResolutionFrame 增加字段；不需要修改 `engine.rs`。
- `rust-game/tests/bq030_attachment_regression.rs`：15 个用例，包含印刷字段核验与真实 Room/Store 回归，覆盖实际支付、四席视图、逐命令重放、任意触发入栈顺序、部分放弃、初拥新操控者、明暗置、横置、异区、地区结附排除、失效和拒绝、响应杀死来源、待选/堆栈 SQLite 重开与重复收据。布局仅在每个用例首条 Room 命令前设置。20 个持久状态损坏反例使用实际命令所得状态，等待接线后执行。
- `tools/cards/compare-bq030-room.mjs`：从未修改的 Native 导出通过实际 WASM ABI 比较完整 transition、状态字节、四席视图、catalog 和非法状态拒绝。必须存在实际准入 BQ030、接受命令、拒绝命令及非法状态；不能把空样本或旧 WASM 视为成功。
- `web/public/cards/BQ030.jpg`：原图字节副本，SHA256 `85aaa7444b65060ab09f33f2ea9057647f88c961ff103fcca0c0763ce2564b86`，未改图。

## 原文与语义

印刷字段和原图来自 `docs/factions/card-specifications.json` BQ030 及 `resource/ymsj-fun.github.io/cards/BQ030 裁定官.jpg`，对应印刷文本“触发 当一个附属结附于本方角色上时，抓一张牌。”成本 2、蓝忠诚 1、鲜血、吸血鬼/政治家、防御 1、临时势力 1、非独有。

规则手册印刷 P2/P4、玩家指南 P10 和 FAQ P3 初拥条款可组合推出：结附成立时，初拥的持续控制已生效；“本方”依当前操控者，因此由新操控方的裁定官观察。这是原文组合推导，不冒称专门 BQ030 FAQ。手册 P2、玩家指南 P10 规定各触发可放弃，控制者自行决定同时触发入栈顺序，再逆序结算；单次 BQ030 结附事件所有合格来源属于同一操控者。

毒血 JC089 成功结附于防御 1 裁定官时，持续减防确定造成死亡，但原文没有唯一规定同次结附的触发资格是在死亡清理前保存还是清理后收集。手册 P6 / 霸权 P16 的来源独立规则仅从“能力加入堆叠后”起适用；FAQ P6 妖火例是随后响应。父端仍需回答最小问题：该裁定官是否保留此次可选抓牌触发。模块收集器尚未接任何实际调用点，其调用时点必须依该答复实现。

## 验证与准确范围

独立 Room 协议对照 2/2 通过，其中 XQ47 在真实 Begin/Submit 谋杀响应令宿主死亡后失效；整个 15 用例测试目标已编译。BQ030 模块还没有被 `lib.rs` 注册，尚未进行其类型检查或真实运行；BQ030 正反行为、SQLite 用例和 Native/WASM 比较均未通过验证，不能用准入门禁替代这些结果。

静态独审为条件 PASS，仅确认独立模块结构和测试路径。实际登记/调用点接线、所有运行用例通过、实际 Native/WASM 比较完成后，还需再次独审。新增源码已格式检查，diff 检查通过。

`/workspace/bq030-implementation-20261011/` 保存共享文件原像校验和、待接线注册 patch、未写回仓库的完整后像、原图清单、对照日志及图映射红/绿检查。只加入 proposed pool 时图映射检查先失败；完整 proposed mapping 后 129 张通过。此检查只说明待接线数据方案一致，仓库实际注册仍为 128 张。

## 接线所有权范围

需要父端确认：`rust-game/src/{rules,resolution,attachment,blue_expansion,model,catalog,lib}.rs`、`rust-game/data/cards.json`、`web/src/game/cardScans.ts`、`web/public/card-scans.json`。另有五处卡池计数断言需仅从 128 改成 129：`jc089_tests.rs`、`deck_seal_search_tests.rs`、`bounded_search_tests.rs`、`gray_lock_tests.rs`、`renown_tests.rs`。

`room.rs` 无需改动；`engine.rs` 的 P1 修复与前端交互仍归原任务。待接线 patch 故意不含成功结附 hook，避免默选未裁定的即时致死时点。拟议版本为 `limited-v2.56-bq030-attachment-candidate` / `rust-v0.2.61-bq030-attachment-candidate`，也需在父端整合时避免与 P1 候选版本碰撞。门禁不支持静默迁移旧局。
