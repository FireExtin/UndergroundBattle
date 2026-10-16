"""Publication-only audit of the exact reviewed artifact promotion and source binding."""
from pathlib import Path
import hashlib,json,subprocess

root=Path('/workspace/UndergroundBattle')
provider=Path('/workspace/hidden-world-sites-abi77-20261015')
evidence=Path('/workspace/pass-context-evidence-20261016')
manifest_bytes=subprocess.check_output(['git','-C',str(root),'show','0dc88c4ae48f67a3b14c6a0928d94c5a4e9fef38:docs/evidence/pass-history-2026-10-16/artifact-manifest.json'])
manifest=json.loads(manifest_bytes)
assert manifest['featureSourceCommit']=='8a27c1ca9cfc065cc03a543d73f14b140ac93515'
assert manifest['reviewedOverlayTree']=='ab1ea260a47cb028085dcfede5be9019b6098d87'
expected_dist={'.openai/hosting.json'}
for file in manifest['files']:
    relative=file['path'][len('sites/'):] if file['path'].startswith('sites/') else file['path']
    data=(provider/relative).read_bytes()
    assert len(data)==file['bytes'] and hashlib.sha256(data).hexdigest()==file['sha256'],relative
    if relative.startswith('dist/'):expected_dist.add(relative[len('dist/'):])
actual_dist={str(file.relative_to(provider/'dist')) for file in (provider/'dist').rglob('*') if file.is_file()}
assert actual_dist==expected_dist,{'unexpectedOutputs':sorted(actual_dist-expected_dist),'missingOutputs':sorted(expected_dist-actual_dist)}
source_files=json.loads((evidence/'provider-source-files.json').read_text())
for file in source_files:assert hashlib.sha256((provider/file['path']).read_bytes()).hexdigest()==file['sha256'],file['path']
provenance=json.loads((provider/'source-provenance.json').read_text())
assert provenance==json.loads((evidence/'provider-provenance.json').read_text())
assert provenance['artifactManifestSha256']==hashlib.sha256(manifest_bytes).hexdigest()
for file in provenance['unchangedStandaloneSupport']:
    assert hashlib.sha256((provider/file['path']).read_bytes()).hexdigest()==file['sha256']
    assert (provider/file['path']).read_bytes()==subprocess.check_output(['git','-C',str(provider),'show','eec9ffb:'+file['path']])
allowed={item['provider'] for item in json.loads((evidence/'provider-copy-plan.json').read_text())}|{'source-provenance.json'}
changes=set(subprocess.check_output(['git','-C',str(provider),'diff','--name-only']).decode().splitlines())
changes.update(subprocess.check_output(['git','-C',str(provider),'ls-files','--others','--exclude-standard']).decode().splitlines())
assert changes<=allowed,sorted(changes-allowed)
assert (provider/'.openai/hosting.json').read_bytes()==(provider/'dist/.openai/hosting.json').read_bytes()
assert json.loads((provider/'.openai/hosting.json').read_text())=={'project_id':'appgprj_6abf7bf54a7481918a50e1ef1509ca68','d1':'DB','r2':None}
report={'artifactFilesMatched':len(manifest['files']),'boundedSourceFilesMatched':len(source_files),'exactDistInventory':True,
    'changedFiles':sorted(changes),'sourceProvenance':provenance,'noDatabaseOrMigrationChanges':not any(path.startswith(('db/','drizzle/')) for path in changes)}
(evidence/'publication-input-audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'reviewedArtifactsMatched':len(manifest['files']),'boundedSourceFilesMatched':len(source_files),'changedSourceFiles':len(changes),'exactDistInventory':True,'noDatabaseOrMigrationChanges':report['noDatabaseOrMigrationChanges']}))
