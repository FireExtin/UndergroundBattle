# 可选防御值 null 呈现小修

输入为父线程已独立审查的候选 `d8707123ab340ef2fd819ab66be8517fb2efb880`，由父线程直接交付完整增量文本。指定 Library ZIP 在本执行环境通过当前正式流程两次准备、三次下载均失败，未声称实际获得该 ZIP 或核验其 SHA。父线程随后明确授权文本作为执行来源并解除跨机 ZIP 前提。生产 CardTile 的基线 blob 为 `f81bada8cbffeec73b0cb78fc1d5a14efb77a262`，修后 blob 为 `530f5e8b7473c155adcc4b3163ec1dc562b42ccb`；新测试 blob 为 `0925ef9d7211e58876181a09b5c108b5f3243c03`，均与所交付完整增量一致。

输出从 Site22 实施 `d143048322e10ac77df73a3125b78fab508c1804` 创建本地私有实施分支，向同一 Sites 源码仅应用 CardTile 和 OptionalNumeric.test.tsx 两个功能文件，不覆盖候选旧基线目录。生产改动新增两行、删除一行：先按 nullish fallback 取防御，再仅在值为 number 时显示；当前/印刷比较亦分别要求 number。合法 0 保留，缺失防御不打印 null 或伪造 0。本说明只保留在实施仓库，非额外 Site 功能变更。

13 个新增用例在旧源码上 8 失败、5 通过，复现手牌/再调度/阅读及 null/0 问题；小修后完整前端 298 项、TypeScript、Vite 构建和 Worker 23 项全部通过。十核 JS/WASM/package 与 Site22 逐字节一致，规则/牌池/引擎保持 `hegemony-pdf-v1 / limited-v2.7 / rust-v0.2.10`；普通卡池 49 项不变，路由、CSS、50 张原图均不变。本次未重编译规则核；既有默认 Native/WASM 验证结论由完整相同字节支持。

边界：父线程已目视审查候选修后阅读截图；本轮只进行指定的本地集成检查，不能代替发布后公网浏览器刷新验收。正在进行的房间 BEB96C32AA9A 从未被本执行器访问或操作；父线程负责发布后刷新。无原 GitHub 分支推送、无新增代理、无数据库迁移或版本升级。
