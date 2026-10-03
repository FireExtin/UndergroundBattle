import json,subprocess,urllib.request,time,hashlib
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
root=Path('/workspace/.private-validation/v027-choice-modal-20261003')
server=subprocess.Popen(['python','-m','http.server','8126','--bind','127.0.0.1','--directory',str(root/'dist')],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
def focus(page):
 return page.evaluate('()=>({tag:document.activeElement?.tagName,label:document.activeElement?.getAttribute("aria-label"),inChoice:!!document.activeElement?.closest(".hg-choice"),inReader:!!document.activeElement?.closest(".hg-reading"),inGuide:!!document.activeElement?.closest(".hg-help"),inert:!!document.activeElement?.closest("[inert]")})')
def ax(page):
 cdp=page.context.new_cdp_session(page)
 try:return [{'role':n.get('role',{}).get('value'),'name':n.get('name',{}).get('value'),'ignored':n.get('ignored')} for n in cdp.send('Accessibility.getFullAXTree')['nodes'] if n.get('role',{}).get('value') in ('dialog','switch') and not n.get('ignored')]
 finally:cdp.detach()
def selected(page):return page.locator('[data-choice-option][aria-pressed="true"]').evaluate_all('es=>es.map(e=>e.getAttribute("data-choice-option"))')
def submissions(page):return json.loads(page.locator('#submissions').inner_text())
try:
 for _ in range(50):
  try:urllib.request.urlopen('http://127.0.0.1:8126/',timeout=1).close();break
  except OSError:time.sleep(.1)
 results=[]
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True)
  try:
   for width,height in [(1536,960),(1366,768),(1280,800)]:
    page=browser.new_page(viewport={'width':width,'height':height})
    errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
    page.goto('http://127.0.0.1:8126/',wait_until='networkidle')
    switch=page.get_by_role('switch',name='无可用行动时自动让过')
    page.get_by_role('button',name='打开本地选择').click();switch.click()
    dialog=page.get_by_role('dialog',name='待完成的选择');expect(dialog).to_be_visible()
    assert focus(page)['inChoice'];assert page.get_by_role('switch').count()==0
    tree=ax(page);assert not any(n['role']=='switch' for n in tree)
    keys=[]
    for key in ['Tab']*30+['Shift+Tab']*12:
     page.keyboard.press(key);f=focus(page);keys.append(f);assert f['inChoice'] and not f['inert']
    page.keyboard.press('Escape');expect(dialog).to_be_visible();assert not selected(page) and not submissions(page)
    page.screenshot(path=str(root/f'choice-{width}x{height}.png'),full_page=True)
    dialog.get_by_role('button',name='保留全部手牌').click();expect(dialog).to_have_count(0)
    expect(switch).to_be_focused();expect(switch).to_be_checked();assert submissions(page)==[{'id':'local-choose','kind':'choose','choiceId':'local-mulligan','label':'选择','selected':[]}]
    page.get_by_role('button',name='打开本地选择').click();expect(dialog).to_be_visible()
    option=dialog.locator('[data-choice-option]').first;option.click();assert selected(page)==['synthetic-hand-0']
    opener=dialog.get_by_role('button',name='放大阅读无知路人').first;opener.click()
    reader=page.get_by_role('dialog',name='放大阅读无知路人');expect(reader).to_be_visible();assert page.get_by_role('dialog',name='待完成的选择').count()==0
    assert not reader.evaluate('e=>!!e.closest("[inert]")')
    reader.get_by_role('button',name='原始牌面').click();img=reader.get_by_role('img',name='无知路人原始牌面');expect(img).to_be_visible()
    expect(img).to_have_js_property('complete',True);assert img.evaluate('e=>e.naturalWidth')>0
    for key in ['Tab']*16+['Shift+Tab']*8:
     page.keyboard.press(key);f=focus(page);assert f['inReader'] and not f['inert']
    reader_tree=ax(page);assert [n['name'] for n in reader_tree if n['role']=='dialog']==['放大阅读无知路人']
    page.screenshot(path=str(root/f'reader-{width}x{height}.png'),full_page=True)
    page.keyboard.press('Escape');expect(reader).to_have_count(0);expect(opener).to_be_focused();assert selected(page)==['synthetic-hand-0']
    page.keyboard.press('Escape');expect(dialog).to_be_visible();assert len(submissions(page))==1
    opener.click();reader.get_by_role('button',name='关闭放大阅读').click();expect(opener).to_be_focused();assert selected(page)==['synthetic-hand-0']
    dialog.get_by_role('button',name='确认选择').click();expect(dialog).to_have_count(0);assert submissions(page)[1]['selected']==['synthetic-hand-0']
    page.get_by_role('button',name='打开本地选择').click();page.get_by_role('button',name='上手指南',exact=True).click()
    guide=page.get_by_role('dialog',name='上手指南');expect(guide).to_be_visible()
    expect(page.locator('.hg-choice')).to_be_attached();assert page.get_by_role('dialog',name='待完成的选择').count()==0
    guide_tree=ax(page);assert [n['name'] for n in guide_tree if n['role']=='dialog']==['上手指南']
    for key in ['Tab']*12+['Shift+Tab']*6:
     page.keyboard.press(key);assert focus(page)['inGuide']
    page.keyboard.press('Escape');expect(guide).to_have_count(0);expect(dialog).to_be_visible();assert focus(page)['inChoice'];assert not dialog.evaluate('e=>!!e.closest("[inert]")')
    dialog.get_by_role('button',name='保留全部手牌').click();assert len(submissions(page))==3
    assert not page.locator('[inert]').count();assert not errors
    results.append({'viewport':[width,height],'choiceAX':tree,'readerAX':reader_tree,'guideAX':guide_tree,'choiceTabSteps':len(keys),'readerTabSteps':24,'guideTabSteps':18,'restoredAutoPassFocus':True,'restoredReaderOpenerFocus':True,'selectionRetainedAcrossBothReaderDismissals':True,'escapeNeverSubmittedChoice':True,'submissions':submissions(page),'consoleErrors':errors})
    page.close()
  finally:browser.close()
 data={'scope':'localhost synthetic Table/Help components under React.StrictMode; natural UI click and keyboard only; evaluations read DOM/focus/AX only; no game room or external API','originalImageSha256':hashlib.sha256((root/'public/cards/JC125.jpg').read_bytes()).hexdigest(),'results':results}
 (root/'result.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n');print(json.dumps(data,ensure_ascii=False))
finally:server.terminate();server.wait(timeout=5)
