# JZ46 本地验证证据（待独审，未发布）

`verification-summary.json` 汇总最终范围；`candidate79-source-binding.json` 绑定123个编译输入及额外30个生成器／支持文件。原始整卡10正向＋34坏档在仓库 `rust-game-wasm/tests/jz46Native79.fixture.json.gz`，保留原JSON字节和哈希。旧433当前79原文在 `/workspace/jz46-evidence-20261016/current79-old-mechanisms.json.cases/`，未提交2.502GB状态文本；这里的 `current79-old-native-index.json.gz` 保留完整原始索引字节和逐份SHA256。

日志`.gz`均为原字节gzip封装，未删失败、重写opaque状态或改身份。Native首轮失败与最终完整通过分别保存；前端／Worker按整轮与限定范围逐份保存，不能相加称一次全绿。额外HTTP恢复测试的输入定位和状态码预期错误也保留。四席浏览器JSON及原始截图来自本地实际HTTP／workerd／WASM／D1及生产GameApp，初始布局和测试时钟明确合成；它们不计线上试玩。

最终生产清单包含三个精确WASM的哈希，没有提交编译JS或WASM。原ABI78/77实际输入仅在 `/workspace/undergroundbattle-build/frozen-kernels/`；缺失或不同时验证须失败，不下载、不重建替代。原云端普通浏览器18:33:55 TLS失败另存JSON，未到达线上版本身份或任何游戏。

源码与最终Git提交的绑定将记录在 `source-closure.json`。完整结论见 `docs/JZ46_SERIAL_REVIEW_HANDOFF_2026-10-16.md`。
