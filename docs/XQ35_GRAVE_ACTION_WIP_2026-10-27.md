# XQ35 白骨仆人：墓地行动 WIP

基线 `b26fe8a6231dc4fb7a8c70e9f5f33f188937de28`。本片新增黑色 XQ35 完整原图字段、墓地标准行动和定向跨端证据；`supported:false`。目录增加一条关闭定义，279 定义 / 236 准入 / 43 关闭。新20张批次仍只有 BQ034、CQ25、XG25 三张已有新增准入；XQ35 尚未独审、未计完成，不合 main、不发布。

原图 `resource/ymsj-fun.github.io/cards/XQ35 白骨仆人.jpg`，SHA256 `16848181852ab99f7caa29cf84b65a1a018c644da3ad6b4e1aa0bb635c7920b3`，本轮直接查看。印刷费用横杠，不是0费普通派遣；永久战斗1、防御1、死亡领域、不死生物/骨骼/奴仆，无忠诚。public JPG 与原图逐字节一致，两处牌面索引同时添加。

机制复用 `pay_ability_costs`、真实封印事件、限次记录及响应帧。标准行动支付1和本人墓地另一张真实角色作为封印费用；费用失败由原 Game 事务回滚。费用在响应前已付，来源离开原墓地实例后不召回同名另一张，不退费。结算以新区域实例横置进入地区一，再走真实进场触发。禁止普通正面派遣和付费现身，仍允许秘密派遣。费用收据、来源、程序、限次及重复续体参与恢复校验。

本轮定向 Native 8项通过；同源码 WASM 构建及真实 Native Room 对照通过：1个明确布局、5条命令、20席位投影、5次 reducer 重复拒绝。UI 2项通过；本地 HTTP/Worker/D1 1项通过，5条命令逐条回执重放、5次冷重开、终态重放第一条回执。相邻 JZ51 墓地回归19项通过。不是自然开局或真实浏览器试玩；完整 native/UI/Worker 套件未重跑，既有固定目录数量门禁需在整批独审后统一对齐，不删除历史摘要或反例。

WIP 暂沿用113身份，不能作为可发布新内核；整批候选冻结时须更新唯一最新引擎/卡池身份并重建同源证据，不保留旧 ABI 兼容。当前原始证据仅绑定本片源码内容，不重签 b26 或以前夹具。

复现需要 Rust1.90、wasm32 target、wasm-bindgen0.2.104、Node24、已有锁定依赖及正常构建输出目录：

```sh
XQ35_NATIVE_OUT=<new-evidence>/native.json node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib xq35_ -- --nocapture --test-threads=1
bash rust-game-wasm/build.sh
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=<new-evidence>/fixtures node tools/fixture-tools/xq35.mjs <new-evidence>/native.json
# 在 web/，同一 HEGEMONY_CANDIDATE_NATIVE_FIXTURES：
npm test -- src/game/XQ35Integration.test.jsx
# 在 sites/：npm run build；只执行仓库已有 dry-run 构建，不发布。
# 从根目录生成全新本地时钟 wrapper，勿部署该 wrapper：
node tools/build-current-local-worker.mjs <repo>/sites/node_modules <repo>/rust-game-wasm/pkg <repo>/sites/dist/xq35-current-clock
# 在 sites/，同时传入上面的 fixtures 与 CURRENT_CLOCK_WORKER=<...>/worker.js：
node --test --test-concurrency=1 test/xq35-integration.test.mjs
```

本片先调查 CQ11/TK008，发现目录明确保留 `generatedTokenLifecycle` 问题：潜伏、封印、跨区、复制和离场消失缺统一原文支持。因此未实现或开放指示物，不以普通角色替身猜规则。XQ35 独审重点为来源实例、付费封印时点、每回合限次与区域更换语义；费用收据目前依循现有原生模块的冻结证据模型。
