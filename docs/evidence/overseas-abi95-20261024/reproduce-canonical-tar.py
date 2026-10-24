"""Reproduce the exact stored v1 tar representation from the frozen upload."""
import argparse
import hashlib
import io
import json
import pathlib
import tarfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--workspace-root', required=True, type=pathlib.Path,
                    help='Preserved task directory containing artifacts/overseas-source-batches/batch-13.tar.gz')
root = parser.parse_args().workspace_root.resolve()
original = root / 'artifacts/overseas-source-batches/batch-13.tar.gz'
destination = root / 'artifacts/overseas-v1-canonical-reproduced-20261024.tar'
expected = 'ea460e2ef5889be3f6f7a6012baed0eb08cccefd24bdb6adf12e2e2aa2a7558f'
with tarfile.open(original) as archive:
    inputs = [(entry, archive.extractfile(entry).read()) for entry in archive.getmembers() if entry.isfile()]
assert len(inputs) == 270
assert not destination.exists(), 'Preserve any prior reproduction instead of overwriting it.'
with tarfile.open(destination, 'w', format=tarfile.USTAR_FORMAT) as output:
    for entry, data in sorted(inputs, key=lambda item: item[0].name):
        header = tarfile.TarInfo(entry.name)
        header.size = len(data)
        header.mode = 0o644
        header.mtime = header.uid = header.gid = 0
        header.uname = header.gname = ''
        output.addfile(header, io.BytesIO(data))
digest = hashlib.sha256(destination.read_bytes()).hexdigest()
assert destination.stat().st_size == 194508800
assert digest == expected
result = {
    'product_release': '76b1b189c42c3da47ffc875a463ada162c1e8fbd',
    'site_source_sha': 'e387d3bb0e045799348c3e0580ac95ad4a94cee7',
    'version_id': 'appgprj_6adc02e46820819188f42c2474995574~appgver_b9df601c0ae88191a1b73eb4acf8a048',
    'platform_archive_sha256': expected,
    'reconstructed_archive_sha256': digest,
    'reconstructed_archive_bytes': destination.stat().st_size,
    'file_count': len(inputs),
    'recipe': 'file-only USTAR; retain dist/ paths; sort paths; mode644; uid/gid/mtime=0; empty uname/gname',
    'same_file_payloads_as_frozen_upload': True,
    'stored_tar_identity_reproduced': True,
    'inference_limit': 'Exact representation for this saved v1 is reproduced; not a claim about all future platform packaging implementations.',
    'prior_execution_error_preserved': 'The prior raw-tar comparison failed and did not gate deployment. It remains historical evidence; no version was republished.',
    'health_endpoint': 'Prior403 is not retried or used for this proof.'
}
(root / 'OVERSEAS_V1_TAR_IDENTITY_CLOSED_20261024.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
