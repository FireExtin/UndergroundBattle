# 清理候选（当前不删除）

已发布v62，远端main固定为 `d1a170939ac680c74dc7b73e2c1da9647d0bf32e`。以下只适用于当前选定云端环境；未访问或测量Mac Air。未删除代码、文件、分支或worktree，也未设置定时删除。

当前可见进程未发现候选目录的cwd／打开文件占用；有2个进程的FD目录不可读，实际删除前还需在目标设备重新核对占用、远端tip和恢复保留策略。原根工作区及当前活动分支保留。

## 文件／制品候选：9条，合计2,482,024,448字节（2.48GB，尚未释放）

所有路径位于 `/workspace/undergroundbattle-build/`。

| 相对路径 | 已分配字节 | 依据 |
| --- | ---: | --- |
| `source-stage-bq035-p2-20261016-53b2114` | 1,092,812,800 | 已合并源码副本；1143个源文件逐Git blob一致，生成78字节已由冻结依赖保留 |
| `target/debug` | 1,214,009,344 | 可重建Rust调试缓存，冻结内核在目录外 |
| `npm-cache` | 162,914,304 | 可重新下载的npm缓存 |
| `tmp/wm062-real-http-CgmvGF` | 2,174,976 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |
| `tmp/wm062-real-http-YpLDZw` | 2,793,472 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |
| `tmp/wm062-real-http-i4LRPn` | 2,424,832 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |
| `tmp/wm062-real-http-lFWkKa` | 2,686,976 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |
| `tmp/wm062-real-http-wsKq1L` | 2,203,648 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |
| `tmp/wm062-real-http-xmImrQ` | 4,096 | 已结束的本地合成WM062浏览器测试持久化目录；fixture／报告另存 |

## 必须保留的恢复与当前材料

- `/workspace/undergroundbattle-build/frozen-kernels`：Exact77/78/79 runtime dependencies; retain.
- `/workspace/UndergroundBattle/rust-game-wasm/pkg`：Current published80 source/runtime; retain.
- `/workspace/wm062-publication-20261017`：Current publication archive, receipts and previousv61 rollback build; retain.
- `/workspace/wm062-evidence-20261016`：Current evidence and unique unpublished ef68 before-fix witness; retain.
- `/workspace/undergroundbattle-build/source-stage-bq035-20261016-11927a3`：Contains unpublished94e5 before-fix witness; not an unconditional whole-directory candidate.
- `/workspace/undergroundbattle-build/source-stage-bq035-20261016`：Contains unpublished94e5 before-fix witness; retain pending explicit archive review.
- `/workspace/hidden-world-sites-abi77-20261015`：Current official Site source checkout; retain.

## 已合并、未被当前工作树占用的分支候选：41条

以下每个精确tip均已证明是固定main的祖先。当前活动 `codex/stage-pass-progress-20261016`、main以及未证明合并的分支均排除；其他设备／远端会话活动未作保证，删除前应再核查。完整证据见 `cleanup-branches.json`。分支内容仍由main保留，空间候选主要来自上面的构建缓存和重复源码目录。

| 分支 | 当前精确tip |
| --- | --- |
| `codex/cloud-handoff-3ee9f70a-20261006` | `3ee9f70a707714d776cf5a19e106963a56bfef05` |
| `codex/deck-collection-page-review-20261016` | `ac831dbb845c41c5f088599d08779082a7b72d88` |
| `codex/engine62-ui-combined-review-20261011-591b48f` | `591b48f0c05e64aa59d3520361a8d53c3fa09d33` |
| `codex/engine63-bq030-ui-combined-review-20261011-7809971` | `780997119458a9140056f2f51d77c6347819e4ca` |
| `codex/experimental-current-only-review-20261007` | `51e0efd5ea2d4be01199b1ed2c7c75e4563f8810` |
| `codex/four-faction-current-choice-review-20261012` | `f2989ba51b33d084e16cc52df2031e4b7d1fcb6a` |
| `codex/hegemony-msjc09-society-20261003` | `c25f20e24444b6f4db023710231cb98cb5862fa2` |
| `codex/hegemony-playable` | `e6866cfab2c4475ae648b80f83ccc536b274b072` |
| `codex/hegemony-seat-return-fix-20261002` | `fd2f1fdd76b95b7f52c99ebb652ffac38f85512d` |
| `codex/hegemony-v026-review-fix-20261002` | `624ca9fd18bd636e08b212376291f6c9c73fe4c2` |
| `codex/hegemony-v026-wip-fast-handoff-20261002` | `525469e3a49e6b3715ef0f3ece19b17599e1a2a4` |
| `codex/hegemony-v027-grave-play-20261002` | `c2704d3e2c660d87e41b8987871e6f07b78e7e24` |
| `codex/implement-formal-matchstate-and-session-reset` | `fc907846e18174fa20c6b500e21ac55aa01a9936` |
| `codex/jc037-blue-hidden-investigation-review-20261013` | `3d642f0c1e6e5ac021cd40560086df35638bfe4c` |
| `codex/jc046-red-mandatory-sacrifice-review-20261015` | `a56827586d58666812bf48f6a3e0a22aecdf9895` |
| `codex/jc052-red-original-region-review-20261013` | `0939eab8f0ac971fc375183315313f4459a4e489` |
| `codex/jc060-gray-occult-adviser-review-20261015` | `e53e8b017f0ec974c6cf3d8f446b6541771cf4a5` |
| `codex/jc061-gray-entry-review-20261013` | `af77581af15a20949772f981f1ee29b62712b212` |
| `codex/jc087-black-entry-review-20261013` | `3860acd80b6ffe38bfe45bddcdc1ade50e8712f0` |
| `codex/jc094-black-sacrifice-review-20261013` | `3bbd8b5e6d4deaf10a348d68e959abdd8c32dc75` |
| `codex/jc095-black-influence-conversion-review-20261015` | `dbdee5d6014c260f9833c162bf85aff4b491b690` |
| `codex/jz35-red-host-return-review-20261013` | `9890163a093621f814f9768712e2663de2fd5239` |
| `codex/jz48-combined-review-20261007` | `16efe7b170ec1cd099ea31bf7ebc62ad023a240c` |
| `codex/jz49-combined-review-20261007` | `2305aa17c53e73dd3683ae9fe1e654531777f16e` |
| `codex/lantern-reviewed-integration-cloud` | `fce0a651701251a8d2cfc7fb56fb3c1222b952c4` |
| `codex/main4918-validation-blocked-review-20261015` | `c9fbaa286144ee4aed620823f9f59d032b64edfe` |
| `codex/object-actions-frontend` | `39d1ead5430581228ef54c0ef7337db4eabeb5ed` |
| `codex/reconstruct` | `da7f2851c6833a5f9956d34bc7c63672b5ef8cbb` |
| `codex/recovery-review-20261015` | `799717557a3536ebae75c472e1e5c4a58ac12fc9` |
| `codex/society-ui-frontend` | `4511cb726d7315cd19e8e7006d5397b4df1041ec` |
| `codex/ui-engine60-review-14b7b5e-20261010` | `14b7b5e06a0b1a3649f2a3f947ad515236ebdfd9` |
| `codex/ui-finished-summary-review-20261011` | `8dd1ebfa3700ca1793b12a2c8fc6c679c752e053` |
| `codex/ui-interaction-review-20261011` | `c285239be866523044734f7b0bf5198664e6396b` |
| `codex/ui-selection-review-20261015` | `9f3556d0983d5887b20ba61f3ae571bcd00defe2` |
| `codex/ui-storage-engine59-review-fed806b-20261010` | `fed806ba0ad29874f192965b93616c5433ce7c7e` |
| `codex/web-timeout-diagnosis-review-20261015` | `6268aea7cb99a31943da1f585b9ab06b215177f6` |
| `codex/wm023-blue-hand-entry-review-20261013` | `c51231c0e0105a8fdba8ce02bada8622c1e37602` |
| `codex/wm038-red-cultist-blood-review-20261013` | `d3035451f3a9375a53f2fd08fbbafd35858e3267` |
| `codex/wm056-black-influence-review-20261013` | `a967e134a5473e2249142ddf4791691aba521051` |
| `codex/xq24-riot-police-review-20261011` | `764d49462a41ff00a7c29725006e125c1e76a985` |
| `codex/xq25-gray-conceal-cost-review-20261013` | `912f2302f924f64921dccc0e700bbcbfe120557e` |

## worktree

当前仓库仅有原 `/workspace/UndergroundBattle` 根工作区，无额外worktree清理候选。重复source-stage目录不是Git worktree。
