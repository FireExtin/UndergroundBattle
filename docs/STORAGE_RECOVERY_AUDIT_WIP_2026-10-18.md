# v69 后空间阻塞：只读盘点与待确认清理方案

状态：**只读核验已完成；清理尚未执行，待父会话确认永久移除范围。** 独立分支 `codex/storage-recovery-audit-wip-20261018` 基于已推送发布回执 `d0a52c8d8c7b9589952b7dadbb0ab7283dd6a8e9`。本任务不修改游戏产品，不推进 main，不触发 Actions，不再次部署。

父会话要求：“仅可恢复的安全清理才允许实施；没有正常恢复路径或属于永久删除须提出准确路径、大小、可重建依据和影响交父确认，不直接rm。”本环境未找到可用的回收站工具或已验证的可撤销删除路径，因此没有把“可重新下载/从 Git 重建”扩展解释成永久删除授权。

## 实测空间、占用与进程

普通磁盘 `/workspace` 是 overlay；共同 target/TMPDIR 与其同盘。`/tmp` 是 tmpfs，未采用。启动守卫保持 **1,073,741,824 字节**。

| 测量 | 可用字节 | 距守卫差额 |
| --- | ---: | ---: |
| 15:35:00 UTC，盘点前 | 994,414,592 | 79,327,232 |
| 15:41:17 UTC，只读核验后 | 994,344,960 | 79,396,864 |

**实际清理释放 0 字节、删除 0 个文件。** 差额的小幅增加来自本任务写入审计脚本和必要证据，不能称作清理后实测。后续提交也会有少量写入，执行任何已批准方案前必须重测；下列释放量是按文件分配块估算，不能替代删除后的 `statvfs` 实测。

主要目录的 `du -x -B1` 已分配字节如下。不同次 du 会各自去重硬链接，表中数值不可跨行直接相加，也不能保证 overlay 删除后等量回收。

| 路径 | 已分配字节 | 处理结论 |
| --- | ---: | --- |
| `/workspace/undergroundbattle-build` | 7,114,399,744 | 不是整体清理对象 |
| `/workspace/undergroundbattle-build/tmp` | 1,798,029,312 | 含测试数据库/回执，保留，不按 tmp 名称删除 |
| `/workspace/undergroundbattle-build/target` | 1,464,537,088 | 当前复用 target，含未合 JC041 相关编译结果，保留 |
| `/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114` | 1,092,812,800 | 已合并源副本已逐文件核实；删除仍待确认 |
| `/workspace/undergroundbattle-build/source-stage-bq035-20261016-11927a3` | 1,092,337,664 | 含未发布修复前见证，排除整目录清理 |
| `/workspace/undergroundbattle-build/source-stage-bq035-20261016` | 1,092,337,664 | 同上，保留 |
| `/workspace/undergroundbattle-build/npm-cache` | 162,914,304 | 仅选出下述可验证缓存包，不授权整目录删除 |
| `/workspace/UndergroundBattle` | 3,614,646,272 | 当前源码、Git、证据、依赖与产物保留 |

其余根目录的完整精确数值见 `directory-sizes.json`。例如 BQ035/P2、JZ46 大型 `.cases` 展开物位于审查证据目录，仍保留，没有把它们当缓存。

进程核验未见活动中的 cargo/rustc/workerd/Chromium 游戏验证进程，154 个历史子进程为僵尸。可读的 cwd/exe/fd/maps 未引用选定三个候选根路径；dockerd/containerd 的 FD 目录及若干进程 maps 无权限读取，因此**未证明全局绝无占用**，不绕过权限读取。精确 PID 和权限边界见 `process-and-recovery.json`。

`gio`、`trash-put` 均不存在；工作区可见 Trash 路径不存在，`/root/.local/share/Trash` 不可读。没有新建回收机制或搬入其他环境。即使同盘移动至 Trash，也不会释放本盘数据块。

## 最小待确认范围：npm 下载缓存内容

两个锁文件与固定 `main@224a619459d64eaead06810b679ff57b1de94fd6` 完全相同。以下 7 个包逐个匹配锁文件 integrity，本地 SHA512 验证通过，并从锁文件的精确 registry URL 重新流式读取，字节数和 SHA512 全部一致；远端数据没有落盘或执行。安装目录、源码、锁文件、当前部署和精确内核均不在候选范围。

完整绝对路径、inode、硬链接数、SHA256/SHA512、URL 及远端读取结果见 [精确候选清单](evidence/storage-recovery-audit-2026-10-18/npm-cache-recovery-candidates.json) 的 `selected` 数组，按表中顺序一一对应。路径全部位于 `/workspace/undergroundbattle-build/npm-cache/_cacache/content-v2/sha512/`，每项 links=1。

| 顺序 | 缓存包 | 版本 | 文件字节 | 分配字节 |
| --- | --- | --- | ---: | ---: |
| 1 | `@cloudflare/workerd-linux-64` | `1.20261001.1` | 39,830,481 | 39,833,600 |
| 2 | `@rolldown/binding-linux-x64-gnu` | `1.0.0-rc.12` | 9,373,010 | 9,375,744 |
| 3 | `@rolldown/binding-linux-x64-musl` | `1.0.0-rc.12` | 9,324,483 | 9,326,592 |
| 4 | `@img/sharp-libvips-linuxmusl-x64` | `1.3.3` | 8,235,549 | 8,237,056 |
| 5 | `@img/sharp-libvips-linux-x64` | `1.3.3` | 8,176,720 | 8,179,712 |
| 6 | `miniflare` | `5.20261001.0-alpha` | 5,312,220 | 5,312,512 |
| 7 | `@esbuild/linux-x64`（tsx 依赖） | `0.28.2` | 4,741,809 | 4,743,168 |

**按已核候选的文件数，至少需要 6 项**：任意 5 项分配量不足。按清单顺序取前 6 项的方案为 **80,265,216 分配字节**，估算可刚越过守卫，余量不足 1 MB，写入证据/运行时后可能再次阻挡。这不是全部子集的最小字节优化。若父会话确认前 7 项，估算释放 **85,008,384 字节**，仅多约 5 MB 守卫余量；这仍不是持续 Rust 构建的容量预算。

影响：本地离线下载缓存减少；现有已安装依赖和游戏产物保留。再次安装可能重新下载。恢复可通过精确 URL 的正常 npm 缓存获取并校验锁文件 integrity，但依赖 registry 可访问和足够磁盘空间；当前已经实读验证，不保证未来服务永远可用。缺失内容的索引记录可能成为 cache miss，未修改缓存索引。**永久移除这些本地文件仍须父确认；未运行 unlink、rm、npm cache clean/verify 或任何具有删除副作用的操作。**

## 更大但仍待确认的备选：单个已合并源暂存副本

`/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114` 的候选源 `53b2114a220ca001ca8c40ff3b2a6f2b6276ef45` 已证明是固定 main224 的祖先。现场复核全部 1,145 个映射文件：1,143 个与 Git 保留 blob 逐字节一致；另 2 个生成 JS/WASM 与目录外保留的 ABI78 规范依赖相同，WASM 从原 gzip 无损读取。除此只有 `.gitignore` 和 `source-provenance.json` 两个自有小文件，其精确字节和 mode 已单独保留。

映射清单在 main224 中的 blob 为 `7ee921de3cb7158b40b8fb3731569292a819a683`，位置 `docs/evidence/bq035-p2-2026-10-16/staged-source-files.json`。重建需要按该映射从保留 Git commit 读取源文件、从保留 ABI78 读取两个生成文件，再恢复上述两个小文件，最后重验全部哈希。这证明可重建性，**不等同回收站撤销**，没有删除授权。

如另行确认删除这一个明确目录，预计容量约 1.09 GB，较适合后续工作；不是删除其他两个包含修复前见证的 source-stage，也不是删除历史 ABI78 依赖、当前工作树或审查原件。目录本身不是 Git worktree，未发现数据库、凭据或唯一源文件；全局占用仍受上述权限限制。此次没有执行这个备选。

## 保留边界、验证与交接

- JC041 已审源 `4ffa7155537187d057b52612d2da076da6a2a47d` 不是 main224 的祖先；其源码、输入、P2 前后见证、审查证据、当前复用 target 和发布材料全部保留。
- 当前 v69 provider 源、当前/旧部署归档和各次审查证据、数据库、保护锁及 ABI77–86 十个兼容核保留。没有把旧部署、历史证据或原始失败记录按年龄判无用。
- 原发布回执 `d0a52c8` 已推送；本小时先核对远端一致，不制造重复发布回执或空提交。
- 本任务没有构建、测试产品或启动浏览器，没有降低空间守卫。只读验证包括目录分配统计、进程引用、Git 祖先/blob、旧批次 1,145 个映射文件、缓存完整性与远端字节读取；这些不构成新增游戏回归通过。
- 第一轮旧 ABI78 定位按未压缩文件名查找而停止；纠正为读取既有 `.wasm.gz` 后恢复字节相同。保留第一轮状态记录，只续跑该阶段，没有重复网络下载，也没有据错误查找删除材料。
- Personal Context 返回不可用，没有从缺失上下文推断额外清理授权。本任务停在永久移除需父确认的边界，来自本次父指令，非技能额外审批要求。

父确认应明确选择精确候选清单中的范围（前 6/7 个缓存文件，或另行确认上述单个已合源副本）。收到确认后仍需重核空间、占用和哈希，执行限定范围，测量实际回收字节并运行原守卫；不得把本页估算报告成已解除阻塞。

证据目录：[storage-recovery-audit-2026-10-18](evidence/storage-recovery-audit-2026-10-18/README.md)。原始只读材料另保留于 `/workspace/storage-recovery-audit-20261018/`。
