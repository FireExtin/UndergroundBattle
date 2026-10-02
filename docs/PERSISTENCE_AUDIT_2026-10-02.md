# 已结束对局的只读持久化审计

核对对象是云端真实 UI 对局的 `/tmp/hegemony-final-validation.sqlite3`，不创建测试管理状态。
2026-10-02 使用 `hegemony-audit` 以只读 SQLite 事务从固定初始状态与 journal 重放全部操作，然后对完整状态作规范化比较。

| 模式 | 房间 ID | 版本/日志条数 | 保存状态与重放 | 最终状态摘要 SHA-256 |
| --- | --- | --- | --- | --- |
| 双人 | `9d2bd03ddd1ae85220d2e04e` | 317 / 317 | 完全一致 | `650dad3407c57f592fa0bafc9c12bf59499ed41529c4deadba79d547c29943c6` |
| 四人 2V2 | `59ed382028ce7ebe934292b9` | 2921 / 2921 | 完全一致 | `e74459dc8392b79079e8adba5521311bbb9cfbb43244f52563ed7c8d67c1df1a` |

两局均为 `finished`，固定版本为 `hegemony-pdf-v1` / `limited-v1` / `rust-v0.1.0`。版本数量包括入席等日志，因此与只统计对局循环动作或 commands 的试玩计数不同。

复核命令：

```bash
target/debug/hegemony-audit /tmp/hegemony-final-validation.sqlite3 \
  9d2bd03ddd1ae85220d2e04e 59ed382028ce7ebe934292b9
```

原始只读审计输出保留在 `/tmp/hegemony-persistence-audit-2026-10-02/final-room-audits.jsonl`，不含令牌或牌面。游戏中待选及断线/刷新/进程重启恢复另见 [云端验收记录](CLOUD_PLAYTEST_2026-10-02.md)。

这证明上述对局的确定性持久化和回放一致；它不证明原 PDF 的每条规则已实现，也不替代互联网发布验收。
