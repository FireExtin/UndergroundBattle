import { describe, expect, it } from 'vitest';
import raw from '../../../rust-game/data/cards.json';
import { previewUwDeckImport, UW_IMPORT_MAX_BYTES } from './uwDeckImport';
import { UW_PLAYER_MAPPINGS, UW_SOCIETY_MAPPINGS } from './uwDeckManifest';

import { importCatalog, entry, uwText, validImportText } from '../test/uwDeckImportFixtures';

describe('UWPro strict deck preview', () => {
  it('checks all 77 explicit identities against the real print registry, leaving the 3 image conflicts isolated', () => {
    expect(UW_PLAYER_MAPPINGS).toHaveLength(77);
    expect(new Set(UW_PLAYER_MAPPINGS.map(item => item.sourcePath)).size).toBe(77);
    expect(new Set(UW_PLAYER_MAPPINGS.map(item => item.targetId)).size).toBe(77);
    const quarantined: string[] = [];
    for (const item of UW_PLAYER_MAPPINGS) {
      const card = raw.cards.find(card => card.id === item.targetId)!;
      expect([card.name, card.kind, card.color, card.cost]).toEqual([item.name, item.kind, item.color, item.cost]);
      expect(['红', '黑', '蓝', '灰']).toContain(item.color);
      const preview = previewUwDeckImport(uwText([entry(item.targetId)]), importCatalog);
      if (['JC029', 'JZ49', 'JC044'].includes(item.targetId)) {
        expect(preview.rows[0].status).toBe('quarantined'); expect(preview.draft).toBeNull(); quarantined.push(item.targetId);
      } else {
        expect(preview.rows[0].status).toBe('mapped'); expect(preview.draft?.cards).toEqual([{ cardId: item.targetId, count: 1 }]);
        expect(preview.validation?.valid).toBe(false);
      }
    }
    expect(quarantined.sort()).toEqual(['JC029', 'JC044', 'JZ49']);
  });
  it('produces a complete valid current-catalog draft without modifying either input or inventing a society', () => {
    const before = JSON.stringify(importCatalog); const source = validImportText();
    const preview = previewUwDeckImport(source, importCatalog);
    expect(preview.totalCards).toBe(50); expect(preview.mappedCards).toBe(50);
    expect(preview.validation?.valid).toBe(true); expect(preview.draft?.societyId).toBeNull();
    expect(preview.draft).toMatchObject({ engineVersion: raw.engineVersion, rulesVersion: raw.rulesVersion, cardPoolVersion: raw.cardPoolVersion });
    expect(JSON.stringify(importCatalog)).toBe(before);
    expect(previewUwDeckImport(source, importCatalog)).toEqual(preview);
  });
  it('retains duplicate entries and combines their counts before the existing same-name copy validator', () => {
    const preview = previewUwDeckImport(uwText([entry('JC084', 2), entry('JC084', 2)]), importCatalog);
    expect(preview.rows).toHaveLength(2); expect(preview.totalCards).toBe(4);
    expect(preview.draft?.cards).toEqual([{ cardId: 'JC084', count: 4 }]);
    expect(preview.validation?.issues.some(issue => issue.code === 'copies')).toBe(true);
    const sameName = previewUwDeckImport(uwText([entry('XQ14', 2), entry('BQ040', 2)]), importCatalog);
    expect(sameName.draft?.cards).toHaveLength(2);
    expect(sameName.validation?.issues.some(issue => issue.code === 'copies')).toBe(true);
  });
  it.each([0, -1, 1.5, '3', null, 2049, Number.MAX_SAFE_INTEGER])('rejects invalid copies %s without coercion or a partial draft', copies => {
    const preview = previewUwDeckImport(uwText([entry('JC084'), { ...entry('JC084'), copies }]), importCatalog);
    expect(preview.rows).toHaveLength(2); expect(preview.rows[1].status).toBe('invalid'); expect(preview.draft).toBeNull();
  });
  it('rejects excessive totals and entries without truncating the source into a usable subset', () => {
    const preview = previewUwDeckImport(uwText([entry('JC084', 2048), entry('JC084')]), importCatalog);
    expect(preview.totalCards).toBe(2049); expect(preview.draft).toBeNull(); expect(preview.rows).toHaveLength(2);
    expect(previewUwDeckImport(uwText(Array.from({ length: 2049 }, () => entry('JC084'))), importCatalog).draft).toBeNull();
    expect(previewUwDeckImport(' '.repeat(UW_IMPORT_MAX_BYTES + 1), importCatalog).issues[0]).toContain('256 KiB');
    expect(previewUwDeckImport(JSON.stringify({ deckName: '字'.repeat(90_000), cards: [] }), importCatalog).issues[0]).toContain('256 KiB');
  });
  it.each(['{', 'null', '[]', '{"cards":{}}', '{"cards":[],"councils":{}}', '{"cards":[]}', '{"deckName":"","cards":[]}'])('rejects malformed document %s', text => {
    const result = previewUwDeckImport(text, importCatalog); expect(result.issues.length).toBeGreaterThan(0); expect(result.draft).toBeNull();
  });
  it('accepts a UTF-8 BOM but rejects extra version, state, and executable-looking fields', () => {
    expect(previewUwDeckImport('\uFEFF' + validImportText(), importCatalog).validation?.valid).toBe(true);
    for (const key of ['engineVersion', 'state', '__proto__']) {
      const text = validImportText().replace(/}$/, `,${JSON.stringify(key)}:{"seed":"123"}}`);
      expect(previewUwDeckImport(text, importCatalog).draft).toBeNull();
    }
    expect(previewUwDeckImport(uwText([{ ...entry('JC084'), effect: 'draw' }]), importCatalog).draft).toBeNull();
  });
  it('rejects deeply nested invalid fields without throwing during error presentation', () => {
    const nested = '{"x":'.repeat(10_000) + '0' + '}'.repeat(10_000);
    for (const invalid of [`{"templatePath":${nested},"copies":1}`, `{"templatePath":${JSON.stringify(entry('JC084').templatePath)},"copies":${nested}}`, '['.repeat(10_000) + '0' + ']'.repeat(10_000)]) {
      const text = `{"deckName":"深层无效字段","cards":[${invalid}]}`;
      expect(text.length).toBeLessThan(UW_IMPORT_MAX_BYTES);
      const result = previewUwDeckImport(text, importCatalog);
      expect(result.rows[0].status).toBe('invalid'); expect(result.draft).toBeNull();
    }
  });
  it.each([
    'res://Cards/Data/PlayerCards/JC/JC043_zonghuokuang.tres',
    'res://Cards/Data/PlayerCards/JC/JC125_wuzhiluren.tres',
    'res://Cards/Data/AreaCards/DQ001_chenmodefeixu.tres',
    'res://Cards/Data/PlayerCards/XQ/XQ23_jinshendebanshiyuan.tres',
    'res://Cards/Data/PlayerCards/JC/../JC/JC084_jietouhunhun.tres',
    'https://example.com/JC084_jietouhunhun.tres',
  ])('keeps an unresolved source visible and blocks the whole conversion: %s', templatePath => {
    const result = previewUwDeckImport(uwText([entry('JC084', 2), { templatePath, copies: 3 }]), importCatalog);
    expect(result.rows[1]).toMatchObject({ sourcePath: templatePath, copies: 3, status: 'unmapped' });
    expect(result.totalCards).toBe(5); expect(result.mappedCards).toBe(2); expect(result.draft).toBeNull();
  });
  it('does not infer an external mapping when a future target catalog adds that card', () => {
    const future = { ...importCatalog, cards: [...importCatalog.cards, { ...importCatalog.cards[0], id: 'JC043', name: '纵火狂', supported: true }] };
    expect(previewUwDeckImport(uwText([{ templatePath: 'res://Cards/Data/PlayerCards/JC/JC043_zonghuokuang.tres', copies: 1 }]), future).rows[0].status).toBe('unmapped');
  });
  it('fails closed for unadmitted, duplicate, missing, changed, or incomplete target identities', () => {
    for (const change of [{ supported: false }, { deckCopyLimit: undefined }, { name: '同号异牌' }, { cost: 999 }, { color: '绿' }, { kind: 'region' }, { permanentIcons: { investigation: 9, combat: 0, influence: 0 } }, { icons: undefined, permanentIcons: undefined, temporaryIcons: undefined }]) {
      const catalog = { ...importCatalog, cards: importCatalog.cards.map(card => card.id === 'JC084' ? { ...card, ...change } : card) };
      expect(previewUwDeckImport(uwText([entry('JC084')]), catalog).draft).toBeNull();
    }
    for (const cards of [importCatalog.cards.filter(card => card.id !== 'JC084'), [...importCatalog.cards, importCatalog.cards.find(card => card.id === 'JC084')!]]) {
      expect(previewUwDeckImport(uwText([entry('JC084')]), { ...importCatalog, cards }).draft).toBeNull();
    }
  });
  it('uses current versions and revalidates society constraints without defaulting or dropping council entries', () => {
    const society = (id: string, copies = 1) => ({ templatePath: UW_SOCIETY_MAPPINGS.find(item => item.targetId === id)!.sourcePath, copies });
    const catalog = { ...importCatalog, engineVersion: 'review-next-engine' };
    const result = previewUwDeckImport(uwText([entry('JC084')], [society('MSJC03')]), catalog);
    expect(result.draft).toMatchObject({ societyId: 'MSJC03', engineVersion: 'review-next-engine' });
    expect(result.validation?.issues.some(issue => issue.code === 'society-color')).toBe(true);
    for (const councils of [[society('MSJC09', 2)], [society('MSJC09'), society('MSJC09')], [{ templatePath: 'res://Cards/Data/CouncilCards/JC/MSJC013_jishizhe.tres', copies: 1 }]]) {
      expect(previewUwDeckImport(uwText([entry('JC084')], councils), catalog).draft).toBeNull();
    }
    const unavailable = { ...catalog, deckBuildRules: { ...catalog.deckBuildRules!, societySupported: false } };
    expect(previewUwDeckImport(uwText([entry('JC084')], [society('MSJC09')]), unavailable).draft).toBeNull();
  });
});
