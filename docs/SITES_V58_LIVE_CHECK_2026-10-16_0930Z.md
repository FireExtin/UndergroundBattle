# v58 正式站真实浏览器复查：09:30 UTC 证书阻断

输入：父会话要求按用户既有定期真实试玩要求，在无活动编辑的安全边界检查最新已审已发布 v58。先核对版本身份，再以正常浏览器访问正式站；仅在可达时新建隔离测试桌，若证书或访问阻断则记录并停止，不绕过安全警告或使用绕过令牌。

本轮开始时工作区干净，HEAD 为 `6277c4b0015afed6ded96cf8fb7800b446767dea`，Web tree 为 `84d9e77f3f592e015dd0bfec807d82f4917e36f4`，没有活动编辑、构建或本地游戏服务。

## 通过：正式版本身份

本轮直接通过官方 Sites 只读工具核对，已从此前父会话转交的成功摘要补充为本次 native 状态读取：

- 正式地址：https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site
- 最新保存版本 v58：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_de61c2cbd32c819187d3476b74d9e8f9`。
- Provider SHA：`eec9ffb81b66b149bdcc4f045c1be834dae21748`。
- Deployment：`appgdep_6ad1e61a0de881918e24c273e78f3eee`，`status=succeeded`，官方成功时间 `2026-10-16T08:53:55.286582+00:00`。
- Site 为 `active`，公开范围 `public`，access revision 2；没有改动设置。

详见 [official-identity.json](evidence/sites-v58-live-check-2026-10-16-0930z/official-identity.json)。保存版本、provider SHA 和 deployment 均与已归档发布身份匹配。

## 阻塞：正常浏览器证书检查

使用本执行环境已安装的真实 Chromium `151.0.7922.173` 和 Playwright，新建空白浏览器 context；`ignore_https_errors=False`，未注入令牌、凭据或旧会话，没有使用证书忽略参数。只导航正式根地址一次，没有携带邀请码或访问用户桌。

- 开始导航：`2026-10-16T09:30:56.687955+00:00`。
- 错误时间：`2026-10-16T09:30:56.922562+00:00`。
- 原始错误：`Page.goto: net::ERR_CERT_AUTHORITY_INVALID`。
- 浏览器显示正常的 “Your connection is not private” / `NET::ERR_CERT_AUTHORITY_INVALID` 安全警告，未点击 Advanced、继续访问或其他绕过入口。
- 游戏 UI 未进入，前端资产身份检查未执行，POST 0，新建/加入测试桌 0，真实用户桌访问 0，完整对局 0。
- 保存失败导航画面后关闭 context 和浏览器；结束时间 `2026-10-16T09:30:57.195688+00:00`。没有再次尝试导航或换用其他访问路径。

这说明该执行环境本轮访问被证书信任检查阻断，不据此推断所有用户浏览器均不可达，也不归因于某个游戏功能。

## 未执行与证据

红、黑、蓝、灰牌组、双端新建桌、选牌、重复点击、刷新恢复、连续让过阶段/回合边界、强制选择以及完整胜负对局全部未执行。既有组件或 localhost 测试没有计作线上试玩通过，官方部署成功也没有计作浏览器试玩成功。

原始浏览器结果见 [normal-browser.json](evidence/sites-v58-live-check-2026-10-16-0930z/normal-browser.json)，警告画面见 [normal-browser.png](evidence/sites-v58-live-check-2026-10-16-0930z/normal-browser.png)。同目录保留本次准确探测脚本、身份读取和文件哈希，不含凭据、浏览器存储或用户数据。

本轮没有开发修复、重建、保存新版本、部署、更改公开范围、操作数据库或真实用户桌，没有合并 main 或触发 Actions。只归档本次正常访问阻断；后续真实试玩仍须从正常浏览器可达开始。
