from pathlib import Path
import datetime,gzip,hashlib,json,re,shutil,subprocess,tarfile
root=Path(__file__).resolve().parent
repo=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
pin='64e76dfcdfae8be35fd2b68f53e9c855d0efb98d'
read=lambda name:json.loads((root/name).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
for name in ['runs/fixed-native-06/receipt.json','cross-logs/wasm-build.json','cross-logs/pack-natural.json','cross-logs/worker-build-isolated.json','cross-logs/worker-natural.json']:
    d=read(name);assert d['state']=='complete' and d['exit_code']==0,name
assert read('runs/fixed-native-06/receipt.json')['HEAD']==pin
comparison=read('natural-cross-comparison.json');audit=read('natural-reachability-audit.json')
assert audit['native_commands']==1075 and sum(c['productionReplayedCommands'] for c in comparison['cases'])==1066
git=lambda *a:subprocess.check_output(['git',*a],cwd=repo,text=True)
assert git('rev-parse','HEAD').strip()==pin and not git('diff','HEAD')
deck=git('show',pin+':rust-game/src/deck.rs')
guard='    #[cfg(test)]\n    if let Some(cards) = crate::next20_natural_room_tests::candidate_registry() {\n        return validate_with_cards(draft, &cards);\n    }\n'
assert deck.count(guard)==1 and deck.replace(guard,'')==git('show','dc7ff9ab:rust-game/src/deck.rs')
lib=git('show',pin+':rust-game/src/lib.rs');assert lib.replace('#[cfg(test)]\nmod next20_natural_room_tests;\n','')==git('show','dc7ff9ab:rust-game/src/lib.rs')
assert not git('diff','1bd0776',pin,'--','rust-game/data/cards.json')
identity={'tested_source_SHA':pin,'previous_runtime_implementation_SHA':'1bd07762824e138c0560863130ec7eacaddbf9c8','production_semantics_change':'none; deck hook and module only cfg(test)','cards_json_bytes_unchanged':True,'candidate_registry':'thread-local, only BQ034/CQ25/XG25, reset on scope exit; global catalog false','new_source_files':git('diff','--name-only','dc7ff9ab',pin).splitlines(),'deck_diff':git('diff','dc7ff9ab',pin,'--','rust-game/src/deck.rs'),'lib_diff':git('diff','dc7ff9ab',pin,'--','rust-game/src/lib.rs')}
(root/'runtime-source-boundary.json').write_text(json.dumps(identity,indent=2)+'\n')

summary={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'tested_source_SHA':pin,'publicAdmission':False,'candidateConstructionOnly':['BQ034','CQ25','XG25'],
 'Native':{'pass':4,'fail':0,'old_tests_filtered_out':2561,'full_old_suite':'NOT_RUN at this source','receipt':'runs/fixed-native-06/receipt.json','natural_commands':1075},
 'limited_WASM':{'cases':3,'commands_after_candidate_start':1066,'views':2132,'actual_duplicate_reducer_probes':1066,'production_candidate_start_rejected':True,**comparison},
 'limited_Worker':{'pass':3,'fail':0,'seconds':86.751,'commands':1066,'each_original_HTTP_receipt':True,'each_persisted_state_journal_and_two_seat_views':True,'D1_reopens_per_case':4,'local_workerd_only':True},
 'builds':{'WASM':'PASS fresh release','UI_typescript_vite':'PASS','Worker':'PASS dry-run only, no deployment'},
 'candidate_setup_boundary':'The 3 Native ready/start commands per case (9 total) use candidate construction admission. Production start rejects. WASM/D1 begins at the genuine Native post-start state; public create/join/start admission is not claimed.',
 'known_source_issue':'BQ034 raw unique:true but current directory deckCopyLimit=3; all these decks use exactly one, fillers exclude raw unique. No production rule changed; review before admission.',
 'remaining':['BQ009/BQ033 seal supply is still absent and out of scope','Production supported:false on all five remains','Complete card admission and parent review pending','New real browser play/remote D1 acceptance NOT_RUN','Old full Native/UI/Worker/WASM483 suites NOT_RUN at this source; historical 9afd8a57 evidence retained under its original SHA'],
 'permissions':'Original Vite dependency-cache write was stopped after EROFS; no retry/elevation of that target. Only independently copied dependencies in authorized Task7 were built.',
 'remote':'main and two review refs read unchanged; progress/batch20-fourth-20261022 read failed with RemoteProtocolError and was not retried. Workflow allowlist rejection not retried.',
 'push':'NOT_ATTEMPTED','CI':'NOT_TRIGGERED','deployment':'NOT_ATTEMPTED','original_table':'97EC1A99E58C and both seats untouched','original32':'unchanged',
 'cases':[{'name':c['name'],'seed':c['seed'],'native_commands':c['nativeCommands'],'cross_commands':c['productionReplayedCommands'],'views':c['views']} for c in comparison['cases']]}
(root/'scope-summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')

metadata=['frozen-source.json','source-checkpoint.json','runtime-source-boundary.json','scope-summary.json','natural-reachability-audit.json','natural-cross-comparison.json','packet-manifest.json','known-source-consistency.json','failed-source-recovery.json','isolated-dependencies.json','cross-build-cache-reuse.json','remote-refs-readonly.json']
scripts=['run_native.py','run_cross.py','run_limited_worker.mjs','pack_natural.mjs','audit_natural.py','archive_natural.py']
selected={root/name for name in metadata+scripts}
for pattern in ['runs/*/receipt.json','runs/*/output.log','runs/*/source-files/**/*','runs/*/native/*.jsonl','cross-logs/*','source-versions/*.rs','fixtures/**/*.gz']:
    selected.update(p for p in root.glob(pattern) if p.is_file())
patterns={'private_key':rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----','github_token':rb'\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})\b','aws_key':rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b','google_api_key':rb'\bAIza[0-9A-Za-z_-]{35}\b','credential_url':rb'https?://[^\s/":@]+:[^\s/"@]+@'}
inventory={}
for p in sorted(selected):
    assert p.is_file() and not p.is_symlink();rel=str(p.relative_to(root));data=p.read_bytes();payload=gzip.decompress(data) if p.suffix=='.gz' else data;payload.decode()
    for name,pattern in patterns.items():assert not re.search(pattern,payload),(rel,name)
    assert not any(x in p.parts for x in ['node_modules','.wrangler','target','dist'])
    inventory[rel]={'bytes':len(data),'sha256':sha(data)}
index={'source_commit':pin,'scope':'Synthetic original Native full/partial traces, three strict packets, all failures, exact compiler inputs and text; no credentials, private user data, databases, dependency caches or build binaries','strong_secret_scan':'PASS','files':inventory}
index_path=root/'natural-evidence-index.json';index_path.write_text(json.dumps(index,indent=2)+'\n')
archive=root.parent/'checkpoints/next20-natural-evidence-20261025.tar.gz';assert not archive.exists()
with tarfile.open(archive,'w:gz',compresslevel=3) as tar:
    for p in sorted(selected):tar.add(p,arcname=str(p.relative_to(root)),recursive=False)
    tar.add(index_path,arcname=index_path.name,recursive=False)
record={'path':str(archive),'bytes':archive.stat().st_size,'sha256':sha(archive.read_bytes()),'source_commit':pin,'entries':len(selected)+1}
(root/'evidence-archive.json').write_text(json.dumps(record,indent=2)+'\n')

deliver=root/'deliverable';assert not deliver.exists();e=deliver/'docs/evidence/next20-natural-reachability-20261025';e.mkdir(parents=True)
for name in metadata+['natural-evidence-index.json','evidence-archive.json']:shutil.copy2(root/name,e/name)
repro=e/'reproduction';repro.mkdir()
for name in scripts:shutil.copy2(root/name,repro/name)
logs=e/'logs';logs.mkdir();log_index=[];ansi=re.compile(r'\x1b\[[0-9;?]*[ -/]*[@-~]')
for folder in sorted((root/'runs').iterdir()):
    for filename in ['receipt.json','output.log']:
        p=folder/filename
        if not p.exists():continue
        dest=logs/(folder.name+('-receipt.json' if p.suffix=='.json' else '.log'))
        if p.suffix=='.json':shutil.copy2(p,dest)
        else:
            data=p.read_bytes();(logs/(folder.name+'.raw.log.gz')).write_bytes(gzip.compress(data,compresslevel=9,mtime=0));dest.write_text('\n'.join(l.rstrip() for l in ansi.sub('',data.decode()).splitlines()).rstrip()+'\n');log_index.append({'path':str(p.relative_to(root)),'raw_sha256':sha(data),'raw_gzip':folder.name+'.raw.log.gz'})
for p in sorted((root/'cross-logs').glob('*')):
    if p.suffix=='.json':shutil.copy2(p,logs/p.name)
    else:
        data=p.read_bytes();(logs/(p.stem+'.raw.log.gz')).write_bytes(gzip.compress(data,compresslevel=9,mtime=0));(logs/p.name).write_text('\n'.join(l.rstrip() for l in ansi.sub('',data.decode()).splitlines()).rstrip()+'\n');log_index.append({'path':str(p.relative_to(root)),'raw_sha256':sha(data),'raw_gzip':p.stem+'.raw.log.gz'})
(e/'raw-log-index.json').write_text(json.dumps(log_index,indent=2)+'\n')
report=f'''# NEXT20 三卡候选合法构筑自然轨迹 · 2026-10-25

固定测试源码 `{pin}`。本片只新增 `cfg(test)`、线程作用域内的 BQ034/CQ25/XG25 构筑许可及自然轨迹测试；正式目录、其他关闭牌和生产准入均不变。仍是277条目录、233张受支持，BQ009/BQ033及这三张均 `supported:false`。测试作用域只改变验证用目录副本的三个 supported 位，复用原构筑校验，退出及 panic 时恢复。生产 release 没有该许可入口。

| 自然路径 | 种子 | Native Room 命令 | 最后回合 | 有限 WASM/D1 命令 |
| --- | ---: | ---: | ---: | ---: |
| BQ034 进场饮血及牺牲附属反制 | 1603 | 386 | 8 | 383 |
| BQ034 展示后选择本方吸血鬼饮血 | 1603 | 332 | 7 | 329 |
| CQ25/XG25 月标、死亡封印、封印后移动、非空检索 | 1723 | 357 | 7 | 354 |

Native 本片 **4项通过/0失败**（3条路径加候选作用域/复制限制），2561项旧 library 测试过滤，未跑旧完整套件。固定源码执行的命令、源文件哈希、原始日志见 [Native receipt](evidence/next20-natural-reachability-20261025/logs/fixed-native-06-receipt.json)。1075条命令每步实际调用 Room.transition、from_persisted 并记录双席视图和实际 reducer 重复探针；暂停/恢复也是普通 SessionAction 命令。

有限 WASM 与实际本地 workerd/D1 三案均通过：**1066条命令、2132个双席视图、1066个实际 reducer 重复探针**；Worker逐条比较完整状态、journal、视图和原始 HTTP 重复收据，每案4次D1重开。Worker三项耗时86.751秒，0失败/0跳过。新WASM release、UI TypeScript/Vite和Worker dry-run构建通过。WASM SHA256：`{comparison['currentWasmSha256']}`；完整源码指纹：`{comparison['currentSourceManifestSha256']}`。

## 构筑和真实来源

每席卡组均50张，同名最多三张（包括无知路人也只用三张），BQ034仅一张；填充牌还排除了原数据标 unique 的牌。用原 Game.new_with_deck/join_with_deck 创建候选大厅，保留两副完整卡组、种子、创建后和加入后状态；其后 ready/start、换牌、逐回合建立资产、抽牌和全部行动都走真实 Room 命令。没有运行中 make_card、填资产/墓地、改牌库或赋月标。三案分别实际建立了16、14、14张资产。源码与自动审计见 [自然来源审计](evidence/next20-natural-reachability-20261025/natural-reachability-audit.json)。

BQ034 两案的 XQ14 均先从合法手牌正常结附于 JC029，再由敌方 JC005 消灭，实际进入拥有者墓地。进场案随后由 BQ034 饮血并在被 JC102 指定后牺牲该鲜血、终止原事务，角色保留且伤害0；展示案用实际 BQ090 手牌行动启动隐秘对抗，展示 BQ034，选择真实 JC029并饮用上述墓地鲜血。

黑卡案由 CQ07 实际派遣和进场触发把隐秘从0变1（命令265）。XG25 被 BQ090 消灭后，合法选择敌方角色作为载体；封印牌拥有者0、载体操控者1。拥有者0实际支付1发动封印后移动（命令345，命令347结算，从相邻地区1移动至0，限次记录uses=1）。随后消灭移入的敌方角色，CQ25触发，从本方**牌库**非空检索XG25并封到伥虎上（命令354）。这些均不依赖 BQ009/BQ033。

## 候选准入与跨端边界

这是隔离候选构筑可达性，不是公开准入。每案前3条 Native ready/start 命令使用候选构筑许可；生产 WASM 的对应 start 实际拒绝，保持完整回滚。有限 WASM/D1 明确从 Native 真实开局后的状态接入，重放其余命令。**9条 Native setup命令不宣称具有生产HTTP准入/收据覆盖**；D1初始状态来自已记录的真实候选开局，后续没有人工填入资产、墓地或标志。公开 create/join/start 三卡可玩性没有被开启或宣称。

本片不覆盖三卡所有边界。CQ25来源离场检索、死亡封印失败入墓等先前定向证据仍保留原来源，本片自然路径不冒充这些分支的自然证明。完整 Native/UI/Worker/WASM483旧套件、新浏览器试玩、远端Cloudflare D1验收均 **NOT_RUN** 于本源码；原9afd8a57完整验证保持原SHA，不重签。

## 保留的失败与待复核项

全部初始失败、部分轨迹及其精确编译输入均保留：最初误以为目录已限制BQ034两张，实际接口允许两张；测试牌组填充池不足；预测选择漏 top/bottom；普通战斗漏 allocations。这些脚本修正未改规则内核。早期编译输入恢复副本均以运行前捕获SHA256核验，未改Native输出或日志。

另有来源一致性问题：原始BQ034记录 `unique:true`、keywords为空，但当前目录 `deckCopyLimit=3`。所有本片牌组只用一张，普通四张超限反例仍通过；没有擅自改变唯一规则或生产目录。该差异须父端复核，详见 [记录](evidence/next20-natural-reachability-20261025/known-source-consistency.json)。BQ009/BQ033封印供给仍明确缺失、未补供给机制。

首个Worker构建尝试通过依赖符号链接写原只读Vite缓存，EROFS后停止。没有提权或重试被拒目录；随后只在已授权Task7目录复制依赖、排除临时缓存，使用独立副本构建成功。失败原日志和路径保留。实际数据包不含依赖、缓存、数据库或构建产物。

## 本地交接

完整原始/部分Native轨迹、三份严格指纹候选包、全部失败和脚本保存在独立证据归档：`{record['path']}`；SHA256 `{record['sha256']}`。逐文件哈希见 [index](evidence/next20-natural-reachability-20261025/natural-evidence-index.json)。源码/测试在现有开发分支正常提交，保留全部原history；最终本地bundle及归档提交SHA记录于本轮交接JSON。

原桌97EC1A99E58C、双席、原32输入、网站和部署未动。没有push、GitHub CI、main合并、发布或新增并行开发写者。main与两个review远端ref只读核对未变；progress分支此次连接器断开，未重试或换路；workflow白名单拒绝未重试。父端继续审查，三卡保持关闭。
'''
(deliver/'docs/NEXT20_NATURAL_CANDIDATE_REACHABILITY_2026-10-25.md').write_text(report)
print(json.dumps({'archive':record,'selected_original_files':len(selected),'original_bytes':sum(x['bytes'] for x in inventory.values()),'deliverable':str(deliver),'summary':summary},ensure_ascii=False,indent=2))
