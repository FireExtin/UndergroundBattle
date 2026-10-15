# JC095真实非零转换：最终证据

新自有专项桌 `cadc0c7dc3a8f7da299f03db`，第8回合席1在纽约将 `[1,0]` 转为 `[0,1]`；两席版本372、来源i183一致。通过UI暂停到373，原两席profile重开后仍暂停且保留转换，0游戏POST。范围仅这一例正数转换，不是完整对局或全部JC095边界。详见[结果说明](../../../../SITES54_JC095_NONZERO_PUBLIC_PROBE_2026-10-15.md)。

- `target-summary.json`、`duel-actions.jsonl`为原主运行结果与65条手动操作记录；普通UI自动让过也经过正常命令管线，完整375次命令HTTP尝试在`network-attempt-01.jsonl`，不能用手动记录数代替请求数。
- `audit.json`按原ID/演员/期望版本/意图核对：377次总尝试、372个回执（含2次创建/加入），5次ERR_ABORTED均有同ID后续200。368个命令接受、2个已知过期拒绝，逻辑结果无未决。中止HTTP仍为中止，不改记成功。
- `paused-reopen-summary.json`是同一专项桌的原两席正常恢复，非新桌；`paused-reopen-runner.py`冻结执行代码。
- `site-identity-before.json`、`site-identity-after.json`、`browser-identity.json`绑定当前公开v54、provider源码cec2bd64…及77/2.70/143卡/入口文件。对应游戏候选4918cfb…的820份绑定见既有发布材料，本轮没有再部署。
- `runner-attempt-01.py`冻结实际主脚本；`audit-tool-identities.json`和`manifest.json`绑定材料及复算程序。
- `visual-review.json`记录真实线上触发/转换/暂停重开截图哈希。原图含自己的手牌，留在任务工作目录，不进Git。

原四人桌 `1609f4e9dabfb4d970646b44` 保持终局，没有打开其profile或重发命令。[历史只读持久记录](../../receipt-reconciliation/proof.json)确认原命令811已提交，但早期3次HTTP尝试仍无法逐项识别，原1830/1827计数不变；与此处5次已追踪中止分开。

从仓库根目录复算（仅本地读证据）：

```bash
JC095_EVIDENCE=docs/evidence/sites54-gap-closure-2026-10-15/jc095-probe/result
PYTHONDONTWRITEBYTECODE=1 python3 tools/cloud-playtest/audit_jc095_nonzero.py \
  --summary "$JC095_EVIDENCE/target-summary.json" \
  --actions "$JC095_EVIDENCE/duel-actions.jsonl" \
  --trace "$JC095_EVIDENCE/network-attempt-01.jsonl" \
  --reopen "$JC095_EVIDENCE/paused-reopen-summary.json" \
  --history docs/evidence/sites54-gap-closure-2026-10-15/receipt-reconciliation/proof.json \
  --output /tmp/sites54-jc095-audit.json
```

没有数据库页缓存、完整内核状态、Cookie、请求头、座位token、浏览器profile或完整私有手牌。CLI日志仅保留进度及结果，原先WIP前缀保持不变。
