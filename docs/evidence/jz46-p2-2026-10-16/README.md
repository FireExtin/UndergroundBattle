# JZ46 P2：付费play容器一致性

`before-native.json`／`after-native.json` 与 `single-bit-raw-input.json.gz` 绑定相同单字段反例；`wasm-baseline-red.log.gz` 证明原445e实际接受。`native-red.log.gz` 是修前0/1失败；`native-green.log.gz` 保留首轮10/11的测试设置错误；`native-green-final.log.gz` 是最终11/11，不相加计数。

`wasm-parity.json`、`http-jz46.log.gz`、`frontend-jz46.log.gz` 和 `four-clients.json` 对应修后8fd制品。新Native capsule保存在仓库既有fixture路径，包含11正向及35固定坏档；只新增本P2坏档，不做无界扫描。所有日志压缩保存原字节。截图在Git外原始证据目录，客户端源码及160个产物均未改。

`backend-runtime-manifest.json` 只保存元数据，不包含编译JS/WASM；原78和77字节不变。`verification-summary.json` 明确限定本次验证范围；旧整组结果不能重标为新制品结果。最终Git源码及123个编译输入的绑定见随后 `source-closure.json`。未发布、未合main、未触发Actions、未线上试玩。
