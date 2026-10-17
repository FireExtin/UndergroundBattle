"""One normal-TLS check of the exact published v64. Never bypass a warning."""
import asyncio, datetime, hashlib, json, subprocess
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

OUT = Path(__file__).resolve().parent
REPO = Path('/workspace/UndergroundBattle')
URL = 'https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site'
IDENTITY = json.loads((OUT / 'official-identity.json').read_text())
assert IDENTITY['officialDeployment']['status'] == 'succeeded'
assert IDENTITY['officialVersion']['version_number'] == 64
assert IDENTITY['officialVersion']['source']['commit_sha'] == '74b1e8f2e6a0c4feb403520554a9275f52be82fd'
manifest_path = 'docs/evidence/response-preview-2026-10-17/runtime-manifest.json'
manifest_bytes = (REPO / manifest_path).read_bytes()
assert manifest_bytes == subprocess.check_output(['git', 'show', IDENTITY['expectedEvidenceCommit'] + ':' + manifest_path], cwd=REPO)
manifest = json.loads(manifest_bytes)
ASSETS = {'/' + name[len('client/'):]: row['sha256'] for row in manifest['files'] if (name := row['path']).startswith('client/assets/')}
ASSETS['/'] = next(row['sha256'] for row in manifest['files'] if row['path']=='client/index.html')
EXPECTED_VERSIONS = {
    'rulesVersion': 'hegemony-pdf-v1',
    'cardPoolVersion': 'limited-v2.74-jc054-complete-conceal-candidate',
    'engineVersion': 'rust-v0.2.82-response-source-preview-candidate',
}
def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

async def main():
    result = {
        'startedAtUtc': now(), 'url': URL, 'scope': 'Real normal cloud Chromium against published production URL; reachability/identity gate before isolated gameplay',
        'expectedDeployedVersion': 64, 'expectedProviderSource': IDENTITY['officialVersion']['source']['commit_sha'],
        'expectedDeploymentId': IDENTITY['officialDeployment']['id'],
        'officialIdentityVerifiedBeforeBrowser': True, 'normalTls': True, 'ignoreHttpsErrors': False,
        'bypassUsed': False, 'certificateWarningClicked': False, 'certificateOverrideFlags': [],
        'customTrustStoreConfigured': False, 'freshBrowserContext': True, 'oldInviteUsed': False,
        'realUserTableAccessed': False, 'roomCreated': False, 'roomJoined': False,
        'posts': 0, 'navigationAttempts': 0, 'uiReached': False, 'versionIdentityReached': False,
        'fullGameCount': 0, 'onlineGameplay': False, 'componentTestUsed': False, 'apiMockingUsed': False,
        'opaqueStateInjected': False, 'compiled': False, 'deployed': False, 'mainMerged': False,
        'browserErrors': [], 'requests': [], 'requestsFailed': [], 'assetIdentityChecks': [], 'consoleErrors': [],
        'expectedBrowserAssets': ASSETS, 'runtimeManifestSha256': hashlib.sha256(manifest_bytes).hexdigest(),
        'gameplayChecks': {name: 'not-executed' for name in [
            'jc054-complete-mandatory-conceal', 'current-window-and-server-candidate-source-hints',
            'card-selection', 'repeat-click-single-submission', 'refresh-recovery', 'complete-game'
        ]}
    }
    pending = []
    async with async_playwright() as p:
        launch_args = ['--no-sandbox', '--disable-dev-shm-usage']
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', headless=True, args=launch_args)
        result['browserVersion'] = browser.version
        result['browserExecutable'] = '/usr/bin/chromium'
        result['browserLaunchArgs'] = launch_args
        context = await browser.new_context(viewport={'width': 1366, 'height': 900}, locale='zh-CN', ignore_https_errors=False)
        page = await context.new_page()
        def requested(request):
            if urlsplit(request.url).hostname != urlsplit(URL).hostname:
                return
            result['requests'].append({'url': request.url, 'method': request.method})
            if request.method == 'POST':
                result['posts'] += 1
        def failed(request):
            result['requestsFailed'].append({'url': request.url, 'method': request.method, 'error': request.failure})
        async def response(reply):
            path = urlsplit(reply.url).path
            if path not in ASSETS or urlsplit(reply.url).hostname != urlsplit(URL).hostname:
                return
            try:
                body = await reply.body()
                digest = hashlib.sha256(body).hexdigest()
                result['assetIdentityChecks'].append({'path': path, 'status': reply.status, 'sha256': digest, 'expectedSha256': ASSETS[path], 'matches': digest == ASSETS[path]})
            except Exception as error:
                result['browserErrors'].append('asset-read: ' + str(error))
        page.on('request', requested)
        page.on('requestfailed', failed)
        page.on('response', lambda reply: pending.append(asyncio.create_task(response(reply))))
        page.on('pageerror', lambda error: result['browserErrors'].append(str(error)))
        page.on('console', lambda msg: result['consoleErrors'].append(msg.text) if msg.type == 'error' else None)
        try:
            result['navigationAtUtc'] = now()
            result['navigationAttempts'] = 1
            reply = await page.goto(URL, wait_until='domcontentloaded', timeout=45000)
            result['navigationStatus'] = reply.status if reply else None
            await page.locator('.hg-app').wait_for(state='visible', timeout=15000)
            result['uiReached'] = True
            result['title'] = await page.title()
            if pending:
                await asyncio.gather(*pending, return_exceptions=True)
            result['browserAssetsMatch'] = set(row['path'] for row in result['assetIdentityChecks'] if row['matches']) == set(ASSETS)
            if not result['browserAssetsMatch']:
                result['outcome'] = 'blocked-browser-asset-identity'
            else:
                catalog = await page.evaluate("""async () => {
                    const reply = await fetch('/api/catalog', {cache:'no-store'});
                    const value = await reply.json();
                    return {status:reply.status, rulesVersion:value.rulesVersion, cardPoolVersion:value.cardPoolVersion,
                            engineVersion:value.engineVersion, cards:value.cards?.length,
                            hasJC054:value.cards?.some(card=>card.id==='JC054'), hasWM062:value.cards?.some(card=>card.id==='WM062'), hasJZ48:value.cards?.some(card=>card.id==='JZ48')};
                }""")
                result['browserCatalogIdentity'] = catalog
                result['versionIdentityReached'] = catalog['status'] == 200 and all(catalog.get(key) == value for key, value in EXPECTED_VERSIONS.items()) and catalog['cards'] == 147 and catalog['hasJC054'] and catalog['hasWM062'] and catalog['hasJZ48']
                result['outcome'] = 'ready-for-isolated-gameplay' if result['versionIdentityReached'] else 'blocked-browser-catalog-identity'
            await page.screenshot(path=str(OUT / 'normal-browser.png'), full_page=True)
        except Exception as error:
            result['errorAtUtc'] = now()
            result['error'] = type(error).__name__ + ': ' + str(error)
            result['outcome'] = 'blocked-tls-before-gameplay' if 'net::ERR_CERT_AUTHORITY_INVALID' in str(error) else 'blocked-before-gameplay'
            result['sameCertificateBlockerAsPreviousV61'] = 'net::ERR_CERT_AUTHORITY_INVALID' in str(error)
            try:
                result['pageUrlAfterFailure'] = page.url
                result['titleAfterFailure'] = await page.title()
                await page.screenshot(path=str(OUT / 'normal-browser.png'), full_page=True, timeout=10000)
            except Exception as capture:
                result['screenshotError'] = str(capture)
        finally:
            if pending:
                await asyncio.gather(*pending, return_exceptions=True)
            await context.close()
            await browser.close()
    result['finishedAtUtc'] = now()
    result['getRequests'] = sum(request['method'] == 'GET' for request in result['requests'])
    (OUT / 'normal-browser.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({key: result.get(key) for key in [
        'outcome', 'navigationAtUtc', 'browserVersion', 'navigationAttempts', 'getRequests', 'posts',
        'uiReached', 'versionIdentityReached', 'sameCertificateBlockerAsPreviousV61', 'error'
    ]}, ensure_ascii=False))

asyncio.run(main())
