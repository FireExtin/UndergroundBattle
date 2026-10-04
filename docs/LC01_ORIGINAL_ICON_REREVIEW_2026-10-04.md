# LC01 原图图标复核：普通调查2、先手额外调查1

本次针对整批候选 `c4d20b599be77a3e09a4bd8dc3e5e0bd272e0620` 的独立审查疑点，重新实际查看原始 `resource/ymsj-fun.github.io/cards/LC01 西比尔.jpg`（400×560，SHA-256 `a0defb07b43463cc882e15b993c73eda22ea77ca39062da71e6518a365cc5bd6`）。同时将原 JPEG 直接嵌入 PDF，通过裁剪窗口与7倍坐标变换放大左侧调查图标，没有修改源像素、没有生成替代牌面。

放大后，从上到下明确为 **白底、黑底、黑底**。因此这张锁定原图不是“上面两个白底、下面一个黑底”。手册原 PDF 第8页、印刷第6页的“临时能力图标”已实际查看：一般图标黑底，临时图标白底，临时图标只在操控者持先手标志时生效。按已核2V2团队语义，先手归属由当前controller的团队决定。

当前 `cards.json` 的 LC01 `permanent.investigation=2`、`temporary.investigation=1` 与此原图一致，**未交换印刷定义**。WM003 的先手调查1/势力1也未修改。正式牌面、候选生产实现及WASM字节全部保持 `c4d20b5` 的原值。

新增一项定向行为回归 `lc01_original_black_two_white_one_rear_two_first_three_exhaust_zero`，使用owner与controller不同的两席来源，分别轮换先手团队，断言：后手调查2、先手调查3、横置有效调查0。定向测试通过。将 LC01 暂时反转成普通1+临时2时，同一测试因后手实际1、不符合原图要求2而失败；恢复 `cards.json` 原字节后再次通过。

证据目录 `/workspace/.private-validation/lc01-icons-correction-20261004/` 包含原图放大PDF/PNG、手册原页、通过及反转失败日志和结构化证明。`cards.json` 恢复后SHA-256 `0901f13a1a1c371ff7bf26dbf159d4c112ff902922086a342f01d42b279b5451`；候选WASM仍为 `36d28923bf077edb23a339ad1c906b4171e09710aa4389309baf75b1c08fd631`。

此次只增测试与说明，没有扩机制、改WM003、push或Sites保存/部署。没有逐项重跑默认全量；整批最终必要回归留到独立审查的最终代码确定后。此前交付的c4d20b5整批生产字节及其默认全量证据仍有效，新增加的测试已有定向通过证据。
