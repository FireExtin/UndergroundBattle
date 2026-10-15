"""Read saved Node CPU profiles and cgroup samples; never execute tests."""
from collections import Counter
from hashlib import sha256
from pathlib import Path
import json
import gzip
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).parent

def kv(value):
    return {key: int(number) for key, number in (row.split() for row in value.splitlines())}

def summarize_profile(path):
    raw = gzip.decompress(path.read_bytes()) if path.suffix == '.gz' else path.read_bytes()
    data = json.loads(raw)
    nodes = {node['id']: node for node in data['nodes']}
    parents = {child: node['id'] for node in data['nodes'] for child in node.get('children', [])}
    categories, leaves, cache = Counter(), Counter(), {}
    def stack(node):
        if node not in cache:
            cache[node] = [nodes[node]['callFrame']] + (stack(parents[node]) if node in parents else [])
        return cache[node]
    for node, delta in zip(data.get('samples', []), data.get('timeDeltas', [])):
        frames = stack(node)
        frame = nodes[node]['callFrame']
        leaves[(frame['functionName'], frame['url'], frame['lineNumber'] + 1)] += delta
        predicates = {
            'testCallback': lambda f: f['url'].endswith('.test.jsx'),
            'roleQuery': lambda f: '/@testing-library/dom/dist/queries/role.js' in f['url'],
            'accessibleName': lambda f: '/dom-accessibility-api/' in f['url'],
            'react': lambda f: '/react/' in f['url'] or '/react-dom/' in f['url'],
            'vitestDeepEquality': lambda f: '/@vitest/expect/' in f['url'] and f['functionName'] in ['eq', 'equals', 'toEqual', 'keys', 'iterableEquality', 'iterableEqualityWithStack'],
            'wasm': lambda f: f['url'].startswith('wasm:'),
        }
        for category, predicate in predicates.items():
            if any(predicate(frame) for frame in frames):
                categories[category] += delta
        if frame['functionName'] == '(garbage collector)':
            categories['garbageCollector'] += delta
    return {
        'path': str(path.relative_to(root)).removesuffix('.gz'), 'bytes': len(raw),
        'sha256': sha256(raw).hexdigest(),
        'testFiles': sorted({Path(n['callFrame']['url']).name for n in data['nodes'] if n['callFrame']['url'].endswith('.test.jsx')}),
        'sampleSeconds': round(sum(data['timeDeltas']) / 1e6, 6),
        'inclusiveSeconds': {key: round(value / 1e6, 6) for key, value in categories.items()},
        'topSelfFrames': [{'function': fn, 'url': url, 'line': line, 'seconds': round(value / 1e6, 6)} for (fn, url, line), value in leaves.most_common(12)],
    }

summary = {'method': 'Node statistical CPU samples; inclusive categories overlap and must not be summed; profiling changes timing', 'runs': {}}
for name in ['baseline', 'fixed']:
    rows = [json.loads(line) for line in (root / f'{name}-resources.jsonl').read_text().splitlines()]
    first, last = kv(rows[0]['cpu.stat']), kv(rows[-1]['cpu.stat'])
    report = json.loads((root / f'{name}-vitest.json').read_text())
    summary['runs'][name] = {
        'passed': report['numPassedTests'], 'failed': report['numFailedTests'],
        'tests': [{'file': Path(item['name']).name, 'firstTest': item['assertionResults'][0]['title'], 'durationMs': item['assertionResults'][0]['duration'], 'status': item['assertionResults'][0]['status']} for item in report['testResults']],
        'resourceObservationSeconds': round(rows[-1]['monotonic'] - rows[0]['monotonic'], 6),
        'cpuQuota': rows[0]['cpu.max'], 'cpuStatDelta': {key: last[key] - first[key] for key in first},
        'memoryLimitBytes': int(rows[0]['memory.max']), 'memoryPeakSampledBytes': max(int(row['memory.current']) for row in rows),
        'memoryEventDelta': {key: kv(rows[-1]['memory.events'])[key] - value for key, value in kv(rows[0]['memory.events']).items()},
        'profiles': [summarize_profile(path) for path in (sorted((root / f'profiles-{name}').glob('*.cpuprofile')) or sorted((root / f'profiles-{name}').glob('*.cpuprofile.gz')))],
    }
(root / 'profile-summary.json').write_text(json.dumps(summary, indent=2, ensure_ascii=False) + '\n')
for name, run in summary['runs'].items():
    print(name, json.dumps({'passed': run['passed'], 'failed': run['failed'], 'tests': run['tests'], 'profiles': [{'test': p['testFiles'], 'inclusiveSeconds': p['inclusiveSeconds']} for p in run['profiles']]}, ensure_ascii=False))
