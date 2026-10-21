import { deckCardAdmission, selectableSocieties, validateDeckDraft } from './deckLibrary';
import type { DeckCatalog, DeckDraft, DeckValidation } from './deckLibrary';
import type { CardDefinition, Icons } from './types';
import { UW_PLAYER_MAPPINGS, UW_SOCIETY_MAPPINGS } from './uwDeckManifest';
import type { UwPlayerMapping } from './uwDeckManifest';

// Resource paths are identifiers only. This module never loads a Godot resource,
// requests a URL, copies external card definitions, or restores game state.
export const UW_IMPORT_MAX_BYTES = 262_144;
export const UW_IMPORT_MAX_CARDS = 2048;
export type UwImportRow = {
  zone: 'cards' | 'councils'; index: number; sourcePath: string; suppliedCopies: string;
  copies: number | null; cardId?: string; name?: string;
  status: 'mapped' | 'unmapped' | 'quarantined' | 'unavailable' | 'changed' | 'invalid';
  messages: string[];
};
export type UwImportPreview = {
  name: string; rows: UwImportRow[]; issues: string[]; totalCards: number | null;
  mappedCards: number; draft: DeckDraft | null; validation: DeckValidation | null;
};
const players = new Map(UW_PLAYER_MAPPINGS.map(item => [item.sourcePath, item]));
const societies = new Map(UW_SOCIETY_MAPPINGS.map(item => [item.sourcePath, item]));
const record = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null && !Array.isArray(value);
// JSON.parse accepts very deep objects; never recursively stringify an invalid
// field just to display it. The complete original input stays in the editor.
const display = (value: unknown): string => value === undefined ? '未提供' : value === null ? 'null'
  : typeof value === 'string' ? value : typeof value === 'number' || typeof value === 'boolean' ? String(value)
  : Array.isArray(value) ? '[数组，格式无效]' : '[对象，格式无效]';
const extraKeys = (value: Record<string, unknown>, allowed: string[]) => Object.keys(value).some(key => !allowed.includes(key));
const iconValues = (icons?: Icons) => icons ? [icons.investigation, icons.combat, icons.influence] : [];
function identityMatches(card: CardDefinition, mapping: UwPlayerMapping) {
  const permanent = iconValues(card.permanentIcons || card.icons?.permanent);
  const temporary = iconValues(card.temporaryIcons || card.icons?.temporary);
  return card.name === mapping.name && card.kind === mapping.kind && card.color === mapping.color && card.cost === mapping.cost
    && permanent.length === 3 && temporary.length === 3
    && mapping.permanent.every((value, index) => permanent[index] === value)
    && mapping.temporary.every((value, index) => temporary[index] === value);
}

/** Preview all supplied entries; any unresolved entry prevents draft conversion. */
export function previewUwDeckImport(text: string, catalog: DeckCatalog): UwImportPreview {
  const result: UwImportPreview = { name: '', rows: [], issues: [], totalCards: 0, mappedCards: 0, draft: null, validation: null };
  if (text.length > UW_IMPORT_MAX_BYTES || new TextEncoder().encode(text).length > UW_IMPORT_MAX_BYTES) {
    result.issues.push('文件超过 256 KiB，请缩小牌组文件后再导入。'); return result;
  }
  let source: unknown;
  try { source = JSON.parse(text.replace(/^\uFEFF/, '')); }
  catch { result.issues.push('JSON 无法解析，请使用 UWPro 导出的牌组文件。'); return result; }
  if (!record(source) || !Array.isArray(source.cards) || (source.councils !== undefined && !Array.isArray(source.councils))) {
    result.issues.push('牌组须包含 cards 数组；councils 如有提供，也须是数组。'); return result;
  }
  if (extraKeys(source, ['deckName', 'cards', 'councils'])) result.issues.push('文件包含此导入格式未支持的字段，不能转换为草稿。');
  result.name = typeof source.deckName === 'string' ? source.deckName.trim() : '';
  if (!result.name || [...result.name].length > 80) result.issues.push('deckName 须为 1–80 字的牌组名称。');
  const councils = (source.councils || []) as unknown[];
  if (source.cards.length > UW_IMPORT_MAX_CARDS || councils.length > UW_IMPORT_MAX_CARDS) {
    result.issues.push('条目数量超过服务容量 2048，未解析或截取部分牌组。'); return result;
  }
  if (councils.length > 1) result.issues.push('当前牌组只能选择一张秘社；请明确调整原文件后再导入。');
  const counts = new Map<string, number>();
  let societyId: string | null = null;
  for (const [zone, entries] of [['cards', source.cards], ['councils', councils]] as const) {
    entries.forEach((entry: unknown, index: number) => {
      const row: UwImportRow = {
        zone, index, sourcePath: record(entry) ? display(entry.templatePath) : display(entry),
        suppliedCopies: record(entry) ? display(entry.copies) : '未提供', copies: null, status: 'invalid', messages: [],
      };
      result.rows.push(row);
      if (!record(entry) || typeof entry.templatePath !== 'string' || !entry.templatePath || extraKeys(entry, ['templatePath', 'copies'])) {
        row.messages.push('条目只支持 templatePath 字符串和 copies 正整数。');
        if (zone === 'cards') result.totalCards = null;
        return;
      }
      const copies = entry.copies;
      if (typeof copies !== 'number' || !Number.isSafeInteger(copies) || copies < 1 || copies > UW_IMPORT_MAX_CARDS) {
        row.messages.push('数量须为 1–2048 的整数；不会自动补成 1 或修正数量。');
        if (zone === 'cards') result.totalCards = null;
        return;
      }
      row.copies = copies;
      if (zone === 'cards' && result.totalCards !== null) result.totalCards += copies;
      if (zone === 'councils') {
        if (copies !== 1) { row.messages.push('秘社数量必须恰为 1。'); return; }
        const mapping = societies.get(entry.templatePath);
        if (!mapping) { row.status = 'unmapped'; row.messages.push('此秘社尚未建立已核验映射，不会替换成默认秘社。'); return; }
        row.cardId = mapping.targetId; row.name = mapping.name;
        const matches = selectableSocieties(catalog).filter(card => card.id === mapping.targetId);
        if (matches.length !== 1 || matches[0].supported !== true) {
          row.status = 'unavailable'; row.messages.push('当前目录未明确开放此秘社，或目录身份不唯一。'); return;
        }
        if (matches[0].name !== mapping.name) { row.status = 'changed'; row.messages.push('秘社目录身份已变化，需要重新核验映射。'); return; }
        societyId = mapping.targetId; row.status = 'mapped'; return;
      }
      const mapping = players.get(entry.templatePath);
      if (!mapping) { row.status = 'unmapped'; row.messages.push('来源路径未核验或超出本次红黑蓝灰范围；不会按编号猜测或丢弃此项。'); return; }
      row.cardId = mapping.targetId; row.name = mapping.name;
      if (mapping.quarantine) { row.status = 'quarantined'; row.messages.push(mapping.quarantine); return; }
      const matches = catalog.cards.filter(card => card.id === mapping.targetId);
      if (matches.length !== 1 || deckCardAdmission(matches[0]).status !== 'buildable') {
        row.status = 'unavailable'; row.messages.push('当前目录未开放此卡，或缺少唯一身份/构筑准入信息。'); return;
      }
      if (!identityMatches(matches[0], mapping)) {
        row.status = 'changed'; row.messages.push('名称、类别、颜色、费用或图标与已核验目录不一致，需要重新核验。'); return;
      }
      row.status = 'mapped'; row.messages.push(...(mapping.notes || []));
      counts.set(mapping.targetId, (counts.get(mapping.targetId) || 0) + copies);
      result.mappedCards += copies;
    });
  }
  if (result.totalCards !== null && result.totalCards > UW_IMPORT_MAX_CARDS) result.issues.push('总张数超过服务容量 2048，不会截断或丢弃条目。');
  if (result.rows.some(row => row.status !== 'mapped')) result.issues.push('存在未解决条目，仅可预览；整个牌组暂不转换为草稿。');
  if (result.issues.length) return result;
  // Pure preview has no random identity or side effects. The UI creates a fresh
  // normal DeckDraft only after the user accepts this complete conversion.
  result.draft = {
    id: 'uwpro-import-preview', name: result.name, description: '由 UWPro 牌组导入；使用当前开放目录。',
    societyId, cards: [...counts].map(([cardId, count]) => ({ cardId, count })),
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion,
    updatedAt: '1970-01-01T00:00:00.000Z',
  };
  result.validation = validateDeckDraft(result.draft, catalog);
  return result;
}
