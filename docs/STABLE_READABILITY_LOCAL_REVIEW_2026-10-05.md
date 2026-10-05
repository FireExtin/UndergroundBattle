# 两处stable可读性重构，独立本地待审

输入为父会话已审游戏提交 `aa1617a1ede90e5ea9de1eadf680b0bb74bda418`，以及先前三处Rust比较的stable建议。本工作在独立工作树 `game-stable-readability`、分支 `codex/hegemony-stable-readability-air-20261005`，与已发布Site31源 `65c9af2` 分离。

只改两处生产源码：`blue_private_choice` 用let-else和非actor提前返回，私有预览的生成表达式保持；`validate_ability` 把原JC032完整定义相等和旧host grant形状命名为两个布尔条件，仍仅接纳其一。limit=2、activation_only、无event、精确cost/op及空modes要求逐项保留。既有闭合定义、移植拒绝和mode guard不变；未改model/serde字段、恢复协议、卡牌规则、冻结kernel、Cargo锁或工具链。

Rust1.90/edition2021离线针对性验证：blue_相关19项通过，含闭合变种/移植拒绝、私有顶六/零命中、恢复回执；旧JC093两次限额/非法费用原子性与清理、有限准入变种两项单独通过。三个原始日志逐字节保留在 `docs/evidence/stable-readability-2026-10-05/`，任务根 `readability-review/` 原记录也保留。没有重跑已经完成的Site构建/前端/浏览器验收，也未重新生成WASM。

生产diff仅两个文件，19行增加、17行删除；本说明为项目每任务新增短文档要求。父会话已独立检查提前返回与DeMorgan命名条件逻辑等价，并接受局部可读性改善，现按授权本地提交。没有GitHub推送、Site源同步、再次发布或部署；原WIP与已审工作树保留。
