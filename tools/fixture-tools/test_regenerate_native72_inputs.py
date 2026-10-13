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
        for name in ["jc061", "jc087", "jc094", "wm056", "wm023", "xq25", "wm038", "jz35"]:
            (tools / f"{name}-current.py").write_text("print('fixture stub')\n")
        for name in ["current-four-faction-choices.py", "current-win-flow.py"]:
            (tools / name).write_text("print('fixture stub')\n")
        (tools / "current-bq030.mjs").write_text("console.log('fixture stub');\n")
        wasm_tests = self.source / "rust-game-wasm/tests"
        wasm_tests.mkdir()
        for name in ["jc061", "jc087", "jc094", "wm056", "wm023", "xq25", "wm038", "jz35"]:
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
    for n in range(12): (directory/f'case-{{n:02}}.json').write_text(json.dumps({{'catalog':{{}},'name':str(n)}}))
if Path(sys.argv[0]).name=='wasm-bindgen':
    Path(sys.argv[sys.argv.index('--out-dir')+1]).mkdir(parents=True,exist_ok=True)
""".format(python=sys.executable)
        for name in ["cargo", "wasm-bindgen", "npm"]:
            path = self.bin / name
            path.write_text(stub)
            path.chmod(0o755)
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
        for entry in ["shell", "python"]:
            with self.subTest(entry=entry):
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


if __name__ == "__main__":
    unittest.main()
