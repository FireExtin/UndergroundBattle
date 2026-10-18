# JC041 原地区UID恢复P2复审材料（WIP，所列验证完成）

分支 `codex/jc041-p2-source-region-review-20261018`，审查基线 `f3dece927c733a761cf78d62514c6299fab280fa`，Rust修复提交 `6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317`。当前完成独审指出的矛盾UID修正与下列集中验证，交父会话串行复审；未发布JC041、未合入该批。已审游戏main仍224a619，原审查分支f3保留。

精确反例来自原夹具water-actor-0首个activate后Room：revision/game.version9，stack/frame i252，来源i251/JC041、owner1/controller0、face_down=true，原地区0=i240/DQJC107。只把source.source_region_instance从i240改i251。原Room31,476B/SHA ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c。Rust类型化克隆并逐字节撤回核对，JS只传递不透明字符串。额外覆盖已知手牌、已支付资产和另一地区槽的UID，共四份坏状态。

产品最小修正仅jc041_source：拒绝来源自身UID；拒绝仍在错误地区槽可辨认的UID（含vacant元数据）；遍历原有卡实体清单时拒绝可辨认的非地区类型。未要求历史原地区仍存在或原槽非vacant；保留原来源定义/owner/明暗检查。没有改卡文、费用、普通派遣/现身、触发时序、数据准入或Wire/池版本。

| 验证 | 实际结果 |
| --- | --- |
| 修前Native/WASM/HTTP | Native1/1明确四状态Game/Room接受；原准确980e核下8/8，四状态6入口均接受、各10 HTTP200，pause各1命令/1日志，重开原收据仍返回；四组合法续行准确 |
| 修后Native | JC04114/14，原13行为及四新拒绝/四合法边界；真实137输入见证导出 |
| 修后集中WASM/HTTP/兼容 | 同一命令29/29；四P2状态各6WASM入口拒绝、共40 HTTP400，D1重开前后room/commands/journal无额外写入；四合法场景56命令逐条重开、全状态/日志/四席视图与原收据；12纯路由、11核同isolate真实HTTP、2身份、4来源、2原86冻结均通过 |
| 常规Native87→WASM | 原10场景155命令9拒绝660视图35坏档完整对照；除137来源绑定外原全部payload逐字节不变 |
| 相关前端 | 实际2文件77/77（JC041与原86），未改前端实现 |
| 生产及本地时钟包装 | 双TS/Vite/11核Wrangler dry-run通过；183生产/22generated/14本地测试文件，168客户端不变，真实生产Date.now；11个测试WASM为准确生产原件的新普通磁盘硬链接 |
| 产物/原件审计 | 137输入逐项匹配6b准确Git blob及工作树，旧183字节保留或绑定不可变f3 Git原件，旧77–86及d58b Native86不变；四份坏字符串、四组合法完整轨迹前后逐字节相同 |

合法范围包含原真实Room直接续行，以及来源移动、原地区UID真正消失、原槽vacant保留元数据三种明确Native初始布局边界；之后均为真实响应、暂停/恢复、本人弃牌命令。布局变更不冒充可玩的移动/换地区响应。四组每组14命令，全状态/日志/四席视图与收据在实际本地workerd/SQLite D1重开后继续准确；不算线上试玩。

修前重放可在当前修后源码上进行：探针将原137编译绑定比对不可变f3产品及6b中当时测试专用源码，仍只使用原980e实物；新增仅source-self一项重放烟测1/1，验证可重复读取四入口/轮询/写入/重开收据，不能与原8项相加或说重跑了整8项。原运行时Room另以固定SHA测试夹具读取，不伪称第138个编译输入。最初专项编译因String引用元组比较错误0项执行，修正后真实复现通过，失败原日志保留。

新WASM3325174B / `f2c1c2f38bf7eec1103c32afac955500d29ab875f4c9039e1a73a787dc6568ed`；新Native raw SHA `9655bf2e2d84669783d5d2d01da4d9728470cc13cb909a26bc839014de1c7b77`；本批未发布87夹具1338486B / `23b52be411f7f1f692bb3a4fdfc2b009597990670199d1e93ead4acc7ff84bf1`。137真实编译绑定与源码Git对应，其他测试/构建输入的工作树SHA和预期Git blob在[实物审计](evidence/jc041-p2-source-region-2026-10-18/artifacts-after.json)。

空间约1GiB，因此本地测试包装用esbuild仅编译同一kernel/http源和固定时钟入口，对11个生产WASM建立新硬链接，普通磁盘上的同inode/大小/SHA已核验；不删旧文件、不改生产字节、不回退tmpfs，原1GiB启动守卫仍执行。新包装8个P2用例与11核真实HTTP实际验证通过。旧14文件Worker与生产server完整移存；168旧客户端165项来自f3准确Git原件、3个动态输出另存，183项均可逐字节还原。

按本次指示未重跑无关完整workspace、1391前端、173全批或59默认Worker；原f3的1327/1391/173属于修前历史，不替代本P2范围。14:13官方核对仍已审已部署v68，14:31本小时唯一普通访问仍ERR_CERT_AUTHORITY_INVALID，1GET/0POST/0线上游戏，立即停止，小时材料仅Git外复用。没有合main、Sites保存/发布、额外Actions或exchange操作。

[验证摘要](evidence/jc041-p2-source-region-2026-10-18/validation-summary.json)、[证据与复现](evidence/jc041-p2-source-region-2026-10-18/README.md)及[WIP时间线](JC041_P2_SOURCE_REGION_WIP_2026-10-18.md)保留全部自然回执和边界。原始材料 `/workspace/jc041-p2-region-evidence-20261018`，所有旧实物、合成DB与缓存保留Git外。剩余为父会话串行复审；本线不自启复审子代理，也不把本地组件或HTTP结果算线上游戏。
