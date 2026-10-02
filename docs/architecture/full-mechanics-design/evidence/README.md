# 原始证据与复现入口

审阅代码 `7b87aa8ef6cb1921fece09bdc16fe25613815773`；目录 `bada4758cf686e9988ecc51c3df3cf2cc7a50946`；仅一次后续固定增量 `a7572e87ab80c9f36fa0112919cce107daa05b43`。

本清单只把实际目视的 **29 个 PDF 页、4 张扩展说明图、21 个卡图记录**计为本次核看。原 PDF 渲染还包括其他定位页，但不把仅渲染未目视的页算进结论。来源哈希只用于证据定位，不是新增游戏签名/校验体系。

## 原页

表内链接直达仓库原始文件；页号均是 PDF 物理页（从 1 开始）。可运行 `python3 docs/architecture/full-mechanics-design/evidence/render_evidence.py` 在临时目录重渲染，需 PyMuPDF。

| ID | 原件与页 | 实际核看主题 |
| --- | --- | --- |
| BQ2 | [PDF 第 2 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=2) | 八色与中立 |
| BQ3 | [PDF 第 3 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=3) | 组件与基本区 |
| BQ4 | [PDF 第 4 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=4) | 目标/不能优先/独有/胜利 |
| BQ5 | [PDF 第 5 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=5) | 秘社及地区类型 |
| BQ7 | [PDF 第 7 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=7) | 设置/忠诚/换牌 |
| BQ8 | [PDF 第 8 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=8) | 桌面设置示意/资产有效属性 |
| BQ9 | [PDF 第 9 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=9) | 回合步骤/临时图标 |
| BQ10 | [PDF 第 10 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=10) | 三对抗/霸权赢区顺序/清理 |
| BQ12 | [PDF 第 12 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=12) | 资产/暗藏/现身潜伏姿态 |
| BQ15 | [PDF 第 15 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=15) | 优先权/LIFO/不可响应/额外费用 |
| BQ16 | [PDF 第 16 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=16) | 领域/拥有控制/封印/宿主 |
| BQ17 | [PDF 第 17 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=17) | 本方友方敌方/声明/替代与来源离场 |
| BQ18 | [PDF 第 18 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=18) | 公开隐匿/传闻/创伤/终止 |
| BQ19 | [PDF 第 19 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=19) | 灵体声望等关键词/闪烁/非资产/护盾 |
| BQ20 | [PDF 第 20 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=20) | 任务/计划/隐秘对抗 |
| BQ21 | [PDF 第 21 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=21) | 核心区/休整/失控/任务胜利 |
| BQ22 | [PDF 第 22 页](../../../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf#page=22) | 霸权2V2全部核心规则 |
| RM4 | [PDF 第 4 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=4) | 可选强制触发/玩家排序/重置 |
| RM5 | [PDF 第 5 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=5) | 响应堆叠/独有 |
| RM6 | [PDF 第 6 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=6) | 结附/多种宿主/封印生命周期 |
| RM8 | [PDF 第 8 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=8) | 效果计算临时图标/灵体分类 |
| RM12 | [PDF 第 12 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=12) | 牺牲/0伤/经典赢区顺序 |
| RM13 | [PDF 第 13 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf#page=13) | 资产空白/忠诚/抽牌/自身名称 |
| FAQ3 | [PDF 第 3 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf#page=3) | 费用忠诚/底牌入手/付费现身/来源地区 |
| FAQ4 | [PDF 第 4 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf#page=4) | 区域目标复核/伤害期间窗口/光环致死/墓地打出 |
| FAQ5 | [PDF 第 5 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf#page=5) | 多类型附属/目标移动/手牌能力 |
| PG14 | [PDF 第 14 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf#page=14) | 50张/同名3张/多色构筑 |
| PG21 | [PDF 第 21 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf#page=21) | 经典2V2不同于霸权 |
| PG24 | [PDF 第 24 页](../../../../resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf#page=24) | 秘社独立与无秘社的秘社区 |

## 扩展原图与代表卡

卡图观察只支持列出的结论，不意味着本次新做了全部整卡验收。初始20个记录加后续JC059；MSJZ02/ZHMSJZ02为同一物理两形态的两个归档记录，不能算两张可构筑牌。

| ID | 原件 | 观察 |
| --- | --- | --- |
| prelude | [原图](<../../../../resource/ymsj-fun.github.io/public/docs/序曲说明书.jpg>) | 遭遇角色、利用、恐怖 |
| intermezzo | [原图](<../../../../resource/ymsj-fun.github.io/public/docs/间奏说明书.jpg>) | 转变、式神、毁灭、闪烁、非资产、迟缓 |
| veil | [原图](<../../../../resource/ymsj-fun.github.io/public/docs/帷幕之后说明书.jpg>) | 计划、隐秘对抗、传闻、七张勘误版本 |
| starlight | [原图](<../../../../resource/ymsj-fun.github.io/public/docs/星光俱乐部说明书.jpg>) | 非地区上的势力标志与隐秘等级 |
| JC003 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC003 帷幕护卫.jpg>) | 目标角色或暗藏者，无本地区/敌方限制；2资源+横置是费用 |
| JC042 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC042 末日信徒.jpg>) | 牺牲费用；下一张正面指定类别减2；非现身 |
| JC049 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC049 深渊细语.jpg>) | 额外牺牲角色；底部两张入手不是抓牌 |
| JC058 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC058 坚毅的刑警.jpg>) | 现身触发而非普通进场；本地区目标暗藏者，可属任何玩家 |
| JC077 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC077 庇佑圣灵.jpg>) | 将受伤害触发；另一名本方非灵体角色转伤；多重时序未裁定 |
| JC016 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC016 执行部精锐.jpg>) | 敌方角色条件不包括暗藏者；临时调查+永久势力 |
| JC070 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC070 虔心修士.jpg>) | 1费用、白忠诚2，公开、声望 |
| JC076 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC076 悟道僧人.jpg>) | 现身抓1，声望；非一般进场 |
| LC23 | [原图](<../../../../resource/ymsj-fun.github.io/cards/LC23 安格鲁，“荆棘”.jpg>) | 金名；横置自身与目标是效果，不是费用 |
| MSJC01 | [原图](<../../../../resource/ymsj-fun.github.io/cards/MSJC01 帷幕守望.jpg>) | 起手6/25黄限制、3费行动条件、4费本盘一次检索 |
| DQJC116 | [原图](<../../../../resource/ymsj-fun.github.io/cards/DQJC116 香港.jpg>) | 所有玩家同时展示并入手，再洗牌 |
| JC028 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC028 鲜血奴仆.jpg>) | 费用牺牲自己后作为鲜血附属；需要付款后实体引用 |
| JC085 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC085 复生行尸.jpg>) | 墓地当手牌打出与有效领域条件；非先回手 |
| JC099 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC099 徘徊的鬼魂.jpg>) | 现身封印顶牌；ready连续移除带封印敌角的领域 |
| MSJC14 | [原图](<../../../../resource/ymsj-fun.github.io/cards/MSJC14 哨兵.jpg>) | 锁定标志与后续移动；不足以推定通用私人检视权限 |
| WM071 | [原图](<../../../../resource/ymsj-fun.github.io/cards/WM071 贝兰丹娜.jpg>) | 转生印刷句无改为；洗回事件前后不得由转写推断 |
| CQ12 | [原图](<../../../../resource/ymsj-fun.github.io/cards/CQ12 深渊惧影盖罗涅.jpg>) | 先兆展示成本/准备/每回合一次；手中本地区锚点疑问 |
| LC15 | [原图](<../../../../resource/ymsj-fun.github.io/cards/LC15 莲月合一.jpg>) | 五领域、指定姓名关联、永恒缺定义 |
| MSJZ02 | [原图](<../../../../resource/ymsj-fun.github.io/cards/MSJZ02 盛夏王廷.jpg>) | 盛夏正形态；条件转变/生成 |
| ZHMSJZ02 | [原图](<../../../../resource/ymsj-fun.github.io/cards/ZHMSJZ02 凛冬王廷.jpg>) | 凛冬转变形态；准备强制牺牲及变回条件 |
| JC059 | [原图](<../../../../resource/ymsj-fun.github.io/cards/JC059 安全保卫部门.jpg>) | 后续增加独立目视：其他友方角色加防；代码仅同controller错误 |

## 源码与运行证据

| 边界 | 固定 SHA 源码 |
| --- | --- |
| 原子apply/阶段/投影 | [rust-game/src/engine.rs:321](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/engine.rs#L321) |
| 有效防御当前漏队友 | [rust-game/src/engine.rs:265](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/engine.rs#L265) |
| 有限声明/验证器 | [rust-game/src/rules.rs:177](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/rules.rs#L177) |
| 持久化Frame/Pending | [rust-game/src/model.rs:407](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/model.rs#L407) |
| 效果游标/目标guard | [rust-game/src/resolution.rs:1](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/resolution.rs#L1) |
| native重复命令契约 | [rust-game/src/service.rs:344](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game/src/service.rs#L344) |
| WASM不透明状态接口 | [rust-game-wasm/src/lib.rs:1](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/rust-game-wasm/src/lib.rs#L1) |
| D1 CAS/nonce/batch | [sites/src/store.mjs:1](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/sites/src/store.mjs#L1) |
| Worker意图收据 | [sites/src/service.mjs:15](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/sites/src/service.mjs#L15) |
| 旧房间内核路由 | [sites/src/kernel-router.mjs:1](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/sites/src/kernel-router.mjs#L1) |
| 当前按钮式牌桌边界 | [web/src/game/Table.tsx:1](https://github.com/FireExtin/UndergroundBattle/blob/7b87aa8ef6cb1921fece09bdc16fe25613815773/web/src/game/Table.tsx#L1) |

运行结果与完整命令见 [checks.json](checks.json)，测试日志保留原始输出。首次直接执行 `validate_specifications.py` 没有 CLI 入口、没有输出，**不计作一次成功校验**；空日志未收录。通过 `validate_coverage.py` 与 unittest 的实际结果另见对应日志。

验证入口不会安装工具或连接在线服务。`run_checks.py` 使用准确 SHA 的临时副本；`verify_design.py` 只检查证据文件与矩阵引用，不证明游戏规则正确。

## 后续目录输入

已读取 a7572e8 的 mechanism 增量和 index totals：52机制、68整卡规格、4blocked、685总记录；尚未整卡核验617项含13缺图。没有重审新增全部规格，也没有复跑该分支46测试。其“现有29定义”描述针对目录自己的基线，不能覆盖本次代码的30定义。JC059友方缺口由本次对旧基线原图与代码另作直接核验。

其他未裁定项见 [规则账本](../03-rule-semantics.md)。不把经典模式、旧卡同名版本、目录转写或其他TCG规则自动并入当前霸权规则包。
