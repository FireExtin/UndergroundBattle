"""Exercise entrypoint/preflight boundaries with the real storage guard, no builds."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


REPOSITORY = Path(__file__).resolve().parents[2]


class GeneratorEntrypoints(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="native72 guard ", dir="/workspace" if Path("/workspace").is_dir() else None)
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self.base / "source with spaces"
        self.bin = self.base / "bin"
        self.bin.mkdir()
        for name in ["Cargo.toml", "rust-game/Cargo.toml", "rust-game-wasm/Cargo.toml", "rust-game/src/lib.rs", "rust-game-wasm/src/lib.rs", "web/package.json"]:
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("untouched source\n")
        tools = self.source / "tools/fixture-tools"
        tools.mkdir(parents=True)
        for name in ["regenerate_native72_inputs.py", "regenerate-and-check-native72.sh"]:
            shutil.copyfile(REPOSITORY / "tools/fixture-tools" / name, tools / name)
        shutil.copyfile(REPOSITORY / "tools/build-storage.mjs", self.source / "tools/build-storage.mjs")
        shutil.copyfile(REPOSITORY / "rust-game-wasm/build.sh", self.source / "rust-game-wasm/build.sh")
        for name in ["jc061", "jc087", "jc094", "wm056", "wm023", "xq25", "wm038", "jz35", "jc052", "jc037"]:
            (tools / f"{name}-current.py").write_text("import json,sys; print(json.dumps(sys.argv))\n")
        for name in ["current-four-faction-choices.py", "current-win-flow.py"]:
            (tools / name).write_text("print('fixture stub')\n")
        (tools / "current-bq030.mjs").write_text("console.log('fixture stub');\n")
        wasm_tests = self.source / "rust-game-wasm/tests"
        wasm_tests.mkdir()
        (wasm_tests / 'current-kernel.mjs').write_text("import {readFileSync} from 'node:fs'; const data=JSON.parse(readFileSync(new URL('../../rust-game/data/cards.json',import.meta.url))); export const catalog={...data,...(process.env.STUB_ABI_ENGINE?{engineVersion:process.env.STUB_ABI_ENGINE}: {}),...(process.env.STUB_ABI_POOL?{cardPoolVersion:process.env.STUB_ABI_POOL}: {})};\n")
        self.pool = self.source / 'rust-game/data/cards.json'
        self.pool.parent.mkdir(parents=True, exist_ok=True)
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.72-jz35-host-return-candidate','cardPoolVersion':'limited-v2.65-jz35-host-return-candidate'}))
        for name in ["jc061", "jc087", "jc094", "wm056", "wm023", "xq25", "wm038", "jz35", "jc052", "jc037"]:
            (wasm_tests / f"{name}_compare.mjs").write_text("console.log('parity stub');\n")
        (self.source / "web/src/game").mkdir(parents=True)
        self.calls = self.base / "calls.jsonl"
        stub = """#!{python}
import json,os,sys
from pathlib import Path
with Path(os.environ['STUB_CALLS']).open('a') as f:
    f.write(json.dumps({{'command':Path(sys.argv[0]).name,'args':sys.argv[1:],'ready':os.environ.get('UNDERGROUNDBATTLE_BUILD_STORAGE_READY'),'target':os.environ.get('CARGO_TARGET_DIR'),'temporary':os.environ.get('TMPDIR')}})+'\\n')
if Path(sys.argv[0]).name=='cargo' and 'STUB_FILL_DISK' in os.environ:
    Path(os.environ['STUB_FILL_DISK']).touch()
if Path(sys.argv[0]).name=='cargo' and 'STUB_LOG_COLLISION' in os.environ:
    log=Path(os.environ['STUB_LOG_COLLISION'])
    if not log.exists(): log.write_text('preserve colliding log\\n')
if 'XQ24_TRACE_DIR' in os.environ:
    directory=Path(os.environ['XQ24_TRACE_DIR']);directory.mkdir(parents=True,exist_ok=True)
    catalog=json.loads(Path('rust-game/data/cards.json').read_text())
    for n in range(12): (directory/f'case-{{n:02}}.json').write_text(json.dumps({{'catalog':catalog,'name':str(n)}}))
if 'XQ24_CONTAINER_EVIDENCE_FILE' in os.environ:
    Path(os.environ['XQ24_CONTAINER_EVIDENCE_FILE']).write_text('{{"opaque":"unaltered Native export"}}\\n')
if Path(sys.argv[0]).name=='wasm-bindgen':
    Path(sys.argv[sys.argv.index('--out-dir')+1]).mkdir(parents=True,exist_ok=True)
""".format(python=sys.executable)
        for name in ["cargo", "wasm-bindgen", "npm"]:
            path = self.bin / name
            path.write_text(stub)
            path.chmod(0o755)
        git = self.bin / 'git'
        git.write_text(f'#!{sys.executable}\nprint("9890163a093621f814f9768712e2663de2fd5239")\n')
        git.chmod(0o755)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        STUB_CALLS=str(self.calls), CARGO_TARGET_DIR=str(self.base / "target"),
                        WASM_BINDGEN=str(self.bin / "wasm-bindgen"),
                        UNDERGROUNDBATTLE_BUILD_ROOT=str(self.base / "build"))
        self.env.pop("NODE_OPTIONS", None)
        self.env.pop("UNDERGROUNDBATTLE_BUILD_STORAGE_READY", None)
        self.env.pop("JZ35_SOURCE_ROOT", None)
        self.env.pop("JZ35_REVIEW_ROOT", None)
        self.python = tools / "regenerate_native72_inputs.py"
        self.shell = tools / "regenerate-and-check-native72.sh"

    def invoke(self, entry, output, extra_env=None, arguments=None):
        env = dict(self.env, **(extra_env or {}))
        if entry == "shell":
            command = ["bash", str(self.shell)] + (arguments if arguments is not None else [str(self.source), str(output)])
        else:
            command = [sys.executable, str(self.python)] + (arguments or [])
            if arguments is None:
                env.update(JZ35_SOURCE_ROOT=str(self.source), JZ35_REVIEW_ROOT=str(output))
        return subprocess.run(command, cwd=self.base, env=env, text=True, capture_output=True, timeout=30)

    def snapshot(self):
        return {str(p.relative_to(self.base)): (p.read_bytes(), p.stat().st_mtime_ns)
                for p in self.base.rglob("*") if p.is_file()}

    def test_nonempty_output_rejects_without_changes_for_both_entries(self):
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
                output = self.base / (entry + " retained output")
                (output / "evidence/ui-refresh").mkdir(parents=True)
                (output / "evidence/ui-refresh/jc061.log").write_text("original log\n")
                (output / "evidence/ui-refresh/progress.json").write_text("original progress\n")
                before = self.snapshot()
                result = self.invoke(entry, output, {"UNDERGROUNDBATTLE_BUILD_STORAGE_READY": "1"})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("empty", result.stderr.lower())
                self.assertEqual(before, self.snapshot())
                self.assertFalse(self.calls.exists())
                self.assertFalse((self.base / "target").exists())
                self.assertFalse((self.base / "build").exists())

    def test_real_storage_guard_rejects_low_disk_before_any_mutation(self):
        mock = self.base / "low-storage.mjs"
        mock.write_text("import fs from 'node:fs'; import {syncBuiltinESMExports} from 'node:module'; const original=fs.statfsSync; fs.statfsSync=(...args)=>({...original(...args),bavail:0n}); syncBuiltinESMExports();\n")
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
                output = self.base / (entry + " low disk")
                before = self.snapshot()
                result = self.invoke(entry, output, {"NODE_OPTIONS": f'--import="{mock}"', "UNDERGROUNDBATTLE_BUILD_STORAGE_READY": "1"})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Build preflight failed", result.stderr)
                self.assertEqual(before, self.snapshot())
                self.assertFalse(output.exists())
                self.assertFalse(self.calls.exists())
                self.assertFalse((self.base / "target").exists())

    def test_output_file_and_symlink_boundaries_preserve_originals(self):
        original = self.base / "existing file"
        original.write_text("preserved\n")
        empty = self.base / "empty symlink target"
        empty.mkdir()
        link = self.base / "output link"
        link.symlink_to(empty, target_is_directory=True)
        broken = self.base / "broken output link"
        broken.symlink_to(self.base / "missing target", target_is_directory=True)
        for entry in ["shell", "python"]:
            for output in [original, original / "child", link, broken, self.source, self.source / "inside source", self.base]:
                with self.subTest(entry=entry, output=output):
                    before = self.snapshot()
                    result = self.invoke(entry, output)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(before, self.snapshot())
                    self.assertFalse(self.calls.exists())
                    self.assertFalse((self.base / "target").exists())

    def test_invalid_parameters_and_source_have_no_side_effects(self):
        for entry in ["shell", "python"]:
            for args in [[], [str(self.source)], [str(self.source), ""], [str(self.source), "   "], [str(self.source), str(self.base / "out"), "extra"], [str(self.base / "missing source"), str(self.base / "out")], [str(self.bin), str(self.base / "out")]]:
                with self.subTest(entry=entry, args=args):
                    before = self.snapshot()
                    result = self.invoke(entry, None, arguments=args)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(before, self.snapshot())
                    self.assertFalse(self.calls.exists())

    def test_fresh_output_runs_all_commands_through_existing_guard(self):
        original_source = self.source
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
                self.source = self.base / (entry + " fresh source")
                shutil.copytree(original_source, self.source)
                output = self.base / (entry + " fresh output")
                if self.calls.exists():
                    self.calls.unlink()
                result = self.invoke(entry, output)
                self.assertEqual(result.returncode, 0, result.stderr)
                calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
                cargo = [call for call in calls if call["command"] == "cargo"]
                self.assertGreater(len(cargo), 10)
                self.assertTrue(all(call["ready"] == "1" for call in cargo))
                self.assertTrue(all(call["target"] == str(self.base / "target") for call in cargo))
                self.assertTrue(all(call["temporary"] == str(self.base / "build/tmp") for call in cargo))
                logs = list((output / "evidence/ui-refresh").glob("*.log"))
                self.assertGreater(len(logs), 20)
                progress = sorted((output / "evidence/ui-refresh").glob("progress-*.json"))
                self.assertEqual(len(progress), len(logs))
                for index, path in enumerate(progress, 1):
                    self.assertEqual(len(json.loads(path.read_text())), index)
                before = self.snapshot()
                repeat = self.invoke(entry, output)
                self.assertNotEqual(repeat.returncode, 0)
                self.assertEqual(before, self.snapshot())

    def test_existing_empty_output_is_accepted(self):
        output = self.base / "existing empty output"
        output.mkdir()
        result = self.invoke("python", output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((output / "evidence/ui-refresh").is_dir())

    def test_explicit_python_relative_arguments_and_missing_environment(self):
        output = self.base / "relative fresh output"
        result = self.invoke("python", None, arguments=[self.source.name, output.name])
        self.assertEqual(result.returncode, 0, result.stderr)
        for env in [{"JZ35_SOURCE_ROOT": str(self.source)}, {"JZ35_REVIEW_ROOT": str(self.base / "unused")}]:
            with self.subTest(env=env):
                before = self.snapshot()
                result = self.invoke("python", None, extra_env=env, arguments=[])
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(before, self.snapshot())

    def test_log_collision_is_preserved_and_not_truncated(self):
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
                output = self.base / (entry + " collision output")
                name = "jc061.log" if entry == "shell" else "jc061-fixture.log"
                log = output / "evidence/ui-refresh" / name
                result = self.invoke(entry, output, {"STUB_LOG_COLLISION": str(log)})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(log.read_text(), "preserve colliding log\n")

    def test_storage_is_rechecked_after_first_cargo(self):
        marker = self.base / "disk filled marker"
        mock = self.base / "changing-storage.mjs"
        mock.write_text("import fs from 'node:fs'; import {syncBuiltinESMExports} from 'node:module'; const original=fs.statfsSync; fs.statfsSync=(...args)=>{const info=original(...args);return fs.existsSync(process.env.STUB_FILL_DISK)?{...info,bavail:0n}:info;}; syncBuiltinESMExports();\n")
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
                if marker.exists():
                    marker.unlink()
                if self.calls.exists():
                    self.calls.unlink()
                output = self.base / (entry + " fills during run")
                result = self.invoke(entry, output, {"STUB_FILL_DISK": str(marker), "NODE_OPTIONS": f'--import="{mock}"', "UNDERGROUNDBATTLE_BUILD_STORAGE_READY": "1"})
                self.assertNotEqual(result.returncode, 0)
                calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
                self.assertEqual(len([call for call in calls if call["command"] == "cargo"]), 1)
                logs = '\n'.join(path.read_text() for path in (output / "evidence/ui-refresh").glob("*.log"))
                self.assertIn("Build preflight failed", logs)

    def test_abi_mismatch_stops_before_native_export_or_input_changes(self):
        original = self.source / 'web/src/game/jz35Native71.fixture.json'
        original.write_text('preserved prior input\n')
        result = self.invoke('python', self.base / 'ABI mismatch output',
                             {'STUB_ABI_ENGINE':'rust-v0.2.73-jc052-original-region-candidate'})
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.calls.exists())
        self.assertEqual(original.read_text(), 'preserved prior input\n')
        self.assertTrue((self.base / 'ABI mismatch output/evidence/ui-refresh/abi-version-check.log').is_file())

    def test_unreviewed_version_is_rejected_before_output_mutation(self):
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.74-unreviewed'}))
        before = self.snapshot()
        result = self.invoke('python', self.base / 'unsupported output')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(before, self.snapshot())

    def test_native73_generation_keeps_native72_inputs(self):
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.73-jc052-original-region-candidate','cardPoolVersion':'limited-v2.66-jc052-original-region-candidate'}))
        original = self.source / 'web/src/game/xq24Native72.fixture.json'
        original.write_text('preserved Native72 input\n')
        output = self.base / 'Native73 current output'
        result = self.invoke('shell', output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(original.read_text(), 'preserved Native72 input\n')
        self.assertTrue((self.source / 'web/src/game/xq24Native73.fixture.json').is_file())
        self.assertIn('"73"', (output / 'evidence/ui-refresh/jz35-fixture.log').read_text())
        self.assertIn('jc052Native73Current.fixture.json', (output / 'evidence/ui-refresh/jc052-fixture.log').read_text())

    def test_native74_keeps_prior73_and72_inputs_and_requires_exact_pool(self):
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.74-jc037-hidden-investigation-candidate','cardPoolVersion':'limited-v2.67-jc037-hidden-investigation-candidate'}))
        original = self.source / 'web/src/game/jc052Native73Current.fixture.json'
        original.write_text('preserved Native73 input\n')
        output = self.base / 'Native74 current output'
        result = self.invoke('python', output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(original.read_text(), 'preserved Native73 input\n')
        self.assertTrue((self.source / 'web/src/game/xq24Native74.fixture.json').is_file())
        self.assertIn('jc052Native74Current.fixture.json', (output / 'evidence/ui-refresh/jc052-fixture.log').read_text())
        self.assertTrue((output / 'evidence/ui-refresh/jc037-fixture.log').is_file())
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.74-jc037-hidden-investigation-candidate','cardPoolVersion':'limited-v2.66-jc052-original-region-candidate'}))
        untouched = self.base / 'wrong74 pool'
        result = self.invoke('python', untouched)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(untouched.exists())

    def test_native75_routes_current_exports_and_preserves_all_prior_inputs(self):
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.75-jc060-occult-adviser-candidate','cardPoolVersion':'limited-v2.68-jc060-occult-adviser-candidate'}))
        originals = []
        for relative in ['web/src/game/xq24Native72.fixture.json',
                         'web/src/game/jc052Native73Current.fixture.json',
                         'web/src/game/jc037Native74.fixture.json',
                         'sites/test/fixtures/jc037-native-v074.json',
                         'sites/test/fixtures/xq24-container-invalid-v074.json']:
            path = self.source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'preserved opaque state 18446744073709551615 and expectations\n')
            originals.append((path, path.read_bytes(), path.stat().st_mtime_ns))
        output = self.base / 'Native75 current output'
        result = self.invoke('shell', output)
        self.assertEqual(result.returncode, 0, result.stderr)
        for path, data, mtime in originals:
            self.assertEqual(path.read_bytes(), data)
            self.assertEqual(path.stat().st_mtime_ns, mtime)
        evidence = output / 'evidence/ui-refresh'
        self.assertTrue((self.source / 'web/src/game/xq24Native75.fixture.json').is_file())
        self.assertEqual((self.source / 'sites/test/fixtures/xq24-container-invalid-v075.json').read_bytes(),
                         (evidence / 'xq24-container-invalid.json').read_bytes())
        self.assertIn('"75"', (evidence / 'jz35-fixture.log').read_text())
        self.assertIn('jc052Native75Current.fixture.json', (evidence / 'jc052-fixture.log').read_text())
        self.assertTrue((evidence / 'jc037-fixture.log').is_file())
        self.assertTrue((evidence / 'jc037-parity.log').is_file())
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        worker = next(call for call in calls if 'current_ui_worker_fixtures' in call['args'])
        self.assertEqual(worker['args'][-1], '75')
        self.assertNotIn('--web-only', worker['args'])
        jc052 = next(call for call in calls if 'current_jc052_worker_fixtures' in call['args'])
        self.assertEqual(jc052['args'][-2:], [str(self.source / 'web/src/game/jc052Native75Current.fixture.json'),
                                           str(self.source / 'sites/test/fixtures/jc052-native-v075.json')])
        self.assertTrue(any('xq24_entry_frame_container_restore_rejects_response_bypass' in call['args'] for call in calls))

    def test_native75_requires_exact_source_and_built_identity(self):
        engine = 'rust-v0.2.75-jc060-occult-adviser-candidate'
        expected_pool = 'limited-v2.68-jc060-occult-adviser-candidate'
        prior_pool = 'limited-v2.67-jc037-hidden-investigation-candidate'
        self.pool.write_text(json.dumps({'engineVersion':engine,'cardPoolVersion':prior_pool}))
        before = self.snapshot()
        result = self.invoke('python', self.base / 'wrong75 source pool')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(before, self.snapshot())
        self.assertFalse(self.calls.exists())
        self.pool.write_text(json.dumps({'engineVersion':engine,'cardPoolVersion':expected_pool}))
        for key, value in [('STUB_ABI_ENGINE', 'rust-v0.2.74-jc037-hidden-investigation-candidate'),
                           ('STUB_ABI_POOL', prior_pool)]:
            with self.subTest(key=key):
                result = self.invoke('python', self.base / key, {key:value})
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.calls.exists())
                self.assertFalse((self.source / 'web/src/game/xq24Native75.fixture.json').exists())

    def test_recorded_current_input_targets_reject_before_any_mutation(self):
        self.pool.write_text(json.dumps({'engineVersion':'rust-v0.2.75-jc060-occult-adviser-candidate','cardPoolVersion':'limited-v2.68-jc060-occult-adviser-candidate'}))
        for relative in ['web/src/game/currentLifecycle75.fixture.json',
                         'web/src/game/jc050Native75QuoteTest.fixture.json',
                         'sites/test/fixtures/xq24-container-invalid-v075.json',
                         'sites/test/fixtures/jc037-native-v075.json']:
            with self.subTest(relative=relative):
                original = self.source / relative
                original.parent.mkdir(parents=True, exist_ok=True)
                original.write_bytes(b'preserve already recorded current input\n')
                before = self.snapshot()
                result = self.invoke('python', self.base / 'new evidence output')
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('Native input already exists', result.stderr)
                self.assertEqual(before, self.snapshot())
                self.assertFalse(self.calls.exists())
                original.unlink()


if __name__ == "__main__":
    unittest.main()
