# XQ27 灰色56候选独立复审

固定源码 `4d63b07608e473c828d16b403227654c5267e5ac` 相对基线 `68547fce82bc5dbd8877fb4dac7685af703eed24`：**通过；已发现的 XQ27-IR-01 关闭，未发现其他阻断。** 结论限于下述固定源码、产物及有限覆盖；尚未推送或发布。

## 实际阻断与修复

原提交 `ab6ff22791d5abb0799e7189f77f208e9abe3d2c` 的完整有效普通打出 Room 只改 root `stack.card.face_down=true`，旧 WASM 的存读与实际响应结算仍接受；既有 Bury 保留该标记，导致普通事务进入 owner 墓地后仍对其他席位显示为暗藏者。这是实际 P2 准入／墓地可见性问题。真实空现身会在部署时恢复正面，所以旧反例不证明再次潜伏。

修复仅在现有 XQ27 root 卡条件中拒绝 `face_down`，另补一个完整 Room 单字段 Native 反例与 Sites ABI 双入口测试。未改变 Bury、同时死亡核心或引入身份队列。独立新 WASM 已确认普通 rootface 反例和现身同字段变体均在 `view` / `applyRoom` 以 XQ27 专属原因拒绝。原阻断报告和输入仍保存于上级目录 `ab6ff-blocked-review.md/json`。

## 独立检查

实际查看完整原图；原图与提交扫描文件字节一致，SHA256 `78f2d5e2422b611bbf159a33083d70c75bf2ac83103325550430a394e8a803b5`。印刷费用4、灰忠诚2、事务／阴谋、无领域，标准行动消灭所有暗藏者。新无参单一操作只收集所有当前地区的 `face_down` 卡，先冻结整波 snapshot，再复用 `remove_death_batch`；未把正面角色、地区、资产区或秘社加入死亡集合。完整定义与递归移植拒绝、帧执行状态、堆栈布局、恢复入口、原空现身与无选择边界已只读复核。

用 Node 192MB 上限运行现成新 WASM，**100/100 断言通过**：13个 Native 非法完整 Room、17个额外标准帧变体、10个真实现身变体、2个 queued/pending 变体均检查两个 ABI 入口；四席实际支付、3个完整 Native transition/四席 view、4个公开预声明参数原子拒绝亦通过。

四个正向行为补充验证：20张分布于所有地区且 owner≠controller 的暗藏者进入各 owner 墓地、恢复正面及 owner controller；公开 JC045 未因暗藏者移除触发角色死亡观察，地区／影响力不变；实际付费 JC003 响应现身在结算前改变当前面并逃离消灭；真实暗藏 XQ27 付费空现身不执行标准消灭；异 owner 的真实现身保留 owner 与 actor controller。

产物为 `2,395,545` 字节，WASM SHA256 `e253c69ed274e477e1fbba8393802314f34975196a8d1524cf8ef4bc17a5a28e`，pkg、Sites generated、final-wasm 三份实际字节均相同。所有原118张卡、牌组、世界及其余目录字段结构完全一致，仅 engine/pool 版本变化和新增 XQ27。19份非 docs 的源码／运行时／测试指纹与父证据逐项相符。独立流式重算历史53/54/55共33份 Native/WASM 的原始 SHA（含ZIP成员），共378,937,871字节，全部一致。

## 父执行证据与边界

复核修复后的日志：Native新卡10＋既有同时死亡15＋目录1；WASM216原始输入／812席位视图精确一致；UI81；双 TypeScript配置、Vite、Wrangler dry-run；Worker6（含真实本地D1重开、重复原 receipt、IR-01双入口、精确旧 engine55 Room拒绝）均通过，40历史 tuple 与5 altered identity拒绝。

独审动态执行只使用供应的固定 WASM；上述 Native、UI、构建与 Worker是父执行日志，未声称独立重跑。使用明确准备布局和真实付费 Room 命令，未运行自然浏览器多人整局、随机长局、新编译器或旧批全套回归；也不声称通用任意存档完整性证明。未写源码、切环境、建 worktree、推送、发布或改 main。后续若仅收束任务文档，应逐字节映射到本固定源码，再沿用动态结论。

原始断言、指纹与证据哈希见本目录 `finite-wasm-probes.json`、`repaired-source-fingerprint.json` 和 `review.json`；额外篡改输入及最终状态保存于上级目录。
