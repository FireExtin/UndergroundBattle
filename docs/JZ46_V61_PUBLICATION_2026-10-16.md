# JZ46 完整批次官方发布 v61（2026-10-16）

父会话确认完整批次 `e7d7b4dd9be2875a5879e0fa833f20cb241ebe6c`／`819a5d41ee0f9add2793859d82ff9385cabfd17a` 已独立复审通过，P2关闭；承接用户对已检查版本持续公开发布的明确授权。本次只沿官方Sites流程发布现有网站，没有推进UndergroundBattle main，没有触发Actions，没有清库或读写真实用户桌。原交接中“未发布／待复审”是当时状态。

## 官方结果

公开网站：[隐秘世界 · 霸权](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)。官方保存版本 **61**，部署状态 **succeeded**，时间 `2026-10-16T20:00:32.003259+00:00`。

- project：`appgprj_6abf7bf54a7481918a50e1ef1509ca68`
- version：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_191645bc523481918cd41e74942b4f19`
- deployment：`appgdep_6ad2824fbaac8191860d8661bf9f1f84`
- Sites源码：**`906a1b4c54e43eff5dd722c275d55ad567b8c20c`**；保存、发布后版本读取均返回同一来源。

官方helper打开原6d1a1fed检出，正常提交推送到906a1b4c；沿用原项目、公开访问、revision 2、逻辑D1 `DB`及原迁移字节。GitHub游戏main仍为 `bd36b70b32890f539fde019613f5ae6d60c2903d`；Sites独立源码库的既有main正常同步不改变游戏main。Actions仍为原有8次，没有工作流修改或手动运行。完整原生回执见 [official-receipts.json](evidence/jz46-v61-publication-2026-10-16/official-receipts.json)。

## 固定来源与发布字节

复制前核对已审游戏保存点819a5d41及其与e7d7b4dd的非文档源码一致性；全部123个编译输入、P2运行文件及160个前端文件哈希匹配已验记录。官方检出共1163个映射文件逐项一致；仅保留官方检出原有prebuild/pretest夹具还原hook和无损gzip储存配置，未增加运行行为。db、drizzle、hosting与发布前字节一致。见 [source-mapping.json](evidence/jz46-v61-publication-2026-10-16/source-mapping.json)。

复用已验8fd新79、原78和原77，不重新编译或重跑已通过测试。官方helper首份完整输出归档含非运行README与source map；保留该档，按原发布范围排除这两项，再由官方helper生成最终归档。最终168个文件全部逐字节核对：164运行文件、hosting和3个原有drizzle元数据文件。没有数据库、凭据、缓存或新迁移。见 [archive-verification.json](evidence/jz46-v61-publication-2026-10-16/archive-verification.json)。

| 内核 | SHA256 |
| --- | --- |
| 当前79 | `8fdc829ab25d0eb5d68d973489fafff6b70e0da8b7837621a3ef86d52360f0fd` |
| 已发布原78 | `9511a72c960cf3e0fa249d62669724875d475ab245de4f9616615f38df43fd5f` |
| 保留原77 | `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c` |

旧房继续按精确三元组使用原78／77，不迁移或重写opaque状态、回执。P2复核范围为Native11、WASM11场景／368命令／1516投影及35坏档420恢复调用、HTTP15、前端14及真实本地四席流程，详见 [P2交接](JZ46_P2_PLAY_CONTAINER_REVIEW_2026-10-16.md)。旧完整workspace／433／全前端／全Worker的历史和分轮边界保持原记录，没有重标为当前制品一次全绿。

本地上传gzip、解压tar和平台保存归档的身份分别记录在 [archive-identities.json](evidence/jz46-v61-publication-2026-10-16/archive-identities.json)。平台确认source、168文件及成功deployment；没有下载平台归档再次逐内部文件核对，不把封装哈希不同解释为已证明整份tar相同。

## 线上验证边界

官方成功结果及URL确认发布；发布后get_site／get_site_version再次确认v61、906a1b4c来源、public和revision 2。后台任务没有额外浏览器交接，没有重试此前普通浏览器证书错误，没有使用访问bypass，也没有进行线上建桌、入桌或真实试玩。本地组件／HTTP／浏览器证据仍不计线上试玩。

发布归档和原始大证据保留在Git外 `/workspace/jz46-publication-20261016/`。凭据仅通过官方helper隐藏stdin传递，会话结束前清除内存引用；不进入脚本参数、文件或Git。发布回执文档保存在原独立进度分支，main保持不变。
