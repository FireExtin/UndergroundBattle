# Card Importer Test Fix

本次任务修复的是 `tools/card-importer` 自身的测试与类型问题，不是 npm 安装环境问题。

## 观察到的症状

- `npm install` 成功后，`npm test` 仍然失败。
- `npm run typecheck` 和 `npm run build` 也失败。
- 失败点集中在四类：
  - 测试辅助函数写临时仓库文件时没有先创建父目录。
  - `vitest` 会把 `dist/*.test.js` 也执行一遍，导致源测试和构建产物测试重复跑。
  - `exactOptionalPropertyTypes` 下，若显式传 `undefined`，现有代码和类型定义不兼容。
  - `RuleDocMeta.id` 被误用成卡牌 ID 的 ASCII 规则，和真实中文规则文档资产不匹配。

## 本次修复

- `src/importer.test.ts`
  - `writeRepoFile` 先 `mkdir -p` 父目录，再写测试文件。
  - 补了对首条 `cardsNormalized` 记录的显式存在性断言，消除严格空值类型错误。
- `src/cli.ts`
  - 改成只在参数存在时才写入 `outputRoot` / `root` / `output`。
  - 避免在 `ImportOptions` 上显式传入 `undefined`。
- `src/importer.ts`
  - 用显式排序辅助函数替代 `toSorted`，兼容当前 `ES2022` lib 目标。
  - 修正了 artwork 索引中可能出现的 `undefined` 捕获值。
  - `loadSchemaVersions` 改为只写入已存在的 schema 字段。
- `src/validators.ts`
  - `RuleDocMeta` 不再复用卡牌/指示物的 ASCII ID 模式校验。
  - 重复 ID 错误改为只在 `sourcePath` 存在时才写入该字段，兼容严格可选属性。
- `src/validators.test.ts`
  - 修正测试里的类型断言写法，兼容严格模式。
- `vitest.config.ts`
  - 将测试范围限制为 `src/**/*.test.ts`，并排除 `dist/**`。

## 结果

以下命令已经通过：

- `cd tools/card-importer && npm test`
- `cd tools/card-importer && npm run typecheck`
- `cd tools/card-importer && npm run build`
- `go test ./...`
- `cd tools/fixture-tools && npm test`
- `cd tools/fixture-tools && npm run typecheck`
- `cd web && npm test`
- `cd web && npm run typecheck`

## 剩余说明

- `tools/card-importer` 的 `build` 目前仍会把测试文件编译进 `dist`，但 `vitest.config.ts` 已保证这些构建产物不会被测试命令重复执行。
- 如果后续希望让 `dist` 只包含运行时代码，可以再拆一个 `tsconfig.build.json`，把 `*.test.ts` 排除出构建。
