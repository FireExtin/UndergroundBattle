# 空间恢复只读核验（清理 WIP）

本目录只保存审计元数据、核验脚本和两个无凭据的小型源暂存管理文件。没有下载缓存本体、数据库、用户私人数据、原始游戏状态大包或构建缓存。

- `inventory-summary.json`：精确盘点前后可用字节；删除/释放均为 0。
- `directory-sizes.json`：根目录及选定子目录分配块统计，不跨次相加硬链接。
- `process-and-recovery.json`：活动进程、候选路径引用、不可读 FD/maps、回收机制边界。
- `npm-cache-recovery-candidates.json`：7 个精确缓存绝对路径、大小/分配块/哈希，与 main224 锁文件及实际 registry 读取相同的依据；未删除。
- `installed-package-retention.json`：已安装包状态，仅供评估缓存缺失影响，未修改 node_modules。
- `merged-stage-recovery.json`：单个 BQ035 P2 源副本 1,145 文件核验、main 祖先和外部规范 ABI78 保留。
- `stage-retained-manifest-proof.json`：重建映射在 main224 保留的精确 Git blob。
- `stage-extra-owned-files.json`：源副本两个小型管理文件的原字节、哈希和 mode，为可能重建保留；不代表已清理。
- `first-pass-status.json`：首轮按未压缩名查找 ABI78 而停止，纠正读取原 gzip 的记录。
- `inventory-readonly.py`、`finish-merged-stage-readonly.py`：现场只读验证流程，后者续跑源副本阶段；没有删除操作。
- `manifest.json`：上述 11 项证据文件的 SHA256 与字节数。

结论和待父确认的最小方案见 [只读盘点与清理 WIP](../../STORAGE_RECOVERY_AUDIT_WIP_2026-10-18.md)。完整发布事实仍以 v69 官方回执为准，没有为本次检查重复保存/部署站点。
