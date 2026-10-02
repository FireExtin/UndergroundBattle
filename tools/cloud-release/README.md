# 最小云端启动入口

从私有仓库检出所需提交后，`start.sh` 使用锁文件构建原生 Rust 服务和 React 界面并启动。不打包数据库、令牌或原始 PDF/卡图，不自动发布网站。

依赖 Linux、Rust 1.90、Node 24/npm 和构建所需系统工具。在仓库根目录运行：

```bash
bash tools/cloud-release/start.sh
```

云端已配置 Rust 的本环境，可在调用前设置工具路径：

```bash
export PATH=/workspace/.cloud-setup/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/.cloud-setup/rustup
export CARGO_HOME=/workspace/.cloud-setup/cargo
```

默认端口 8090，数据库 `rust-game-v2.1.sqlite3`，可用 `PORT`/`HEGEMONY_DB` 更改；数据库路径应位于持久卷。v2.1为外科医生费用修复后的 `rust-v0.2.1 / limited-v2.1`，状态 schema 仍为2，但固定版本拒绝将v0.1/v0.2.0旧局当作新局加载；旧对局须使用对应原二进制与数据库。当前云端旧服务占用8090/8091/8098时，新核心显式使用其他空闲端口和独立数据库。

相同版本的数据库重新启动恢复房间及待选，浏览器保留原座位令牌才能恢复既有座位。运行中 SQLite 备份使用 backup API，不单独复制正在写入的 WAL 主文件。

这是原生云端启动方法，不是 Worker 部署包。WASM 构建见 [规则核心说明](../../rust-game-wasm/README.md)。已有保存、恢复及只读回放证据见 [云端验收记录](../../docs/CLOUD_PLAYTEST_2026-10-02.md)。互联网入口仍需部署架构确定，本机健康检查不能替代互联网可玩验收。
