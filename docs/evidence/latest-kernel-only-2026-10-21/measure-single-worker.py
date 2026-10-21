from pathlib import Path
import json, subprocess, sys, time

root = Path('/workspace/audit-underground-100161b')
repo, worker, label = sys.argv[1:]
persistence = root / 'single-kernel-measurements' / 'synthetic-databases'
persistence.mkdir(parents=True, exist_ok=True)
results = []
for iteration in range(3):
    log = root / 'single-kernel-measurements' / f'{label}-{iteration}.log'
    with log.open('w') as output:
        p = subprocess.Popen(['node', str(root/'measure-single-worker.mjs'), repo, worker, str(persistence), f'{label}-{iteration}'], stdout=output, stderr=subprocess.STDOUT)
        peak = 0
        while p.poll() is None:
            processes = {}
            for status in Path('/proc').glob('[0-9]*/status'):
                try:
                    data = dict(line.split(':', 1) for line in status.read_text().splitlines() if ':' in line)
                    processes[int(status.parent.name)] = (int(data['PPid'].strip()), int(data.get('VmRSS', '0 kB').split()[0])*1024)
                except (OSError, KeyError, ValueError):
                    continue
            selected = {p.pid}
            while True:
                expanded = selected | {pid for pid, (ppid, _) in processes.items() if ppid in selected}
                if expanded == selected: break
                selected = expanded
            peak = max(peak, sum(processes.get(pid, (0, 0))[1] for pid in selected))
            time.sleep(0.02)
    if p.returncode:
        raise SystemExit(f'Measurement failed: exit={p.returncode}; preserve {log}')
    row = next(json.loads(line) for line in reversed(log.read_text().splitlines()) if line.startswith('{'))
    row.update(iteration=iteration, processTreePeakRssBytes=peak, log=str(log), samplingIntervalMs=20)
    results.append(row)
    print(json.dumps(row), flush=True)
(root/'single-kernel-measurements'/f'{label}.json').write_text(json.dumps({'scope': 'Three sequential local Miniflare/workerd starts with identical health/create/read workload. Process-tree RSS sums may double-count shared pages; not production memory or guaranteed latency.', 'results': results}, indent=2)+'\n')
