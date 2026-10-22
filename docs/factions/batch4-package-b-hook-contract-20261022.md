# B 钩子对共享协议的要求

供 A 与集成线程使用。实现位于 `rust-game/src/batch4_vampire_contest.rs`，不替代 A 的协议。

|入口|B 行为与所需状态|
|---|---|
|`mystic_start_observers(id)`|当前 `mystic_contest` 为同 ID 的 `BeforeSelection`。从当前合法 BQ038 附属建立普通可选声明，使用现有响应堆叠，并登记本次开始观察的排序组。|
|`mystic_win_observers(outcome)`|向 effects 追加 BQ031 的个人胜利观察；不登记排序。共享核心必须仍在 A/B 全部追加后统一登记一次。|
|`mystic_extra_shown_bonus(shown)`|BQ027 固定加2，其他0；不读取或改变地区势力。|
|`mystic_shown_after(outcome,index)`|当前为同次 `After`，`next_shown == index+1`。BQ027 建立自己的 `post_frame`，先选本方吸血鬼，再使用既有私有鲜血选择。|
|`mystic_validate_bonus(bonus)`|只承认 BQ038 完整已结算帧；金额须等于去重宿主见证数。|

BQ038 的 `StartObservation` 在未结算时有 `contest_id/turn`，`witnesses` 和
`resolved_id` 均为 None。结算时捕获见证并分配新的结算事件身份，再提交给原
`mystic_add_bonus`。Some(empty) 是已结算的 +0，不能等同未结算或放弃。

BQ027 的专属后置帧包含 `BQ027ShownFeed { contest_id,index }`、原展示来源和唯一
帧身份。目标选择前 `targets=[]`，选择后保留一个原目标绑定，并同步 `post_frame`。
饮血完成后清除该 `post_frame`，仍通过核心已有 `MysticContinue` 前进，不重复展示、
洗牌、付费或移动游标。饮血的正常进场观察仍走原事件队列。嵌套、终局清理或暂停
期间，核心必须保存/恢复与原对抗配对的帧、待选及续体；不能只认证 WM061 后置。

BQ032/BQ036 的起始帧由现有费用和响应路径产生，固定 `MysticContest` 加值分别0/2。
BQ032 使用原 `Discarded` 和 `Assets` 收据；BQ036 使用 `Assets` 与 B 专属
`BQ036Attachment` 收据。B 校验器完整认证费用、来源、原宿主及固定程序。
三个 A 路由入口只进行有限委托：`admitted_start`、`validate_pack_a_declaration`、
`validate_pack_a_frame`；没有修改 A UID 谓词或任何协议步骤。

另一个公共接线是 `batch4::related` 识别 BQ038 开始事件，确保把该事件塞入外国
程序时也会被拒绝。B 专属费用收据本身同样触发 B 帧校验，不能藏在普通 JC125
派遣帧的 `already_paid` 内。这一费用移植在修前确实被 Game 恢复接受，修后
Game/Room 两入口都拒绝。

## 当前真实阻塞

`b_contest_protocol_response_reveal_must_dispatch_before_outer_commit` 单独执行失败：
BQ032 原行动 → 接受 BQ038 开始触发 → 真实轮转优先权 → 座位2付费现身 WM022。
现身结束时：

- 堆叠顶部仍是 `batch4-bq038-contest-bonus`；
- 效果队列为 `MysticContinue`，然后 `pack-a-wm022-reveal`；
- 新生成的强制声明未进入堆叠，受到已有开始续体阻挡。

该回归当前标 ignored，但提供了实际 `--ignored` 失败日志；修复后应移除 ignored
并在同一真实链继续覆盖嵌套。不是靠种入声明来宣称合法响应已通过。

旧 `tools/cards/package-b-interface-probe.rs` 已在本次整合 B 克隆重新执行5项。
其中 BeforeSelection/After 探针明确使用带真实原结果的人工边界夹具；它们确认普通
响应可等待、空手空库仍显式拒绝，并继续复现声明延后、嵌套拒绝和后置终局清理
失败。5项“通过”意味着成功复现断言，绝不代表三处缺陷已经修复。
