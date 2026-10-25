from pathlib import Path
import json,subprocess,sys
root=Path(__file__).resolve().parent;env=json.loads((root/'native-env.json').read_text())
def run(name,args):
 r=subprocess.run([sys.executable,str(root/'run_stage.py'),'pack-'+name+'-2',*map(str,args)])
 if r.returncode:raise SystemExit(r.returncode)
for name,key in [('batch3-inspections','BATCH3_ABI94_SMOKE_OUT'),('batch3-cleanup','BATCH3_ABI94_CLEANUP_SMOKE_OUT'),('batch3-cq07','BATCH3_CQ07_SMOKE_OUT')]:run(name,['node',root/'pack-smoke-current-counts.mjs',env[key],'rust-game-wasm/tests/current112/'+name+'.json.gz'])
run('batch4',['node','tools/batch4-platform-compare.mjs',root/'native/single/batch4_abi95_native_out.json','rust-game-wasm/tests/current112/jz56-platform.json.gz',root/'batch4-compare.json'])
run('batch20-review',['node','tools/check-batch20-review-fixes.mjs',env['ABI112_REVIEW_NATIVE_OUT'],root/'batch20-review-compare.json'])
run('rewards',['node','tools/check-shared-keyword-rewards.mjs',env['ABI112_REWARD_NATIVE_OUT'],root/'rewards-compare.json'])
run('sentinel',['node','tools/fixture-tools/current-sentinel.mjs',env['SENTINEL_WORKER_WITNESS'],'rust-game-wasm/tests/current112/sentinel-worker.json.gz',root/'sentinel-compare.json'])
run('small',['node',root/'finalize_small_packets.mjs'])
print('REMAINING_PACKETS_COMPLETE',flush=True)
