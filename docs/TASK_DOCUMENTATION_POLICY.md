# TASK_DOCUMENTATION_POLICY

Purpose: records the repository rule that every completed task must leave behind a short explanatory document in `docs/`.

## Rule

- 每次完成一个任务，都必须在 `docs/` 目录下新增一篇说明文档。
- 文档不要求冗长，但必须解释这次任务里值得说明的内容。
- 可说明内容包括：真相源、设计取舍、生成方式、已知限制、后续待办、风险边界。

## Minimum Expectations

- 文档标题应能直接看出对应任务。
- 文档必须指出这次改动的输入来源和输出落点。
- 若任务存在未完成部分，必须明确写出，不允许靠省略制造“已完成”的假象。
- 若任务依赖人工判断或外部真相源，也必须在文档中标明。

## Intent

- 让后续的人类开发者和 AI 协作者能快速理解上一轮任务的真实结果。
- 降低口头约定丢失的概率。
- 避免只看 diff 无法理解约束、假设和残留问题。

