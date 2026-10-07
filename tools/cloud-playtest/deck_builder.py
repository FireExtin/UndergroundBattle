"""Deck UI browser regression, using an explicitly supplied public catalog.

Run against a local Vite preview. Only /api/catalog is fulfilled from the given
recording; no room is created and no gameplay or other API write is allowed.
This proves frontend interactions, not live server deck admission or deployment.
The missing-image scenario deliberately returns 404 for one registered original.
"""
import argparse
import hashlib
import json
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


def run(args):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    raw_catalog = Path(args.catalog).read_bytes()
    catalog = json.loads(raw_catalog)
    report = {
        'scope': 'local-frontend-public-catalog-recording',
        'baseUrl': args.base_url, 'catalogSha256': hashlib.sha256(raw_catalog).hexdigest(),
        'versions': {key: catalog[key] for key in ('rulesVersion', 'cardPoolVersion', 'engineVersion')},
        'liveServerAdmissionTested': False, 'published': False,
        'apiWrites': [], 'browserErrors': [], 'checkpoints': [], 'screenshots': {}, 'passed': False,
    }

    def checkpoint(name):
        report['checkpoints'].append(name)

    def screenshot(page, name):
        path = output / f'{name}.png'
        page.screenshot(path=str(path))
        report['screenshots'][path.name] = hashlib.sha256(path.read_bytes()).hexdigest()

    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True,
                                                 args=['--no-sandbox', '--disable-dev-shm-usage'])
            try:
                page = browser.new_page(viewport={'width': 1440, 'height': 1000})
                page.on('pageerror', lambda error: report['browserErrors'].append(str(error)))

                def api(route):
                    if route.request.url.endswith('/api/catalog') and route.request.method == 'GET':
                        route.fulfill(json=catalog)
                    else:
                        report['apiWrites'].append({'method': route.request.method, 'path': route.request.url.split('/api/', 1)[-1]})
                        route.abort()

                page.route('**/api/**', api)
                page.goto(args.base_url, wait_until='networkidle')
                page.get_by_role('button', name='展开组卡工作台').click()
                page.get_by_role('button', name='复制为草稿').click()
                expect(page.get_by_role('button', name='保存并选择此牌组')).to_be_enabled()
                page.get_by_role('textbox', name='牌组名称').fill('卡面浏览验收')
                positions = page.locator('[data-catalog-card]').evaluate_all('(cards) => cards.slice(0, 12).map(card => ({x: card.getBoundingClientRect().x, y: card.getBoundingClientRect().y}))')
                first_row = sum(abs(card['y'] - positions[0]['y']) < 2 for card in positions)
                assert first_row >= 4 and any(card['y'] > positions[0]['y'] + 100 for card in positions), positions
                assert not page.evaluate('document.documentElement.scrollWidth > innerWidth')
                page.locator('[data-catalog-card="JC125"] img').wait_for()
                expect(page.locator('[data-catalog-card="JC125"] img')).to_have_js_property('complete', True)
                assert page.locator('[data-catalog-card="JC125"] img').evaluate('(img) => img.naturalWidth') > 0
                screenshot(page, 'desktop-grid')
                checkpoint(f'desktop-{first_row}-columns-and-multiple-rows')

                name = '无知路人（JC125）'
                initial = int(page.get_by_role('spinbutton', name=name + '张数').input_value())
                page.get_by_role('button', name='添加 ' + name, exact=True).click(click_count=4, delay=35)
                expect(page.get_by_role('spinbutton', name=name + '张数')).to_have_value(str(initial + 4))
                opener = page.get_by_role('button', name='预览 ' + name, exact=True)
                opener.click()
                dialog = page.get_by_role('dialog', name='卡面预览 ' + name)
                expect(dialog.get_by_role('img')).to_have_js_property('complete', True)
                assert dialog.get_by_role('img').evaluate('(img) => img.naturalWidth') >= 400
                dialog.get_by_role('button', name='预览减少 ' + name).click(click_count=3, delay=35)
                expect(dialog.get_by_role('status')).to_have_text(f'已加入 {initial + 1} 张')
                screenshot(page, 'desktop-original-preview')
                page.keyboard.press('Escape')
                expect(opener).to_be_focused()
                expect(page.get_by_role('spinbutton', name=name + '张数')).to_have_value(str(initial + 1))
                page.get_by_role('button', name='预览已选 ' + name).click()
                page.get_by_role('button', name='关闭卡面预览').click()
                expect(page.get_by_role('dialog', name='卡面组卡工作台')).to_be_visible()
                checkpoint('repeated-grid-preview-and-selected-list-counts-and-focus')

                page.get_by_role('combobox', name='印刷费用').select_option('0')
                page.get_by_role('searchbox', name='检索卡牌').fill('JC125')
                page.get_by_role('checkbox', name='只看已加入').check()
                expect(page.locator('[data-catalog-card]')).to_have_count(1)
                page.get_by_role('searchbox', name='检索卡牌').fill('没有这张卡')
                expect(page.locator('[data-catalog-card]')).to_have_count(0)
                expect(page.get_by_text('当前开放目录没有符合筛选的卡牌。')).to_be_visible()
                page.get_by_role('button', name='重置筛选').click()
                expect(page.get_by_role('checkbox', name='只看已加入')).not_to_be_checked()
                checkpoint('zero-cost-selected-search-empty-reset')
                page.get_by_role('combobox', name='派系', exact=True).select_option('yellow')
                page.get_by_role('combobox', name='卡牌类型').select_option('character')
                page.get_by_role('combobox', name='印刷费用').select_option('2')
                page.get_by_role('searchbox', name='检索卡牌').fill('JC002')
                expect(page.locator('[data-catalog-card]')).to_have_count(1)
                expect(page.locator('[data-catalog-card]')).to_have_attribute('data-catalog-card', 'JC002')
                page.get_by_role('button', name='重置筛选').click()
                checkpoint('combined-faction-type-cost-id-filters')

                limited = catalog['cards'][1]
                limited_name = f'{limited["name"]}（{limited["id"]}）'
                assert limited['deckCopyLimit'] == 3
                page.get_by_role('searchbox', name='检索卡牌').fill(limited['id'])
                current = int(page.get_by_role('spinbutton', name=limited_name + '张数').input_value())
                page.get_by_role('button', name='添加 ' + limited_name, exact=True).click(click_count=4-current, delay=35)
                expect(page.get_by_role('spinbutton', name=limited_name + '张数')).to_have_value('4')
                expect(page.get_by_role('button', name='保存并选择此牌组')).to_be_disabled()
                expect(page.locator(f'[data-catalog-card="{limited["id"]}"]')).to_have_attribute('data-card-issue', 'copies')
                page.get_by_role('list', name='牌组校验问题').scroll_into_view_if_needed()
                screenshot(page, 'desktop-copy-limit-feedback')
                page.get_by_role('button', name='保存牌组', exact=True).click()
                page.get_by_role('button', name='返回大厅组卡').click()
                expect(page.get_by_role('dialog')).to_have_count(0)
                page.reload(wait_until='networkidle')
                expect(page.get_by_role('textbox', name='牌组名称')).to_have_value('卡面浏览验收')
                expect(page.get_by_role('spinbutton', name=limited_name + '张数')).to_have_value('4')
                expect(page.get_by_role('button', name='保存并选择此牌组')).to_be_disabled()
                checkpoint('invalid-draft-saved-and-restored-with-legality-retained')

                page.get_by_role('button', name='目录减少 ' + limited_name).click()
                expect(page.get_by_role('button', name='保存并选择此牌组')).to_be_enabled()
                page.get_by_role('button', name='保存并选择此牌组').click()
                expect(page.locator('.hg-entry-form')).to_contain_text('当前牌组：卡面浏览验收')
                expect(page.locator('.hg-library-saved')).to_contain_text('已选择入席')
                page.get_by_role('button', name='展开组卡工作台').click()
                page.get_by_role('searchbox', name='检索卡牌').fill('JC125')
                page.keyboard.press('Escape')
                expect(page.get_by_role('button', name='展开组卡工作台')).to_be_focused()
                page.get_by_role('button', name='展开组卡工作台').click()
                expect(page.get_by_role('searchbox')).to_have_value('JC125')
                checkpoint('valid-draft-selection-and-workspace-close-reopen')
                page.get_by_role('button', name='重置筛选').click()
                extra = next(card for card in catalog['cards'] if card['id'] == 'JC008')
                extra_name = f'{extra["name"]}（{extra["id"]}）'
                page.get_by_role('searchbox', name='检索卡牌').fill(extra['id'])
                page.get_by_role('button', name='添加 ' + extra_name, exact=True).click()
                page.get_by_role('checkbox', name='只看已加入').check()
                page.get_by_role('button', name='预览 ' + extra_name, exact=True).click()
                page.get_by_role('button', name='预览减少 ' + extra_name).click()
                expect(page.get_by_role('dialog', name='卡面预览 ' + extra_name).get_by_role('status')).to_have_text('已加入 0 张')
                page.keyboard.press('Escape')
                expect(page.get_by_role('searchbox')).to_be_focused()
                expect(page.locator('[data-catalog-card]')).to_have_count(0)
                checkpoint('last-selected-copy-removal-and-search-focus-fallback')
                page.get_by_role('button', name='返回大厅组卡').click()

                page.set_viewport_size({'width': 390, 'height': 844})
                page.get_by_role('button', name='重置筛选').click()
                page.locator('.hg-library-catalog').scroll_into_view_if_needed()
                assert not page.evaluate('document.documentElement.scrollWidth > innerWidth')
                first = page.locator('[data-catalog-card]').first
                second = page.locator('[data-catalog-card]').nth(1)
                assert abs(first.bounding_box()['y'] - second.bounding_box()['y']) < 2
                first.scroll_into_view_if_needed()
                screenshot(page, 'mobile-grid')
                page.get_by_role('button', name='预览 ' + name, exact=True).click()
                expect(page.get_by_role('button', name='关闭卡面预览')).to_be_in_viewport()
                page.get_by_role('button', name='预览添加 ' + name).click()
                expect(page.get_by_role('button', name='关闭卡面预览')).to_be_in_viewport()
                page.get_by_role('dialog').evaluate('(el) => el.scrollTop = 0')
                screenshot(page, 'mobile-original-preview')
                page.get_by_role('button', name='关闭卡面预览').click()
                expect(page.get_by_role('dialog')).to_have_count(0)
                checkpoint('mobile-two-columns-preview-controls-close-no-overflow')

                page.set_viewport_size({'width': 1440, 'height': 1000})
                page.route('**/cards/JC125.jpg', lambda route: route.fulfill(status=404, body='simulated missing original'))
                page.reload(wait_until='networkidle')
                page.get_by_role('button', name='展开组卡工作台').click()
                page.get_by_role('searchbox', name='检索卡牌').fill('JC125')
                expect(page.locator('[data-catalog-card="JC125"] [data-face-state="missing"]')).to_be_visible()
                page.get_by_role('button', name='预览 ' + name, exact=True).click()
                expect(page.get_by_role('dialog').get_by_text('暂无原始牌面')).to_be_visible()
                screenshot(page, 'desktop-image-failure-placeholder')
                page.locator('.hg-library-preview-backdrop').click(position={'x': 5, 'y': 5})
                expect(page.get_by_role('dialog', name='卡面组卡工作台')).to_be_visible()
                checkpoint('simulated-image-404-placeholder-and-backdrop-close')

                assert not report['apiWrites'], report['apiWrites']
                assert not report['browserErrors'], report['browserErrors']
                report['passed'] = True
            finally:
                browser.close()
    finally:
        (output / 'browser-results.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'passed': report['passed'], 'checkpoints': len(report['checkpoints']), 'screenshots': len(report['screenshots'])}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', default='http://localhost:5173')
    parser.add_argument('--catalog', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    run(parser.parse_args())
