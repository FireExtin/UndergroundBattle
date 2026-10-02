# Site9 明确 review 缺口的红绿修复

基线为 `04d414ebb0982611bc02c8f657a9dfdca13a343c`。本切片只补明确验收缺口与暗藏阅读契约，仍使用 rust-v0.2.6 / limited-v2.4。美术呈现以后补，原图放大阅读保留。

真实 `Window::Win` 回底测试现在让四个 owner 各自提交不同顺序，核对完整尾牌序、原牌库前缀不变、每张新身份唯一，以及 owner/controller、正反面、横置、伤害、创伤和护盾重置；所有选择提交前地区与附属保持冻结。隔离副本故意反转最终回底循环，强断言失败；正式代码通过。默认 native/WASM 导出加入此前只在切片参数里运行的真实赢区 fixture。

暗藏投影按当前 controller 授权，组件原先按 owner 擦除会挡住合法资料。现在卡片、悬停、选择阅读、原图弹窗保持相同授权。合成控制权初态后，通过正常 JC063 潜伏流程验证 controller 得到正面、owner 不得正面；现行卡池没有可自然完成控制权转移的牌，不把此测试称为自然整局。隔离 UI 副本恢复 owner 误判，卡片和原图两项测试失败，正式前端157项通过。

原生91项（60单元+31集成）与真实 Worker/D1 15项通过。前端构建和类型检查随 Worker build 完成。当前 WASM SHA仍为 `d4a9d88fe3c44d94da938c0351a4f1654d72514568c9f927fecd6d2f9f3ded3f`；旧核1–5保留，不迁移用户房间或清数据。

完整验证命令均在 `/workspace/hegemony-v026-review-fix` 执行。一次准备命令误在 v027 WIP 目录执行，其日志不作为本切片证据；已明确 cwd 重跑本切片。证据仅收正确目录的输出与隔离反转结果。

原稿尚未明确 JC016 撤回宿主与 BQ022 回收附属在赢区的优先关系。本次仅登记，保持现有规则，等待定点补证或用户裁定，不将这个组合算入完整机制验收。测试绿不等于全部678张机制实现。

下一切片 JC084/JC085 的有限条件图标与墓地正面出牌保存在 `codex/hegemony-v027-grave-play-20261002`，不随本修复解锁。v027 独立原生/WASM/七核/前端及双席479次正常UI整局已验；该整局自然发生墓地出牌并以新身份正面入场，最后6:9。四套零中立研究候选仍未解锁。独立四席 v026 整局继续验收；已有双席 v026 整局382次正常UI、8:5。DB、身份与手牌截图留本机，不上传。

默认 WASM 对照585步、1865席位投影、1报价和6个拒绝请求全部一致。公开证据在 `docs/evidence/v026-review-red-green-2026-10-02/`，红测试是隔离副本故意制造的失败；正式工作树全绿。部署版本通过后追加。

发布确认：实施e1e2dea581fbb97f1c2a64f7133179d50f1dd4d8；官方bundle构建、验证、推送和打包的Site源码5889aeba0fd18e6a3ccdfaa18f5d266cdccdf7df；Site10，deployment appgdep_6ac01892c1b48191938b156611457e80成功。原公开audience revision2与env revision0保持。发布后原匿名房主profile正常恢复房45845084f946f770e7aaf26d、lobby revision1，加载新JS index-NdWStdMt、健康端点200/engine6、原BQ022图SHA逐字相同，0游戏POST、无浏览器或传输错误。首次只读探针把health字段误写为engine，实际契约为engineVersion，修正后通过；未修改服务或重试游戏请求。

待验收双席入口：https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site/?invite=0ADD203A461C ，仍只有原房主一席，原身份保留，未分享或伪造座位凭据。四席满桌75038039090bef579f32b5fb和旧C26房不动。v027新机制WIP已经合入本修复，最新验收记录随独立分支保存，不在Site10启用。
