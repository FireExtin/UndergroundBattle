import json,subprocess,socket,urllib.request,time
from pathlib import Path
from playwright.sync_api import sync_playwright, Error
root=Path('/workspace/.private-validation/v027-modal-diagnostic-20261003')
server=subprocess.Popen(['python','-m','http.server','8124','--bind','127.0.0.1','--directory',str(root/'dist')],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
try:
 for _ in range(50):
  try:
   urllib.request.urlopen('http://127.0.0.1:8124/',timeout=1).close();break
  except OSError:time.sleep(.1)
 results=[]
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True)
  try:
   for width,height in [(1536,960),(1366,768),(1280,800)]:
    page=browser.new_page(viewport={'width':width,'height':height})
    page.goto('http://127.0.0.1:8124/',wait_until='networkidle')
    if not page.locator('.hg-table').count():
     results.append({'viewport':[width,height],'blocked':page.locator('body').inner_text()[:500]});page.close();break
    switch=page.get_by_role('switch',name='无可用行动时自动让过');box=switch.bounding_box();assert box
    center={'x':box['x']+box['width']/2,'y':box['y']+box['height']/2}
    hit=page.evaluate('p=>{const e=document.elementFromPoint(p.x,p.y);const b=e?.closest("button");return {tag:e?.tagName,text:b?.innerText,choiceOption:b?.getAttribute("data-choice-option"),insideChoice:!!e?.closest(".hg-table-choice-layer")};}',center)
    cdp=page.context.new_cdp_session(page);tree=cdp.send('Accessibility.getFullAXTree')
    accessible=[n for n in tree['nodes'] if n.get('role',{}).get('value')=='switch' and n.get('name',{}).get('value')=='无可用行动时自动让过']
    before=page.locator('[data-choice-option][aria-pressed="true"]').count()
    page.mouse.click(center['x'],center['y'])
    selected=page.locator('[data-choice-option][aria-pressed="true"]').evaluate_all('es=>es.map(e=>e.getAttribute("data-choice-option"))')
    focus=[]
    for _ in range(30):
     page.keyboard.press('Tab')
     focus.append(page.evaluate('()=>({tag:document.activeElement?.tagName,role:document.activeElement?.getAttribute("role"),label:document.activeElement?.getAttribute("aria-label"),insideChoice:!!document.activeElement?.closest(".hg-table-choice-layer")})'))
    record={'viewport':[width,height],'switchVisibleToRoleQuery':switch.count()==1,'switchInAXTree':bool(accessible),'switchAXIgnored':[n.get('ignored') for n in accessible],'switchBox':box,'physicalHit':hit,'selectedBefore':before,'selectedAfter':selected,'autoPassCheckedAfter':switch.is_checked(),'submissions':page.locator('#submissions').inner_text(),'choiceRole':page.locator('.hg-choice').get_attribute('role'),'choiceAriaModal':page.locator('.hg-choice').get_attribute('aria-modal'),'keyboardReachedBackgroundSwitch':any(f['role']=='switch' for f in focus),'focus':focus}
    page.screenshot(path=str(root/f'viewport-{width}x{height}.png'),full_page=True)
    results.append(record);page.close()
  finally:browser.close()
 (root/'result.json').write_text(json.dumps({'scope':'isolated local synthetic component fixture; no game room, state injection or remote API','results':results},ensure_ascii=False,indent=2)+'\n')
 print(json.dumps([{k:v for k,v in r.items() if k!='focus'} for r in results],ensure_ascii=False))
finally:
 server.terminate();server.wait(timeout=5)
