# JC102 父维护规格同步

父已独立审查原图、纠正核和八个完整场景，接受 `e8ad5d3aa1134a12455cfcbad6157048e37fa87a`，并明确授权应用此前独立交付的两处规格建议。输入为原 JPEG `resource/ymsj-fun.github.io/cards/JC102 妖火.jpg`（SHA256 `842fb9f07e9b8afbc0cecd6c224972c5643bad9b710daa518faceb59fc9a3b5d`）及本轮实际放大的费用下方紫色忠诚箭头；原先漏读的来源和核纠正见 `JC102_PURPLE_LOYALTY_CORRECTION_2026-10-04.md`。

输出仅修改 `docs/factions/card-specifications.json` 中JC102的 `fields.loyalty` 为一枚紫色忠诚，以及 `JC102-03` 错误验收句为费用2、紫忠诚1、血脉领域1且血脉不能代替紫忠诚。其他卡和其他规格字段保持原值。此同步不改变已审生产核。

按本次授权只验证JSON有效性及递归字段差异，确保恰有上述两处；不重跑全套，不push、发布或访问房间。
