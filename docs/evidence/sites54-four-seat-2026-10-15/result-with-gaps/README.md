# Site54四席自然终局：有明确缺口的结果

原桌 `1609f4e9dabfb4d970646b44`，第11回合5:12、团队1获胜、版本1904。`result.json`的naturalGamePassed与terminalReopenPassed为true，overallAcceptanceComplete为false：JC095非零转换未观察，3次历史HTTP尝试缺少逐项回执。详见[结果文档](../../../SITES54_FOUR_SEAT_PC_RESULTS_2026-10-15.md)。

- `teams-summary.json`是第四段原始完成摘要；前3段失败摘要、例外与原执行脚本继续保留在早期WIP目录。
- `four-seat-reopen-summary.json`是原四个profile关闭重开及四页刷新共8份视图，0游戏POST、0浏览器错误、0转发异常；不是另开牌桌。
- `action-log-composition.json`绑定最终1822条动作日志：第三检查点的1558条前缀与本目录264条增量。合并后的字节、行数及SHA均记录；没有重写原始前缀。
- `network-attempt-04.jsonl`及[第三段追踪](../wip-checkpoint-03/network-attempt-03.jsonl)记录命令原身份及公共响应，不含请求头、Cookie、完整手牌或会话凭据。
- `jc046-sequences.json`核对实际4条合格席位链及来源离场续席；`jc060-combat-summary.json`记录12591条限定范围观测；`jc095-entries.json`记录两次真实接受触发和零敌方标记边界。
- `request-receipts-summary.json`保留1830次尝试/1827个回执、2次过期409和3个未确认HTTP尝试。没有将三次缺回执当作三个已知不同命令。
- `site-identity-reconfirmed.json`是官方只读身份复核；`public-identity-attempt-04.json`是浏览器77/2.70/143卡及入口文件核对；本任务没有部署。
- `visual-inspection.json`只存线上选择/终局截图哈希及审阅结论。原图含自己的私有手牌，留在任务目录，未复制到Git。

可从仓库根目录复算动作日志及专项审计（仅读证据，不访问牌桌）：

```bash
cat docs/evidence/sites54-four-seat-2026-10-15/wip-checkpoint-03/teams-actions.jsonl docs/evidence/sites54-four-seat-2026-10-15/result-with-gaps/teams-actions-after-checkpoint03.jsonl > /tmp/sites54-four-actions.jsonl
PYTHONDONTWRITEBYTECODE=1 python3 tools/cloud-playtest/abi77_jc046_public_sequences.py --summary docs/evidence/sites54-four-seat-2026-10-15/result-with-gaps/teams-summary.json --actions /tmp/sites54-four-actions.jsonl --output /tmp/sites54-jc046.json
PYTHONDONTWRITEBYTECODE=1 python3 tools/cloud-playtest/abi77_jc060_public_combat.py --actions /tmp/sites54-four-actions.jsonl --catalog sites/test/fixtures/jc046-native-v077.json --output /tmp/sites54-jc060.json
PYTHONDONTWRITEBYTECODE=1 python3 tools/cloud-playtest/abi77_request_receipts.py --summary docs/evidence/sites54-four-seat-2026-10-15/result-with-gaps/teams-summary.json --actions /tmp/sites54-four-actions.jsonl --trace docs/evidence/sites54-four-seat-2026-10-15/wip-checkpoint-03/network-attempt-03.jsonl --trace docs/evidence/sites54-four-seat-2026-10-15/result-with-gaps/network-attempt-04.jsonl --output /tmp/sites54-receipts.json
```

catalog参数仅供应印刷定义，任何fixture游戏状态都不参与线上对局或此审计。完整12591条战斗审计和1035次追踪审计可由上述输入复算；Git中的摘要保留完整输出SHA/字节数，避免重复大量可重建记录。`audit-tool-identities.json`记录生成脚本指纹，`manifest.json`绑定本目录材料。
