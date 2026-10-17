# WM062＋JZ48：官方v62发布、main快进及清理候选

父会话确认独立复审已通过，并转达用户对这个具体批次的发布确认后，沿官方Sites流程完成保存和部署。随后按追加授权正常快进UndergroundBattle远端main。没有新开发、工作树、测试重启、强推、GitHub Actions或删除操作；exchange／Lab未涉及。

## 官方发布结果

| 字段 | 返回值 |
| --- | --- |
| URL | https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site |
| project | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| version | **62** |
| saved version ID | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_34afa8141f088191846be0b8cacc405c` |
| deployment ID | `appgdep_6ad2db9aecb08191885336dfa84796a1` |
| status | **succeeded** |
| updated UTC | `2026-10-17T02:21:25.001895+00:00` |
| 官方来源SHA | `1f1b785df84738e6f85c1462482236b3ddabb8be` |
| 已审游戏代码 | `a9161c958dedceb95b70310d897dd5eeb8550e52` |
| 已审证据／GitHub main | `d1a170939ac680c74dc7b73e2c1da9647d0bf32e` |
| 公开范围 | 原public，access revision2 |
| 当前80 WASM SHA256 | `0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472` |

`official-receipts.json`保存官方helper的打开、推送／打包结果，官方保存、部署及版本／Site回读、D1表概览。部署原生调用直接返回`succeeded`，没有重复部署或额外状态轮询。回读确认同一个v62、来源SHA、部署ID与原URL。没有创建新Site或改变公开范围。

本轮没有浏览器访问或线上试玩，没有进入真实用户旧桌。旧v61证书错误仍只属于此前记录；不能据此声称v62的浏览器TLS成功或失败，也不能将发布成功报成已完成线上自然对局。

## 源码与运行文件核验

- `preflight-audit.json`：游戏工作区及远端固定为d1a1709且干净，66个变动非文档文件、125个Native编译输入及166个运行文件匹配已审来源／清单；四核原字节保留。
- `provider-copy-plan.json`、`provider-input-audit.json`、`provider-source-files.json`及`verify_provider.py`：1169个有限映射；其中1167个源码输入逐Git blob匹配a9161c9，另两个为已固定JS绑定／WASM。两个package只保留原正式站的fixture还原钩子，原压缩样本和还原脚本不变；没有未知源码改动。
- `publication-archive-audit.json`：部署包中166个运行文件逐SHA256和字节数匹配审查清单。另6文件只有hosting、原drizzle三文件、README及source map。
- `saved-archive-identity.json`、`archive-encoding-audit.json`：保存回执的规范TAR由同172份文件负载重构，SHA256和字节数与官方存档一致。

本地gzip包SHA256为`6a29c1bf144a170b2368ca79e971c5360af7e17707bbb46d0ae26e468f196c0c`，73,061,508字节。初次直接比较GNU TAR容器与官方存档失败，原文见`archive-storage-comparison.json`。对同172个负载按路径排序、uid/gid/mtime清零、mode0644重编码为PAX TAR后，得到官方`sha256:3afe8f43897a947d351d7add83e1138d7229b82f18c5c4f60a14673103e8ad21`，108,646,400字节。没有修改原gzip包、负载、版本头或重新保存版本。

原77／78／79的WASM、JS及gzip身份均已核对，来源和运行包继续精确四核路由。hosting、wrangler和全部迁移文件逐字节保持；官方D1前后均为原`DB`绑定及`commands`、`entry_receipts`、`journal`、`rooms`、`seats`五表。本轮核验没有读取、修改或迁移真实桌行数据；不把表概览冒称逐行或旧桌实战验收。适用行为证据仍为已独审批次中的Native／WASM／HTTP／前端／本地真实浏览器记录，沿用其完整回归和定向补跑边界，没有改称一次全绿。

## main结果

`main-preflight.json`和`main-result.json`保留祖先关系、16个已审提交及适用测试证据。原远端main为`bd36b70b32890f539fde019613f5ae6d60c2903d`，是固定d1a1709的祖先；没有main独有提交或相对已审a9161c9的非文档增量。16个提交均带`[skip ci]`，两端都无workflow文件。

使用普通`git push origin d1a170939ac680c74dc7b73e2c1da9647d0bf32e:refs/heads/main`成功快进，随后精确回读main和原审查分支均为d1a1709。没有强推或制造额外merge commit；游戏工作区仍在原审查分支且干净，官方源码工作区也干净。GitHub Actions发布前／main推送后均为8次原有Copilot记录。

## 清理只列候选

见[CLEANUP_CANDIDATES.md](CLEANUP_CANDIDATES.md)、`cleanup-branches.json`、`cleanup-artifacts.json`和`cleanup-p2-source-identity.json`。本环境候选为9个目录、2,482,024,448字节（约2.48GB），以及41条已合并、未被当前工作树占用的分支；无额外worktree候选。P2重复源码副本的1143个源文件逐Git blob匹配已合并53b2114，所含生成字节与保留的78一致。原94e5及ef68见证、当前四核、当前发布包、上一v61回退构建和当前源工作区均排除。

当前可见进程未发现候选目录占用，但两个进程FD目录不可读，其他设备或远端会话活动也未作保证。实际删除前必须重新核查目标设备、活动和恢复保留要求。本轮没有删除或设置定时删除，Mac Air未直接访问或测量；候选字节数不是已释放空间。后续定期清理策略由父会话统一安排。

## 归档范围

可归档本目录顶层的README、候选说明、JSON审计和Python核验脚本；`evidence-file-hashes.json`只固定这些小型材料。不要把`wm062-reviewed-80.tar.gz`、`prepublish-v61-dist/`或`provider-retained-storage/`作为新源码提交。运行包、回退构建及样本备份继续留在Git外。来源凭据只经会话内存和隐藏stdin传递，使用后已从会话存储清除，没有凭据、数据库、缓存或运行二进制进入Git。
