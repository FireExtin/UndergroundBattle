from pathlib import Path
from playwright.sync_api import sync_playwright,expect
import json,hashlib
truth=json.loads(Path('/tmp/undergroundbattle-shared-cloud/web/src/game/jz50Native48Test.fixture.json').read_text())
view=truth['private']['views'][0];catalog=truth['private']['catalog']
out=Path('/tmp/search-family-evidence/browser48-final');out.mkdir(exist_ok=True)
result={'scope':'local-real-React-readonly-actual-Native48-JZ50-desktop-projections','productionWrites':False,'APIWrites':[],'errors':[],'checks':[],'passed':False}
session={'roomId':view['roomId'],'inviteCode':view['inviteCode'],'token':'read-only-local-fixture-token','seat':0}
with sync_playwright() as p:
 b=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--no-zygote','--single-process','--disable-gpu'])
 context=b.new_context(viewport={'width':1440,'height':1000})
 context.add_init_script('localStorage.setItem("hegemony.session.v1",'+json.dumps(json.dumps(session))+');')
 page=context.new_page();page.on('pageerror',lambda e:result['errors'].append(str(e)))
 def api(route):
  request=route.request
  if request.method!='GET':
   result['APIWrites'].append(request.method+' '+request.url.split('?')[0]);route.fulfill(status=410,json={'error':'read_only_fixture'});return
  if request.url.split('?')[0].endswith('/catalog'):route.fulfill(json=catalog);return
  if '/state' in request.url:route.fulfill(json=view);return
  route.fulfill(status=404,json={'error':'unimplemented_readonly_fixture'})
 page.route('**/api/**',api)
 try:
  page.goto('http://127.0.0.1:5193',wait_until='networkidle')
  modal=page.get_by_role('dialog',name='待完成的选择');expect(modal).to_be_visible()
  expect(modal.get_by_role('button',name='不取牌并洗牌',exact=True)).to_be_enabled()
  expect(modal.get_by_role('button',name='跳过此选择',exact=True)).to_have_count(0)
  ids=[o['id'] for o in view['pendingChoice']['options']];assert len(ids)==2
  expect(modal.locator('[data-choice-option]')).to_have_count(2)
  page.screenshot(path=str(out/'desktop-private-zero-with-candidates.png'))
  result['checks'].append('controller-private-actual-candidates-allow-zero-completion-without-second-decline')
  modal.locator('[data-choice-option="'+ids[0]+'"]').click()
  expect(modal.get_by_role('button',name='确认选择',exact=True)).to_be_enabled()
  expect(modal.locator('[data-choice-option="'+ids[1]+'"]')).to_be_disabled()
  modal.locator('[data-choice-option="'+ids[0]+'"]').click()
  expect(modal.get_by_role('button',name='不取牌并洗牌',exact=True)).to_be_enabled()
  page.reload(wait_until='networkidle')
  expect(page.get_by_role('dialog',name='待完成的选择').locator('[data-choice-option]')).to_have_count(2)
  result['checks'].append('desktop-refresh-retains-original-seat-and-identical-private-choice')
  view=truth['empty']['views'][0]
  expect(page.get_by_role('dialog',name='待完成的选择').locator('[data-choice-option]')).to_have_count(0,timeout=7000)
  expect(page.get_by_role('button',name='不取牌并洗牌',exact=True)).to_be_enabled()
  page.screenshot(path=str(out/'desktop-empty-private-confirmation.png'))
  result['checks'].append('actual-empty-library-hit-view-still-offers-private-zero-and-shuffle-confirmation')
  view=truth['selected']['views'][0]
  expect(page.get_by_role('dialog',name='待完成的选择')).to_have_count(0,timeout=7000)
  page.reload(wait_until='networkidle')
  expect(page.get_by_role('dialog',name='待完成的选择')).to_have_count(0)
  page.screenshot(path=str(out/'desktop-completed-owner-graveyard-refresh.png'))
  result['checks'].append('actual-completed-Native-owner-graveyard-projection-closes-private-modal-and-survives-refresh')
  assert not result['errors'] and not result['APIWrites'];result['passed']=True
 except Exception as e:result['error']=str(e)[:1800]
 finally:
  b.close();result['screenshots']={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.glob('*.png')};(out/'results.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False))
if not result['passed']:raise SystemExit(1)
