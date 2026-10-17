"""One normal-TLS check of the exact published v65. Never bypass a warning."""
import asyncio, datetime, hashlib, json, subprocess
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

OUT = Path(__file__).resolve().parent
REPO = Path('/workspace/UndergroundBattle')
URL = 'https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site'
IDENTITY = json.loads((OUT / 'official-identity.json').read_text())
assert IDENTITY['officialDeployment']['status'] == 'succeeded'
assert IDENTITY['officialVersion']['version_number'] == 65
assert IDENTITY['officialVersion']['source']['commit_sha'] == '6efc6273a4788e3ed4774d1a07b0921ff6d9d9a0'
manifest_path = 'docs/evidence/msjc0405-2026-10-17/runtime-manifest.json'
manifest_bytes = (REPO / manifest_path).read_bytes()
assert manifest_bytes == subprocess.check_output(['git', 'show', IDENTITY['expectedEvidenceCommit'] + ':' + manifest_path], cwd=REPO)
manifest = json.loads(manifest_bytes)
ASSETS = {'/' + name[len('client/'):]: row['sha256'] for row in manifest['files'] if (name := row['path']).startswith('client/assets/')}
ASSETS['/'] = next(row['sha256'] for row in manifest['files'] if row['path']=='client/index.html')
EXPECTED_VERSIONS = {
    'rulesVersion': 'hegemony-pdf-v1',
    'cardPoolVersion': 'limited-v2.75-msjc04-msjc05-societies-candidate',
    'engineVersion': 'rust-v0.2.83-msjc04-msjc05-societies-candidate',
}
def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

async def main():
    wait_started = now()
    previous_probe = datetime.datetime.fromisoformat('2026-10-17T15:26:24.323582+00:00')
    minimum_navigation = previous_probe + datetime.timedelta(hours=1)
    while (remaining := (minimum_navigation - datetime.datetime.now(datetime.timezone.utc)).total_seconds()) > 0:
        await asyncio.sleep(min(remaining, 30))
    result = {
        'startedAtUtc': now(), 'url': URL, 'scope': 'Real normal cloud Chromium against published production URL; reachability/identity gate before isolated gameplay',
        'expectedDeployedVersion': 65, 'expectedProviderSource': IDENTITY['officialVersion']['source']['commit_sha'],
        'previousProbeAtUtc': previous_probe.isoformat(), 'minimumNavigationAtUtc': minimum_navigation.isoformat(), 'waitStartedAtUtc': wait_started,
        'expectedDeploymentId': IDENTITY['officialDeployment']['id'],
        'officialIdentityVerifiedBeforeBrowser': True, 'normalTls': True, 'ignoreHttpsErrors': False,
        'bypassUsed': False, 'certificateWarningClicked': False, 'certificateOverrideFlags': [],
        'customTrustStoreConfigured': False, 'freshBrowserContext': True, 'oldInviteUsed': False,
        'realUserTableAccessed': False, 'roomCreated': False, 'roomJoined': False,
        'posts': 0, 'navigationAttempts': 0, 'uiReached': False, 'versionIdentityReached': False,
        'fullGameCount': 0, 'onlineGameplay': False, 'componentTestUsed': False, 'apiMockingUsed': False,
        'neutralAndDualPaused': True,
        'opaqueStateInjected': False, 'compiled': False, 'deployed': False, 'mainMerged': False,
        'browserErrors': [], 'requests': [], 'requestsFailed': [], 'assetIdentityChecks': [], 'consoleErrors': [],
        'expectedBrowserAssets': ASSETS, 'runtimeManifestSha256': hashlib.sha256(manifest_bytes).hexdigest(),
        'gameplayChecks': {name: 'not-executed' for name in [
            'red-admitted-mechanisms', 'black-admitted-mechanisms', 'blue-admitted-mechanisms', 'gray-admitted-mechanisms', 'msjc04-paid-initiative-draw-and-red-unique-search', 'msjc05-paid-initiative-draw-and-gray-empty-search', 'initial-society-and-legal-deck-selection',
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
                            societies:value.societies?.length, hasMSJC04:value.societies?.some(card=>card.id==='MSJC04'), hasMSJC05:value.societies?.some(card=>card.id==='MSJC05')};
                }""")
                result['browserCatalogIdentity'] = catalog
                result['versionIdentityReached'] = catalog['status'] == 200 and all(catalog.get(key) == value for key, value in EXPECTED_VERSIONS.items()) and catalog['cards'] == 147 and catalog['societies'] == 10 and catalog['hasMSJC04'] and catalog['hasMSJC05']
                result['outcome'] = 'ready-for-isolated-gameplay' if result['versionIdentityReached'] else 'blocked-browser-catalog-identity'
            await page.screenshot(path=str(OUT / 'normal-browser.png'), full_page=True)
        except Exception as error:
            result['errorAtUtc'] = now()
            result['error'] = type(error).__name__ + ': ' + str(error)
            result['outcome'] = 'blocked-tls-before-gameplay' if 'net::ERR_CERT_AUTHORITY_INVALID' in str(error) else 'blocked-before-gameplay'
            result['sameCertificateErrorAsPriorV65'] = 'net::ERR_CERT_AUTHORITY_INVALID' in str(error)
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
    result['gameWriteRequests'] = [r for r in result['requests'] if r['method'] not in ['GET', 'HEAD', 'OPTIONS']]
    result['zeroGameWriteRequests'] = len(result['gameWriteRequests']) == 0
    result['probeReason'] = 'Parent requested a new routine real-browser check of the same reviewed and successfully deployed v65; normal navigation starts at least one hour after the previous v65 check.'
    result['getRequests'] = sum(request['method'] == 'GET' for request in result['requests'])
    (OUT / 'normal-browser.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({key: result.get(key) for key in [
        'outcome', 'navigationAtUtc', 'browserVersion', 'navigationAttempts', 'getRequests', 'posts',
        'uiReached', 'versionIdentityReached', 'sameCertificateErrorAsPriorV65', 'error'
    ]}, ensure_ascii=False))

asyncio.run(main())
