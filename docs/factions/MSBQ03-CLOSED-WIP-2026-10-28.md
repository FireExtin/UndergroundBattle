# MSBQ03 圣甲虫药业封闭实现

基线a8b4b80afb7de8121a5aa5db04e7c6e2733d0b75。正式黑色秘社，原图`resource/ymsj-fun.github.io/cards/MSBQ03 圣甲虫药业.jpg` SHA256 efeaa853e66ff3d49c2fe5dc5834b9799238e9833d1285626595078ddbe0838f，Society16/17，金名，企业，起手6，副名斯卡拉跨国集团。已直接查看原图，第一行动费用明确1不是旧定位数据2。

验收：黑白中立构筑且无名尸体不限张；行动1/横置，先声明本方产业附属目标，响应后选择本人墓地尸体封印（不是费用或预先目标）；持续按每个本人产业载体上实际封印的印刷角色牌数加永久势力，控制权/离场实时变化；行动3/横置每局一次，按副名搜索而不额外限定附属，展示后入手再洗牌。复用既有封印事件、检索、费用及次数，不重写工业卡语义。

原spec的sealedChoicePrintedFields是明确卡文授权的印刷类别读取边界，不是需要把全部封印牌恢复普通角色属性的规则；既有BQ093和JC066已有受限印刷读取。未依赖生成角色或BQ095总部基础层。保持supported=false，自测不替代独审，不重试被拒审查、不推送/合main/部署。

失败历史：01发现69，67过2失败；测试把JC001误当非角色，实际应计3；自然witness误用不存在的attach命令。02发现70，69过1失败；自然调查奖励需完整预测顶/底分配，witness错误使用普通selected。修正测试输入与实际命令，未放宽生产规则。03发现20，20过0失败/忽略、2726过滤；含7项MSBQ03及JC008回归。自然构筑seed12、481条Room命令，真实重复请求和四座位视图，开局后无布局/资源/回合修改。01/02日志与模块/测试副本保留；不声称这两轮拥有完整编译输入快照。03起382个Rust输入冻结。

本检查点开始累计全库及同源WASM/UI/Worker验证，结果待定。两个目录枚举仅插入已有实现且关闭的MSBQ03；原目录摘要、236公开准入与未实现负例未降低。

03默认WASM构建及481命令/1924视图/481重复比对通过；UI03 55过1失败（测试误点普通卡DOM，地区附属实际使用独立控件），UI04修正后56/56通过；前端构建通过。之后自查伪造“MSBQ03待选＋外来帧＋越界行动者”发现WASM RuntimeError: unreachable。保留原损坏快照与复现脚本；这是人为损坏快照的恢复缺口，不声称正常点击可触发。为修复该缺口停止即将失效的03全库和Worker；全库停止时有851条完整单行ok记录、0已报失败、1ignored，但未完成；带额外输出的用例不能靠该进度计数推定总执行数；Worker未完成。停止不是权限拒绝，也不能记PASS。

04新增Native红测0过1失败，精确复现index out of bounds（4个座位却索引99999）。05在读取待选候选区域前要求原MSBQ03来源和有效行动者；同一反例及Game/Room校验通过。05相关84/84、0失败/忽略、2663过滤，48.69秒测试、119.14秒含编译；新Native自然链重新导出，未给03输出换签。接下来累计全库及WASM以05的382输入冻结清单为准。

## 最终同源收束

被测机制源码`0edc228320a1b882f2e559466fcabb23ad49a5c3`，382个Rust输入全程冻结。原full-05进程正常结束，实际发现2747：2746通过、0失败、1ignored、0过滤，1841.34秒（回执1841.55秒），子进程峰值RSS851280 KiB。定向84与全库重叠，不相加；03中断不计通过。期间模型容量提示没有重启原full-05进程。

命令：`node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- --nocapture --test-threads=1`。同源导出`MSBQ03_NATIVE_OUT`、`BQ063_NATIVE_OUT`以及`SENTINEL_WORKER_WITNESS`、`SENTINEL_PLATFORM_WITNESS`均指向新`/workspace/msbq03-validation/full-05/`下对应文件。两个witness主体实际执行：`sentinel_local_worker_room_witness`1场景5命令，`wm044_local_platform_room_witness`2场景6命令；它们是准备布局，不是自然开局。唯一ignored仍为`bq001_natural_room_tests::bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment`，原因`Internal candidate catalog variant only; mainline admission remains closed`，没有擅自开启。

05默认WASM构建通过；MSBQ03新Native自然链481命令/1924四座位视图/481实际重复回执全部一致。全库再次导出的MSBQ03文件与定向输出逐字节相同，原始116975217字节SHA256 `19888b359f8a755f8098ee3bd4b323f9d09bc9ca07be407754f94105c6c61a07`。BQ063在同一源码上新导出并比对332命令/1328视图/332重复，含318自然命令及两个各7命令准备布局。未给旧夹具换签。

当前完整源码清单SHA256 `7db16c83d9b03a5a25f83ddb4bb130b6f3f4b1aa4a7f439cfb95b47bf75171d8`，WASM SHA256 `f1d9dec5b104c8d4967454a6545761a5e258537cb45c81670565ed53c633f1ca`。损坏外来帧WASM复现从03的RuntimeError陷阱变为05明确规则拒绝；Native同一红测已通过。UI05 4文件56/56通过，原图/public/index、封闭构筑和真实行动/选择保留；Worker05 1/1场景通过，481命令、481冷D1重开及终态旧收据复取，0失败/取消/跳过，600.52秒、峰值RSS834756 KiB。当前前端类型/生产构建通过，13.02秒。

Rust/Cargo1.90.0、wasm-bindgen0.2.104、Node24.19.0，构建/测试线程各1，离线锁定依赖，三个锁文件未改。峰值RSS：定向含编译2709372 KiB、WASM1630248 KiB；cgroup观测包含文件缓存，不等同进程RSS。全程云端，没有占用用户本机模型。

未运行8个独立集成目标、无关web/sites全套或真实浏览器试玩；独审未执行且既有拒绝未重试。生成触手的离场/翻暗/封印等裁定仍未知，见`RULE-DECISIONS-MINIMAL-2026-10-28.md`。MSBQ03保持关闭，正式公开新增仍3/20，不发布、不合main、不推送。此处源码身份/规则清单和自测均不替代完整批次独审。
