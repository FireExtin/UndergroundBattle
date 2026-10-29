# 生成触手：项目裁定基础层 WIP

基线：31479054779fb31e7380a9b9b413f4ecb813fc71。裁定来源见 RULE-DECISIONS-MINIMAL-2026-10-28.md；不是官方勘误。TK010 保持 supported:false，不计正式新20张完成数。

原图 docs/factions/recovered-originals/TK010 触手.png：红色指示物角色，突变体/器官，防御1，鲜血领域；本方核心防御+1，同地区本方核心存在时临时战斗2。没有“触手”子类，故 BQ048/BQ063 改用明置角色的印刷名称识别，不能给TK010捏造子类。原PNG直接复制至public，双索引保持原字节。

本层：普通离场不进入手牌/牌库/墓地，先执行原宿主关联清理；翻暗保留，封印保留为空白封印实体，封印释放到手牌时因禁止进入实体牌区而消失。死亡仍保留死亡观察，但没有虚构墓地实例。其余牌不适用消失规则。构筑拒绝TK010，即便调用方提供私人候选目录；恢复拒绝实体牌区里的触手。闭合生成入口尚待BQ052工程接入，不能把当前准备布局当自然生成证明。

## 已执行

同工具链Rust1.90.0，offline/locked，CARGO_BUILD_JOBS=1，RUST_TEST_THREADS=1，通过tools/build-storage.mjs。

- tentacle-red-01：4失败，实际原因是首次加入TK010目录后尚缺类型化规则声明；没有到达生命周期断言。保留，不冒称业务反例红测。
- tentacle-green-02：4/4通过。
- tentacle-targeted-03：40/40通过（触手9、BQ048/BQ063、JC066）。
- tentacle-targeted-04：65发现/65通过/0失败/0忽略，2693过滤；覆盖触手11项以及BQ048/BQ063、JC066、死亡观察。与前轮重叠，不能相加。
- 04命令：node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- tentacle_tests bq048_tests bq063_tests jc066_tests death_observer_tests --nocapture --test-threads=1
- 04耗时100.59秒，测试38.41秒；子进程最大RSS2705188KiB，cgroup采样峰值3477344256字节。
- 准备布局Room轨迹经真实transition、恢复、四席视图及实际重复命令，属于组件/协议验证，非自然开局/浏览器试玩。

## 本检查点尚未执行

BQ052准备生成、三触手牺牲支付和随机核心进场自然链；同源WASM、UI/Worker；累积完整Native库。未执行独审，既有审查拒绝不重试。没有推送、合main、Actions或部署。后续结果应追加真实源码身份，不覆盖上述历史。
