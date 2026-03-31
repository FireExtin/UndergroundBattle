# Card Importer NPM Cache Fix

本次任务只处理 `tools/card-importer` 的本地依赖安装环境问题，不涉及业务逻辑。

## 问题

- `tools/card-importer` 是一个嵌套的独立 npm 项目。
- 从该目录执行 `npm install` 时，npm 的 project config 根是 `tools/card-importer`，不会自动读取仓库根目录的 `.npmrc`。
- 本机默认 cache 仍指向 `~/.npm`，而该目录中存在 root-owned 缓存文件，导致 `npm install` 触发 `EACCES`。

## 修复

- 在 `tools/card-importer/.npmrc` 中显式指定本项目使用用户可写 cache：

```ini
cache=${HOME}/.cache/undergroundbattle-npm
```

## 验证

- 在 `tools/card-importer` 目录执行 `npm config get cache`，结果应为：

```text
/Users/ddd/.cache/undergroundbattle-npm
```

## 结论

- 该修复只解决 `tools/card-importer` 的安装链路。
- 若未来还有其他嵌套 npm 子项目需要单独安装依赖，应为对应子项目单独放置 `.npmrc`，或显式传入 `--cache`。
