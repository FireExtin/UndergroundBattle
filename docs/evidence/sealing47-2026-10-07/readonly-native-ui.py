from pathlib import Path
from playwright.sync_api import sync_playwright,expect
import json,hashlib
truth=json.loads(Path('/tmp/undergroundbattle-shared-cloud/web/src/game/sealedNative47Test.fixture.json').read_text())
view=truth['cases']['sealed-trigger']['views'][2]
out=Path('/tmp/sealing-family-evidence/browser47-final');out.mkdir(exist_ok=True)
result={'scope':'local-real-React-readonly-actual-Native47-desktop-projections','productionWrites':False,'APIWrites':[], 'errors':[], 'checks':[], 'passed':False}
session={'roomId':view['roomId'],'inviteCode':view['inviteCode'],'token':'read-only-local-fixture-token','seat':2}
with sync_playwright() as p:
 b=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--no-zygote','--single-process','--disable-gpu'])
 context=b.new_context(viewport={'width':1440,'height':1000})
 context.add_init_script('localStorage.setItem("hegemony.session.v1",'+json.dumps(json.dumps(session))+');')
 page=context.new_page();page.on('pageerror',lambda e:result['errors'].append(str(e)))
 def api(route):
  request=route.request
  if request.method!='GET':
   result['APIWrites'].append(request.method+' '+request.url.split('?')[0]);route.fulfill(status=410,json={'error':'read_only_fixture'});return
  if request.url.split('?')[0].endswith('/catalog'):route.fulfill(json=truth['catalog']);return
  if '/state' in request.url:route.fulfill(json=view);return
  route.fulfill(status=404,json={'error':'unimplemented_readonly_fixture'})
 page.route('**/api/**',api)
 try:
  page.goto('http://127.0.0.1:5193',wait_until='networkidle')
  page.get_by_text('封印牌 · 1',exact=True).click()
  expect(page.get_by_role('region',name='场外封印牌')).to_be_visible()
  page.screenshot(path=str(out/'desktop-public-sealed-menu.png'))
  page.get_by_role('button',name='阅读封印牌启迪之梦',exact=True).click()
  modal=page.get_by_role('dialog',name='放大阅读启迪之梦')
  expect(modal.get_by_text('已封印 · 场外空白牌',exact=True)).to_be_visible()
  assert modal.locator('.hg-cost,.hg-icons').count()==0
  page.wait_for_function("()=>Array.from(document.querySelectorAll('.hg-reading-card img')).every(img=>img.complete&&img.naturalWidth>0)")
  page.screenshot(path=str(out/'desktop-sealed-blank-reader.png'))
  result['checks'].append('public-exact-owner-carrier-reader-has-no-current-printed-stats')
  modal.get_by_role('button',name='原始牌面',exact=True).click()
  expect(modal.get_by_role('img',name='启迪之梦原始牌面')).to_be_visible()
  page.wait_for_function("()=>Array.from(document.querySelectorAll('.hg-source-reading img')).every(img=>img.complete&&img.naturalWidth>0)")
  expect(modal.get_by_text('原图中的印刷能力和数值在封印期间不生效。',exact=True)).to_be_visible()
  page.screenshot(path=str(out/'desktop-original-face-inactive.png'))
  result['checks'].append('original-face-is-explicitly-inactive-during-sealing')
  view=truth['cases']['owner-return']['views'][2]
  expect(page.get_by_role('dialog',name='放大阅读启迪之梦')).to_have_count(0,timeout=6000)
  expect(page.get_by_text('封印牌 · 1',exact=True)).to_have_count(0)
  result['checks'].append('latest-Native-return-projection-closes-stale-reader-and-menu')
  page.reload(wait_until='networkidle')
  expect(page.get_by_text('封印牌 · 1',exact=True)).to_have_count(0)
  expect(page.get_by_role('dialog',name='放大阅读启迪之梦')).to_have_count(0)
  page.screenshot(path=str(out/'desktop-refreshed-owner-return.png'))
  result['checks'].append('desktop-refresh-retains-original-seat-and-latest-owner-return-view')
  expect(page.get_by_text('unimplemented_readonly_fixture',exact=True)).to_have_count(0)
  assert not result['errors'] and not result['APIWrites'];result['passed']=True
 except Exception as e:result['error']=str(e)[:1800]
 finally:
  b.close();result['screenshots']={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.glob('*.png')};(out/'results.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False))
if not result['passed']:raise SystemExit(1)
