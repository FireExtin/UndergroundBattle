# SCHEMA_VERSIONING

Purpose: defines the repository-wide rules for evolving shared schemas without losing fixture or replay integrity.

## Principles

- schema 使用 semver。
- minor 版本尽量向后兼容。
- major 版本必须配套迁移脚本。

## Working Rules

- 所有共享 schema 文件都应显式携带语义化版本号。
- fixture、expectation、回放文件在落盘时都应记录对应 schema version。
- minor 升级优先新增字段、保留旧字段语义，并在消费端提供兼容路径。
- major 升级前必须准备迁移脚本、迁移说明和回归测试。
- 任何破坏性 schema 变更都不能只改文档，必须同步更新工具链和测试。

## Migration Expectations

- 迁移脚本应能批量处理历史 fixture。
- 无法自动迁移时，工具必须显式报错并提示需要人工处理。
- schema 版本策略的验证应进入 Go 与 TypeScript 的自动化测试流程。
