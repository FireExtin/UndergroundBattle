# A/B隐秘对抗集成检查点

集成树固定在ce022d5c，仍原生22/50。A在独立普通克隆编写共享核和WM021/WM022/WM032/WM061/BQ082；B的BQ027/BQ031/BQ032/BQ036/BQ038保留归属，未在本树复制代码。此文是接线准备清单，不是源码审查结论或测试通过证据。

A报告拟固定类型`mystic_contest::{MysticContest, ContestOutcome, MysticShown}`，入口`Game::mystic_contest_start(frame: ResolutionFrame, actor_bonus: u32) -> RuleResult<()>`。提交和奖励继续使用`ChoiceResolution::Frame`。结果保留contest_id/turn/frame_id/actor/source/ability_key/participants/shown/totals/winner_team/first_winners，展示项保存原卡和当次加值来源见证。现有费用、响应、续体、恢复与四席视图入口继续复用。

B单独拥有`mystic_hooks.rs`：开始观察`mystic_start_observers(&mut self, contest_id: &str)`、胜利观察`mystic_win_observers(&mut self, outcome: &ContestOutcome)`、逐席展示后置`mystic_shown_after(&mut self, outcome: &ContestOutcome, index: usize) -> RuleResult<()>`及展示加值函数。BQ038加值拟通过`mystic_add_bonus(frame, amount)`绑定本卡真实来源与算值，未识别收据默认拒绝。最终签名与固定提交仍待A/B交付。

实际集成时需要核实：

- 开始观察的普通声明、可响应帧与唯一继续效果的队列顺序；不得在开始响应尚未结束时移入牌库顶牌，也不得重复移入。
- 全部玩家提交前，其他席位含队友不能读到提交卡UID、名称、印刷费用或私有预览；重复查询不消费随机数、序列或续体。全部提交后同时公开，个人展示牌各自洗回拥有者牌库。
- 四席的团队关系沿已有seat/2与正对面seat^2；个人第一次获胜不能因观察者重新进场而重置。平局、消灭席位与可放弃奖励分别核原规则。
- BQ027展示后置、BQ031获胜观察、BQ038开始触发必须引用同一不可变结果/对抗身份，不能在恢复后重算一次、重新收费用或领取重复奖励；各待选节点分别检查Game与Room恢复。
- 旧目标合法失效继续沿原UID整效取消；存档结构损坏、未识别收据或程序移植须拒绝。不得把当前目标不存在误当成损坏存档。
- 目录新增条目顺序、原图SHA和原213记录保持；公共接线与包内卡程序分开审查，只保留最新内核。整批50张齐备前不开放未完整准入。

A已明确未裁定“参与者手牌、牌库同时为空”时无法展示的数值/退出语义。不要把无牌等同一张0费牌。若核心在结算阶段返回待裁定错误，会由既有克隆事务回滚本次操作；这仍属于未开放边界，不能声称该极端局面已经完整结算或用静默pass掩盖。XQ20计划跨阈值与重复达成问题已由父端交其他线程只读核实，不与A/B重复开发。

本次读取A未提交文件仅用于定位接线位置；由于仍在编辑，没有对其运行测试或形成固定SHA审查结论。交付后按具体代码差异进行必要定向验证，不重复已完成整套。
