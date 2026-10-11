# XQ24：持久效果队列响应绕过的限定修复

修复父候选 `107fc0634db5fee76cc795d563b1d98e687915c6`。
工作区 `/workspace/UndergroundBattle`，分支 `codex/xq24-riot-police-review-20261011`。
开始时 HEAD 精确匹配、工作树干净。父会话转交独立复审REQUEST_CHANGES与P1复现；
本任务只修复XQ24 entry帧的持久容器准入，不重写全引擎或扩大其他灰牌的限制。

## 根因与真实复现

`Game::validate_gray_expansion_state()` 原来对 `Effect::Frame` 只调用完整程序校验。
被接受的XQ24 `entry-hide-cheap-characters` 帧本身的来源、程序、cursor=0及Unchecked守卫
全都合法，故将它从stack移入effects仍能恢复。同步清空pacing.window后，Room的堆顶/窗口
关联检查也通过。下一次普通Game pass的drive直接执行该effects帧，绕过双方响应。

按独立复审交接，读取 `sites/test/fixtures/xq24-native-v064.json` 的duel-accept
`steps[5].expected.state`，仅移走stack item、把原frame放入effects并清空窗口。
使用Python保留精确u64，未经JavaScript重序列化状态。真实旧生产WASM确认：
合法原状态Game pass返回invalid_action；非法状态view成功、窗口为空；
Game pass被接受、revision10→11，地区2暗藏数量0→1。teams按能力定位对应接受步，
也复现0→1（revision16→17），不是把两人步下标错误套到四人场景。
复现报告 `/workspace/qa-xq24-container-20261011/logs/baseline-wasm-reproduction.json`。

## 限定修复

在现有 `Effect::Frame` 完整校验之后，仅拒绝来源为XQ24且ability_key为
`entry-hide-cheap-characters` 的持久effects帧。该可响应单步程序在双方让过后被推到
队首，并于同一命令的drive内全部执行，没有帧内选择或可保存的中断边界。
因此必须保存于正常响应stack；已接受但移入effects的快照属于非法状态。

拒绝发生在Game/Room恢复入口，生产WASM的view、applyRoom、pollRoom均通过decode
在行动前拒绝。没有通过改变普通pass合法性掩盖非法恢复，也没有添加运行时通用容器限制。
保留XQ24可选声明、正常stack与完整响应后的状态；其他灰牌的Frame/Declare准入逻辑不变。
当前未发布Engine64候选的版本身份与卡池不变；没有改schema、规则裁定、卡图或客户端组件。

## 回归与证据

先增加Native非法恢复用例，再修复。旧Native红灯为“XQ24 entry frame in effects must fail
Game restore”；旧生产WASM红灯为Missing expected exception，记录均保留。
新Native测试用duel/teams、u64::MAX、真实付费/接受/PassResponse命令构造合法路径，
导出4份非法Room快照：移走stack并清空窗口，或保留stack而追加重复effects帧。
每份必须在Game和Room恢复时失败；真实正常响应的各边界逐次恢复成功。

`sites/test/fixtures/xq24-container-invalid-v064.json` 是实际Native导出，225037字节。
首轮RED与GREEN的Native导出逐字节一致。随后为HTTP路由改用24位hex测试房间标识，
单独重跑该Native测试并重新导出最终fixture；整个opaque状态字符串不重序列化。
`tools/cards/check-xq24-containers.mjs` 使用实际生产WASM，对4份输入的3个恢复入口断言拒绝，
并核原状态普通Game pass仍被拒绝、未到期poll不改变原状态。
它还逐项比对原12条Native连续场景的117次合法命令及所有座位视图，涵盖接受、拒绝、
双方响应、来源移动/消灭、原地区替换/空缺、暂停恢复及日志；沿用准确标注的预置场景。

| 本轮执行 | 结果 |
|---|---|
| Native XQ24专项 | 16/16通过 |
| Native gray专项 | 47/47通过，其他灰牌及锁定恢复回归 |
| 生产WASM容器与合法场景 | 4非法快照×3恢复入口拒绝，12场景/117合法命令/370座位投影一致 |
| 传统完整Native/WASM ABI | 433场景、24403转移、96975投影、319拒绝命令、42拒绝建牌、1报价一致 |
| 真实XQ24 Worker/D1 | 原12/12连续场景通过；新增HTTP测试1/1，4次重开、8次恢复拒绝、零状态/日志写入 |
| 实际XQ24组件UI | 9/9通过 |
| 两套类型检查与本地dry-run构建 | 通过，147客户端资产，未部署 |

最终fixture版本的Native容器测试单独1/1通过；此前16项中的其余15项未变动。
WASM容器检查已接入标准 `rust-game-wasm/verify.sh`，以后标准核验自动执行。
本轮复用了前轮实际Native全量语料并重新执行整个生产WASM对照，没有重复生成2GB语料。

首轮新增HTTP测试因Native房间标识不是24位hex而返回404，尚未到恢复入口。
检查既有路由与令牌格式后，仅将测试房间标识改为24位hex，并使用符合原格式的合成hex令牌，
重新Native导出及HTTP检查后通过。另为两种模式使用独立测试数据库邀请字段，保持opaque存档不变。
首轮组合命令结果是12项合法场景通过、1项新HTTP测试失败；最终只重跑修正后的新测试1/1通过，
没有把首轮失败改写为成功，也没有修改路由、认证或HTTP业务代码。工具/外部账号没有发生认证拒绝。

机器摘要、日志SHA与输入SHA见 [本轮执行证据](XQ24_CONTAINER_FIX_EVIDENCE_2026-10-11.json)。
生产WASM为2670198字节，SHA256
`8a306d040cfebdc92f26f9a4f7f225bd9e7ab20fb700080d08ed954bcb1c7b2b`。

可重复运行的主要命令（锁文件和既有工具链不变）：

```sh
node tools/build-storage.mjs -- cargo test --locked --no-default-features --lib xq24 -- --test-threads=1
node tools/build-storage.mjs -- cargo test --locked --no-default-features --lib gray -- --test-threads=1
bash rust-game-wasm/build.sh
node tools/cards/check-xq24-containers.mjs
npm --prefix sites run build
(cd sites && node --test --test-concurrency=1 test/xq24-container-restore.test.mjs test/xq24-integration.test.mjs)
npm --prefix web test -- src/game/XQ24Integration.test.jsx
```

原候选945/842/103的全量结果保留在其原说明中，属于前一提交的证据，不能写成本修复提交
重新执行的全量结果。本轮核验以表格中实际执行为准。

## 交付边界

待独立复审及父会话验收，未发布、未部署，父试玩Engine63/v3不变。
正常后续提交仅在本次专用审查分支交付；不改main，不强推、不建worktree，不重试历史拒绝专项或Library认证，不操作Air。
工作区保留原失败与复审记录。执行日志与机器摘要在 `/workspace/qa-xq24-container-20261011`。
