# PDF_RULES_SYNC_2026-03-31

Purpose: explains the current repository task that re-aligned rule and guide markdown files to the upstream PDFs.

## What Changed

- `organized_content/rules/隐秘世界规则手册.md` 现已改为以 `resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf` 为真相源的重排版。
- `organized_content/rules/隐秘世界玩家指南.md` 现已改为以 `resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf` 为真相源的重排版。
- `organized_content/rules/隐秘世界勘误及释疑.md` 现已改为以 `resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf` 为真相源的重排版。
- `organized_content/rules/霸权说明书.md` 已改为权威源说明页，明确当前必须直接参考 `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf`。

## Method

- 对可直接抽取文字的 PDF，先抽取文本，再移除页眉页脚和明显噪声，最后写回 Markdown。
- 重排目标以“纠正顺序、改善可读性、保留原文”为主，不主动扩写规则含义。
- 对图片型 PDF，不伪造正文；在 OCR 未完成前，Markdown 只保留来源说明与当前状态。

## Current Limits

- 纯文本重排无法完整保留 PDF 中的图标、字体、版式和插图。
- `霸权说明书.pdf` 当前仍需完整 OCR 与人工校对，仓库里暂未恢复其全文 Markdown。
- `隐秘世界规则手册.md`、`隐秘世界玩家指南.md`、`隐秘世界勘误及释疑.md` 虽已明显好于原先的错序版本，但仍属于“文本重排版”，不是排版复刻版。

## Next Step

- 为 `霸权说明书.pdf` 跑完整 OCR，并以同样原则补齐 `organized_content/rules/霸权说明书.md`。
- 如后续规则工具开始依赖这些 Markdown，应优先读取文件头声明的真相源说明，避免把 Markdown 误当成比 PDF 更高的权威。

