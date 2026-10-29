# CQ27 骨根灵主封闭机制组

基线 `dacb2ed0964e9372f7c60856219a703f53665ff5`。本组仅新增正式黑卡 CQ27，复用已实现的 TK001 行尸，不把指示物重复计为新卡。目录307条，公开准入仍236条；CQ27 `supported:false`。中立/双色暂停，不改变TK010项目例外。

## 原图与规则

原图 `resource/ymsj-fun.github.io/cards/CQ27 骨根灵主.jpg`，SHA256 `19c1a81185aaf17365c8888005b7ac4eb926db248e1349710b43de3b0aa1814d`。公用JPG、索引与客户端映射保留原始字节。黑色、费用5、黑忠诚1、独有、遗迹守卫者；植物/仙灵/超能力者，永久调查1/战斗1，临时影响1，防御3。右下骷髅是死亡领域；类型行右侧面具是版本标记，不是心灵领域。纠正旧规格空领域；未修改原图。

卡文：“〔日形条件〕非资产；混乱；不能移动”“准备步骤触发 将你墓地中的一张角色牌置于你的牌库底，然后在本地区生成一个行尸指示物角色。”“墓地行动 牺牲三个角色：将骨根灵主从墓地放置进场。”日形只限定非资产，不是强制公开关键词；正常潜伏仍需普通潜伏费用。

《霸权说明书》SHA256 `a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886` 印刷p18/PDF19：混乱进场不能指定地区，随机选择。沿已存在共享混乱入口，在实际进入时随机一次；没有发明可预选地区或失败重抽规则。指示物费用0、无忠诚、非资产依该手册印刷p5/PDF6。

《隐秘世界规则手册》SHA256 `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35` 印刷p7说明领域位于右下；p10牺牲只能由操控者执行。墓地行动费用要求三个不同的自己操控明置角色，不能用队友、敌方、暗藏者或核心区对象支付。实际生成物也可支付，按既有项目生命周期离场消失，不进入实体墓地；未把触手特例推广给行尸。

准备触发沿既有有限准备观察：来源/操控者/原地区在事件时固定；当前自己墓地角色有牌必选一张置底，非角色及队友墓地排除。原文没有“若你如此做”条件，空墓地仍执行能做的生成部分。新行尸具有真实出生身份并因迟缓横置进入。

## 实现与验证边界

普通派遣、潜伏/现身、墓地牺牲回场、准备生成均走实际引擎入口。响应移走原墓地实体后不以同名替身回场、不退牺牲费用、不消耗随机数。明置禁止移动不影响混乱进场；翻暗失去印刷能力后可移动。来源、费用回执、实际栈卡、有限程序、待选和同帧唯一载体由Game/Room恢复校验。没有旧ABI兼容。

本地完整命令与结果记录在 `/workspace/bq084-validation/cq27-*.result.json` 和对应 `logs/cq27-*.log`；交付汇总与恢复bundle位于 `/workspace/cq27-validation/delivery/`。最终 Native 本组11/11，共享准备/生成物/死亡续体/混乱回归37/37，两组48个不同测试，失败0、ignored0；显式过滤未受影响的旧JC035长轨迹，不重跑全库。

新Native三场景186命令：合法50张自然入口176命令，普通派遣/现身预设入口各5。默认同源WASM逐条对照186命令、744视图、186重复结果一致。UI相关整套8文件151/151通过；Web、默认WASM及本地Worker构建通过。本地Worker三场景3/3通过，158命令、632视图、158次实际重复提交及冷重开，失败0/跳过0；自然准备前28命令仅Native/WASM，Worker从真实自然准备后的墓地行动支付前检查点执行后续148命令，普通派遣/现身各5命令。

Native原始JSON SHA256 `42acf756f2edc888e125ee8300bf67260758b9cb14ba45f79953a09bebe9a757`；WASM `5f131d73833963a11fdc4d06dfa58826127728fe9aa87aeb1beef1ea7669ee30`；Rust来源清单 `7f2277ba5ab8cc74d3eedb880754e326639812fe95954a4be35c94b30dbeec38`。Cargo.lock SHA256 `1745cc1af931f500e763222c39100da3ac0521f4a1f96fc54e82c8589e2edb54`未变。Rust/Cargo1.90.0、Node24.19.0、wasm-bindgen0.2.104；单Cargo作业/单测试线程。

原失败保留：02为潜伏准备场景未备普通费用，9过1失败，修测试输入不改规则；03为自然指令见证未完整分配地区奖励返回的预测牌，10过1失败，按实际返回的全体选项修顶/底分配，不跳过校验。首次01的2项通过不能替代最终覆盖。

所有结果均为作者的Native/WASM/UI组件/本地Worker验证，不是独审或真实浏览器试玩。原独审拒绝、Library401、推送限制维持，不重试、不换入口、不合main、不部署。混乱随机选中已受入场禁令地区的既有公共裁定边界未在本组另造规则。

## 精确复现

```sh
export UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/undergroundbattle-build
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1
CQ27_NATIVE_OUT=/workspace/cq27-validation/evidence-04/native.json node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib cq27_ -- --nocapture --test-threads=1
node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- jc035_tests:: generated_tests:: next20_death_core_tests:: chaos --skip jc035_native89_actual_room_trajectories_and_rejections --skip cq27_ --nocapture --test-threads=1
bash rust-game-wasm/build.sh
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/cq27-validation/fixtures-04 node /workspace/cq27-validation/verify-04.mjs
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/cq27-validation/fixtures-04 node tools/build-storage.mjs -- npm --prefix web test -- CQ27Integration Import20CardFaces ChoicePanel.test ChoiceRecovery deckLibrary.test Next20Presentation --maxWorkers=1
```

复现使用新证据目录以保留原输入。Worker从sites目录运行：`CURRENT_CLOCK_WORKER=/workspace/UndergroundBattle/sites/dist/cq27-local-04/worker.js HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/cq27-validation/fixtures-04 node --test --test-concurrency=1 test/cq27-integration.test.mjs`；本地Worker绑定实际当前WASM，禁止部署此测试包装器。
