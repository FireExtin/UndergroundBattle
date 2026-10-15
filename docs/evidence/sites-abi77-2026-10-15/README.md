# Site54发布与PC线上验收证据

游戏候选：`4918cfb6372fb9ec900f037e8ed518385d6183e6`。
provider源码：`cec2bd64b847d47ac971a459dc8ef2d61aec9e89`。
正式版本54，deployment `appgdep_6ad09dcf575c819182ed0f3340eccfb1`，原生结果succeeded。

## 发布身份

- `publication-result.json` / `saved-version-recheck.json`：官方helper结果、原生save/deploy及后续只读查询；save/deploy各一次。
- `source-provider-binding.json`：从两个固定Git提交读取并比较820份文件；两条大fixture由gzip恢复后逐字节相同；package JSON只增加列明的恢复hook。
- `prepublication-artifacts.json`：三份WASM bytes/SHA、Worker SHA。
- `client-files-158.json`：全部158份实际前端构建产物的路径、长度、SHA。
- `archive-verification.json`：官方归档全部166文件指纹；Worker、WASM及158前端与已测输入一致，hosting及迁移不变。
- `archive-storage-binding.json`：本地tar.gz、解压tar与原生content_hash各自真实值。平台content_hash未说明等同tar整体SHA，没有冒称这些整体hash相等。

## 真实公开PC浏览器

`online-identity.json`直接复算了全部158份公开前端响应的SHA，health/catalog为77 / 2.70 / 143卡。
`online-pc-checks.json`保留选择、取消、换选、刷新草稿、暂停/恢复、同伴刷新和同步双击锁检查。它包含同一房间续跑的重复检查条目，不据此累加独立测试通过数。

`online-complete-summary.json`为完整最终摘要：同一自有桌 `fd08636bd17bf0e956e194d1`，第6回合，5:8，team1胜，版本344。
341个真实UI POST包含创建/加入2个及339个行动命令；337个行动200，2个明确的window_expired/409。
0页面脚本异常、0其他浏览器错误；两条对应HTTP409控制台记录明确归类并保留。

- `duel-actions.jsonl`：339条真实UI行动记录，只保存各自公共投影、实际动作及命令ID，没有私有手牌列表或凭据。
- `full-game-summary.json`：最终成功进程的原始回执与累计UI请求预算，passed及completeGame为true；该进程从前71个请求继续至341。
- `online-run.log`：原始CLI中止、进度与最终成功输出。
- `online-transport.json`：最终进程的传输结果与明确分类的到期409控制台记录。
- `interrupted-attempts/`：四段原始中止摘要保持passed=false。原因分别为暂停页停止轮询、旧目的地按钮定位、未等待恢复视图就绪，以及第一次真实响应窗口到期。终局来自第五段，在原桌继续，没有重置牌桌或中局注入。
- `online_pc_abi77.py` / `summarize_online.py`：本轮固定路径验收及汇总快照。采用现有tools/cloud-playtest框架，所有行动经真实UI；没有行动API设置或owner bypass。
- `final-screenshot.json`：实际线上终局截图的本地指纹；图片和持久浏览器profiles保留本地，不提交。

## 发布前受影响的定向检查

`fixture-storage-verification.json`为无损恢复器3项检查。
`staged-jc046-worker.log`为38/38，`staged-jc046-web.log`为6/6。
它们独立于历史Native1169/Web1145/Worker179及P2定向45项，未跨轮相加，也未声称本次重跑当前全部180项Worker。

范围为PC1180×760的一局自然1V1及限定恢复交互，不外推为线上四人、手机或全卡池机制验收。ABI74历史hash问题与unsupported/410边界未改写。完整解释见[发布说明](../../SITES_ABI77_PUBLICATION_2026-10-15.md)。
