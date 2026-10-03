# JC088 从已审 v027 的有限本地迁入

本地分支 `codex/jc088-local-from-v027-20261003` 从 `3fda3eb1aab506226c83e95ba0638ec30a82cf1a` 出发，继承已审实现 `37a09a2cc0088845657eafd7088f54a2bbc19012`、v027 报告 c2704d3，以及本轮 QA 取消修复。只取 WIP `52ea9faee3f35101e3ac205196573c0baab9be93` 中 JC088、冻结核和相应 Worker 回归的有限差分；没有整支迁入。该 WIP 不包含后来的 v027 墓地、晚响应和 QA 修复，不能用它覆盖已审基线。

## 真相源与付款时点

实际查看 `resource/ymsj-fun.github.io/cards/JC088 职业杀手.jpg` 原图，SHA256 `9c270203639dae978acb65e9255ac34b5a5d5192e681f8aafd63018ede301a5f`；与 WIP 原图一致。整牌为3费、黑色忠诚1、人类/罪犯、永久及临时战斗各1、防御1，无横置来源费用。原图两项能力均进入执行核：

- 现身触发：消灭本地区印刷费用≤2的目标角色，可选任意一席的合法正面角色。
- 快速行动：支付2费，自身潜伏。

实际查看原始扫描版 `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` 的印刷 P7、P10、P11、P14、P15、P16（物理页分别为8、11、12、15、16、17）。P10/P16规定先选目标、再付款、入堆叠等待响应；P11规定付费现身满足忠诚，现身后才可发动现身触发，暗藏者不是角色，翻面保持横置/重置状态，潜伏按离场处理结附；P14/P16规定快速响应及能力入堆叠后独立于来源。

因此“≤2”是目标的印刷费用门槛，不是现身折扣。JC088现身支付3费；现身触发无额外费用，可放弃；自潜伏宣告时支付2费，效果等待响应。结算不再次付款；目标隐藏或离场后旧 instance 失效，费用不退；来源先自潜伏也不取消已发动的消灭能力。原图及上述原页未发现本迁入所需的新裁定。整理稿不作为原规则替代。

## 实现边界

`rules.rs` 增加可序列化、默认空的 `printed_cost_max` 目标谓词和 JC088 两项定义；`resolution.rs` 在选择及结算守卫共用路径筛选正面角色的印刷费用。目标费用修正不会改变印刷门槛。自潜伏沿用已审支付、离场、新 instance、横置保持与 controller 隐私投影路径。

运行卡池仅增加完整 JC088：40张定义含10地区，30张非地区牌。新候选 `reclaimers` 预组以3张JC088替换3张JC125，仍为50张；其余预组无变。原研究索引、其他卡及墓地准入不扩展。普通正面打出JC088不触发现身能力。

原生服务的新默认数据库为 `rust-game-v2.8.sqlite3`，并加入忽略规则；不会默认打开旧版原生数据库。本轮实际可验入口使用单独的本地D1目录。

新本地候选使用 `rust-v0.2.8 / limited-v2.6`，防止用变更后的规则读取旧版本状态。线上七核保留：原六个冻结目录逐字节不变，原v027冻结为第七个目录，WASM SHA与Site13一致；本地路由为七个旧核加新候选核。旧房按完整版本元组返回其冻结卡池与规则，新核直接拒绝旧身份。没有推送public仓库、合主分支、发布Site、改域名或访问其他人的房间。

`attachment.rs`、`play_sources.rs`、QA执行器取消修复及其测试与基线逐字节一致。JC016+BQ022赢区撤回组合仍为合法可达、待裁定、未验收；本次没有禁用、加裁定或改回收顺序。JC088正常自潜伏时BQ022按既有正常离场回收路径处理，与该待裁定赢区批量返回场景不同。

## 自动证据

证据位于 `docs/evidence/jc088-local-2026-10-03/`，均由本候选实际运行生成，未引用 WIP 的通过声称。

| 检查 | 实际结果 | 边界 |
| --- | --- | --- |
| 原生 workspace | 103项通过，其中71项库测试、32项集成测试 | 包含6项JC088、真实Window::Win回底、墓地与SQLite恢复 |
| 前端 | 18文件160项通过；类型检查和生产构建通过 | 保留controller隐私、回座、命令恢复等回归 |
| QA取消 | 2项通过 | 取消初始建席和最终视图失败均保留failed checkpoint并关闭context |
| 默认原生/WASM对照 | 22场景607转换1953投影，1报价6拒绝命令一致 | 默认含真实赢区、墓地正面付款、新instance和JC088；不是公网自然对局 |
| 实际workerd / 本地D1 | 全套18项通过 | 七旧核与新核路由、CAS、回执、计时、重开；属于提供方本地仿真 |
| JC088 Worker四席补测 | 1项通过 | 隐藏前后及重开四席投影，选择仅操控席可见，支付一次及重复回执 |
| 两份独立变异 | 均exit101、各1项失败 | 去掉印刷门槛、2费改0费；独立源码及Cargo目录，未污染候选 |

JC088六项原生测试额外覆盖0/1/2费及四拥有者候选、费用修正不放宽印刷门槛、同区/异区及隐藏目标、普通打出不触发、无目标/可放弃/横置现身、仅1资产拒绝且无旁写、2费付款时点、正常结附回收、目标响应潜伏失效、来源响应自潜伏后能力继续、controller与owner不同的隐私。控制转移初始布局为明确标注的本地合成场景，当前准入卡池没有夺取操控牌。

## 本地可验环境与复现

已安装 Rust1.90、wasm32 target、官方 wasm-bindgen0.2.104，并完成已有锁文件的Sites依赖安装；npm缓存和Wrangler配置均放在workspace。原核验证清单见 `kernel-preservation.json`，候选WASM SHA256为 `9f1a76887968ee6c18d9cb1db8dfbe0f2e1438f1541bd65bd54ba0854bc2ebda`。

在仓库根可运行：

```bash
export PATH=/workspace/.rust/cargo/bin:$PATH
export CARGO_HOME=/workspace/.rust/cargo
export RUSTUP_HOME=/workspace/.rust/rustup
export CARGO_TARGET_DIR=/workspace/.rust/targets/jc088-local
export WASM_BINDGEN=/workspace/.rust/wasm-bindgen-0.2.104/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen
cargo test --locked --workspace
bash rust-game-wasm/verify.sh
npm --prefix web test
npm --prefix sites run build --cache /workspace/.npm-cache
npm --prefix sites test --cache /workspace/.npm-cache
python -m unittest discover -s tools/cloud-playtest
```

Worker回归使用由本候选 `native_fixtures` 实际导出的 `prepared-assassin-v028.json`、`prepared-response-v028.json`；已比对与新导出一致。测试仅向明确标注的本地合成D1库写入布局，未向浏览器或公网对局注入。

本地Worker入口为 `http://127.0.0.1:8102`，D1单独保存在 `/workspace/.private-validation/jc088-local-runtime-20261003`。健康、卡池、入口HTML和JC088原图均只读检查；未建桌或代玩家出牌。该地址仅在本执行环境可访问，父线程的云浏览器能否连接或开启四个独立profile仍需另行处理。

如需重启，在 `sites/` 中运行：

```bash
XDG_CONFIG_HOME=/workspace/UndergroundBattle/sites/.wrangler/config \
npm_config_cache=/workspace/.npm-cache WRANGLER_SEND_METRICS=false \
npx wrangler dev --local --ip 127.0.0.1 --port 8102 \
  --persist-to /workspace/.private-validation/jc088-local-runtime-20261003 \
  --config wrangler.jsonc
```

完成的是源码、自动回归及本地可验环境；没有新增公网或独立玩家自然试玩证据。父线程单profile多席缺口不阻塞本次实现，但仍未解决。原中断四浏览器私有目录不在本容器，未索取或复制凭据；前次中止QA不算墓地通过，旧房线上兼容也不因本地合成回归而变成已验。
