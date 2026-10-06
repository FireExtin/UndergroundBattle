# Sites36 默认测试与干净构建（2026-10-06）

此测试/构建说明候选基于父已审 `4de46d2952bfd5d570f0ed18854e6acc293cad57`，不修改Rust规则、WASM源码、冻结文件、生产Worker源码、依赖锁或发布配置。原候选保持固定；此提交仍需父审后才可参加用户约定的22点main筛选。未提前写main、推送或发布。

## 失败分类与有限修正

原样默认 `cd sites && npm test` 实跑23项：20通过、3失败、0跳过。失败日志完整保留，三项都在旧11的首个身份/数量断言处终止；没有以定向运行替代默认测试。

| 入口 | 契约与修正 |
| --- | --- |
| integration.test.mjs | 当前HTTP入口应为已审36/pool33、100张普通卡。更新这些精确断言，增加JZ31/JZ55存在检查，并将当前36加入已有paced版本处理。MSJC01已正式受支持，不再作为非法秘社；非法用例改用明确不存在的秘社ID。旧1..10房间回执、卡池数量和版本约束断言保留。 |
| kernel-routing.test.mjs 原测试 | 内容是固定11对10的JC005增量与旧核兼容测试。明确绑定原冻结11，保留原50/49张、JC005增量和所有旧身份拒绝断言；另加当前36对已审冻结35的真实契约检查。 |
| msjc09-society-integration.test.mjs | 内容是只有MSJC09的固定11自然HTTP场景。用原冻结11在私有输出中编译临时Worker并运行，保留50张、唯一秘社及原非法秘社断言；没有改初始布局或注入数据库对局状态。 |

默认测试选择器仍是原 `node --test test/*.test.mjs`，未添加skip/only、失败屏蔽或新的测试框架。上述临时Worker只沿用现有后端dry-run helper的一个封闭 `--legacy-v0.2.11` 模式；仅改临时副本中当前核声明，身份来自实际原ABI。两个隔离模式均不回写生产generated/dist，正常无参数模式仍精确使用已审36。

修正后原样默认 `npm test` 已到明确终态：24项全部通过、0失败、0跳过（94.63秒）。额外fixture另计1项通过；不加入默认24项统计。正常Worker运行代码、全部38个WASM以及新生成当前pkg的五个文件，均逐字节等于已审501产物。原23项20通过3失败的基线日志、fixture身份拒绝日志和最终成功日志均保留。

## 干净游戏checkout的当前36构建

当前 `rust-game-wasm/pkg/` 是忽略的构建产物，Git checkout不会带来它。这里先从源码生成，不能拷贝其他工作树的遗留pkg作为干净构建证明。

工具链来自仓库 `rust-toolchain.toml` 的Rust 1.90.0、Cargo.lock及精确wasm-bindgen 0.2.104依赖。预先安装 `wasm32-unknown-unknown` target和相同版本的wasm-bindgen CLI。缺少工具时可用 `rustup target add --toolchain 1.90.0 wasm32-unknown-unknown`，以及 `cargo +1.90.0 install wasm-bindgen-cli --version 0.2.104 --locked --root "$task_tools"`；后者是工具安装步骤，不修改游戏依赖锁。

在游戏根执行以下流程，`task_build_cache` 是绝对路径的私有构建目录，`WASM_BINDGEN` 指向已核版本的CLI：

```sh
: "${task_build_cache:?Set a separate absolute build directory}"
: "${WASM_BINDGEN:?Set the verified 0.2.104 executable}"
rustc --version                 # 1.90.0
"$WASM_BINDGEN" --version       # 0.2.104
export CARGO_TARGET_DIR="$task_build_cache/production-target"
bash rust-game-wasm/build.sh    # cargo --locked --release；不带fixture feature
node --input-type=module <<'JS'
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as k from './rust-game-wasm/pkg/hegemony_wasm.js';
k.initSync({module:readFileSync('rust-game-wasm/pkg/hegemony_wasm_bg.wasm')});
const c=JSON.parse(k.catalog());
assert.equal(c.engineVersion,'rust-v0.2.36-jz55-unique-destroy-candidate');
assert.equal(c.cardPoolVersion,'limited-v2.33-jz55-unique-destroy-candidate');
assert.equal(c.cards.length,100);assert.equal(c.societies.length,8);
assert(c.societies.every(s=>!s.id.startsWith('FIXTURE_')));
assert.equal(createHash('sha256').update(readFileSync('rust-game-wasm/pkg/hegemony_wasm_bg.wasm')).digest('hex'),'0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036');
JS
npm --prefix web ci
npm --prefix sites ci
npm --prefix sites run build
npm --prefix sites test
```

本次云端新工作树在开始时确实没有pkg/generated；使用空的专用production-target、Rust1.90.0与CLI0.2.104实跑上述顺序。`CARGO_HOME=/workspace/jz31-tools/cargo`、`RUSTUP_HOME=/workspace/jz31-tools/rustup` 和该目录bin加入PATH只是云端已有工具的位置；包输出在本新工作树。生成36的WASM SHA256精确等于已审值，38WASM的Sites dry-run构建和原web类型/Vite构建成功。最终默认测试结果与原失败日志一并在审查包中，不混用之前的定向测试统计。

## 额外society-fixtures的准确生成与隔离

该模式不属于默认 `*.test.mjs` 选择器。当前feature生成独立fixture36身份、100张普通卡与三个 `FIXTURE_` 秘社；原fixture入口的旧8/48张标量过时，改为实际当前fixture契约，四座位、约束、付费、回执、重开与自然重置行为断言保留。

```sh
# 游戏根；与生产target隔离，输出固定在被忽略的pkg-society-fixtures。
CARGO_TARGET_DIR="$task_build_cache/fixture-target" bash rust-game-wasm/build.sh --society-fixtures
# sites目录；输出必须私有，helper仅改临时副本身份，不回写生产包。
cd sites
HEGEMONY_BACKEND_TEST_ROOT="$task_build_cache/fixture-worker" node scripts/build-backend-test.mjs --society-fixtures
HEGEMONY_SOCIETY_WORKER_DIST="$task_build_cache/fixture-worker/fixture-worker/dist" node --test test/society-foundation.fixture.mjs
```

实际生成并核对fixture身份为 `rust-v0.2.36-jz55-unique-destroy-fixture`，pool仍是当前33；三个秘社均为FIXTURE，生产pkg哈希不变。原helper在生成包存在后仍因正常36的静态lazy声明而拒绝fixture，旧失败已保留。本修正只在隔离副本读取实际ABI身份并替换既有当前核声明，保留正常生产源码与全部历史字节；fixture没有加入生产路由。额外fixture测试须显式执行，不能并入默认通过计数。

所有Wrangler调用仅dry-run。官方Sites包下载403仍按父指示暂停，Air不参与研发。本结果是本地workerd/D1仿真及源码构建证据，不表示线上发布或云端真实D1验收。
