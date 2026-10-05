# MSJC07 经审增量上的 JC103 同一纠正（私有，2026-10-05）

从父端已审接受的独立 MSJC07 增量 `468e75858034c8d76bf3bba6213e4f48953c0a90` 继续，只应用已审 `134f1286891ada7722719cc4f002ddcea006b40d` 的同一 `painter.response_policy = ResponsePolicy::Immediate`。JC112 既有 Immediate 保持不变；引擎、结算器、费用支付、独立死亡触发、附属离场、Room 和秘社程序均未改。原 MSJC07 Library 包及其实际原产物不改写。

新固定标识为 `rust-v0.2.26-msjc07-resource-policy-candidate`，卡池仍为 `limited-v2.23-msjc07-candidate`，87普通定义／3秘社。父端原图审定的 MSJC07 黑牌至少25、起手6、3+横置先手抓牌、4+横置检索黑色独特牌每局一次保持原实现。原实际26的全部五个生成文件另冻结为 `legacy-v0.2.26`，WASM1,974,053字节、SHA `c187a493d3e32f82c8609cb203e12ffb1661fbe348260f6e010254bb9af395af`。

本纠正实际 WASM 为1,974,208字节、SHA `79b0a9fa5e9f2acc015230ec3cd42e7f0fae41d8e8f720be7e6c81db52cd824e`。定向 native：紫牌12、MSJC07 8、费用死亡独立触发1、JC042即时减费恢复1，共22通过。新增 Room 真实声明同时验证 JC103 横置和 JC112 牺牲后即时减费、没有自身响应窗口及四席恢复。费用死亡用例仍保留自身可响应声明和可恢复选择；它是原语夹具，不表示 JC112 印有死亡能力。

该实际 WASM 与 fresh native 完整比较：MSJC07 17完整正文869转换／3464投影，紫牌26完整正文317转换／1268投影，共43正文1186转换／4732投影。MSJC07的17正文在仅归一化引擎标识后，与原已审完整正文全部逐字节相同，包括四席自然规则案例651转换／2598投影。当前真实 JC103 的 applyRoom 声明立即获得修正且四席无窗口。

六个受影响前端文件／25项测试通过。一次首次测试调用重复传入 --run，被 Vitest 参数校验拒绝；只修正调用命令后通过，未据此改产品代码。没有本轮默认327全量、全量前端或公网 UI 声明；按父端要求，与下一批统一执行最终默认全量。本纠正未部署，当前 Site 发布候选仍排除 MSJC07。

完整当前43正文及索引、实际产物、定向日志与资源记录在 `/workspace/.private-validation/msjc07-resource-policy-20261005/`；提交内摘要在 `docs/evidence/msjc07-resource-policy-2026-10-05/`。父端所有 `docs/factions/card-specifications.json` 未改，SHA仍为 `797b502b7d5261ce467f24308f15e7b78f25def6067701f9d4fadff9958a2d68`。
