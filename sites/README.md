# Sites Worker / D1宿主

本目录为React客户端提供HTTP房间服务、D1事务和静态资源，并调用同一Rust WASM规则核。当前候选ABI93单核、旧桌失效边界及测试阶段见[根README](../README.md)；固定677a7b1的单核删除改动已独立补验，本轮夹具迁移尚未独立复审，未发布。

## 构建、本地运行与测试

从仓库根目录执行；先完成Web依赖和当前WASM构建；构建与运行无需旧ABI输入：

```sh
npm --prefix sites ci
npm --prefix sites run build
npm --prefix sites run db:local
npm --prefix sites run dev -- --ip 127.0.0.1
```

`build`调用`wrangler deploy --dry-run`并输出`sites/dist/server`与`sites/dist/client`，不是部署。`db:local`使用本地D1，dev在8102本地运行。当前HTTP测试使用单核固定时钟wrapper；wrapper不能发布。先构建，再从仓库根目录运行：

```sh
node tools/build-current-clock-worker.mjs "$UNDERGROUNDBATTLE_BUILD_ROOT/current-worker/server"
cd sites
CURRENT_CLOCK_WORKER="$UNDERGROUNDBATTLE_BUILD_ROOT/current-worker/server/worker.js" \
  node --test --test-concurrency=1 test/*.test.mjs
```

wrapper目标必须是新目录；standalone Site在其根目录执行构建工具和测试，省略`cd sites`。测试只使用当前核及新导出的共享夹具，迁移后的最终全套状态见根README，不把未完成项称为通过。

[src/http.mjs](src/http.mjs)处理路由与认证入口，[service.mjs](src/service.mjs)协调可信时间、请求意图与回执，[store.mjs](src/store.mjs)保存opaque状态、日志和命令回执。CAS失败的尝试不能旁写日志/回执；创建/加入同样提供requestId回执。数据库故障不应被确认为成功。

[src/kernel.mjs](src/kernel.mjs)只导入当前ABI93内核；[kernel-router.mjs](src/kernel-router.mjs)在执行前检查当前完整身份及schema。旧身份返回410并要求新开；旧状态、回执留存但不再运行，不做迁移。HTTP / workerd / 本地D1 SQLite测试不是线上D1或浏览器验收。

这是隐秘世界后续版本的持续约定：交付只含最新内核，以性能和体积优先；不要重新加入旧核路由、保留包或旧桌迁移。当前身份、u64精度、状态完整性、鉴权和幂等检查仍保留。

## 已有Site及官方发布流程

身份在[.openai/hosting.json](.openai/hosting.json)，项目更新应复用其project_id和访问范围，不重新创建Site。最近确认成功为公开v73；v74既有部署仍pending，v75多核保存版已退出发布队列。GitHub产品仓库与Sites托管源码仓库不同，GitHub main提交不能直接当作Sites的源码commit_sha。

正式发布时使用当前官方Sites插件流程。以下是有发布授权时的操作说明，**本次文档更新没有执行这些发布步骤**：

1. 读取`get_site`确认既有项目和受众。授权发布后取得该项目`create_source_repository_write_credential`，使用插件实际安装路径的`site-workflow.mjs`打开已有Sites源码checkout。
2. 把已审核产品源码同步为该项目的有限源码副本；现有[scripts/stage-source.mjs](scripts/stage-source.mjs)可复制到仓库外的独立目录，并记录GitHub源commit及当前WASM哈希。同步前应处于已保存的源码状态。它不复制原始PDF/完整资料库、运行数据库或凭据，也不包含外部冻结运行字节。
3. 用官方source helper执行剩余检查、构建、源码commit/push及打包。凭据只能通过隐藏stdin传入，不放进命令行、仓库、日志或README。保存时的`project_id`、`commit_sha`、`archive`使用helper验证后的同一结果；归档在保存成功前不改写。不要手写Git/打包替代官方发布工作流。
4. 公开等受支持受众使用`save_site_version`后`deploy_site_version`；确认owner-private的项目使用`save_version_and_deploy_private`或对应私有发布两步。保存版本本身不等于已部署；已有保存版本/部署应继续原ID，不重复创建。
5. 只对非终态部署跟进`get_deployment_status`，以官方终态结果及返回URL为发布证据；不把本地dry-run或GitHub推送称为发布成功。

部分官方源码凭据支持明确接受、限时有效的`publish_on_push`。只有返回该授权窗口时才按其契约处理，源码push可能直接产生私有生产发布；不能假设所有push无副作用，也不能把空/缺失字段当作已取消旧窗口。本轮没有获取写凭据、接受该窗口、save或deploy。

禁止GitHub CI的项目限制仍有效。推main前须确认成果范围、未知远端改动、Actions触发器与外部自动发布配置；应读取实际workflow/config；连接器某接口不受支持不等于存在CI或Git保存被拒。保持权限和账户设置不变，不通过绕过认证或强推消除阻塞。

构建后也可用`node scripts/qa-preview.mjs`启动本地预览；它核对实际生成的当前catalog，不再硬编码ABI78。该预览与历史发布证据均不能代替线上验收。
