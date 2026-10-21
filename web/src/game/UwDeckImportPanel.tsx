import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { createDeckDraft } from './deckLibrary';
import type { DeckCatalog, DeckDraft } from './deckLibrary';
import { previewUwDeckImport, UW_IMPORT_MAX_BYTES } from './uwDeckImport';
import './uw-deck-import.css';

export function UwDeckImportPanel({ catalog, disabled = false, hasUnsavedChanges = false, onImport }: {
  catalog: DeckCatalog; disabled?: boolean; hasUnsavedChanges?: boolean; onImport: (draft: DeckDraft) => void;
}) {
  const id = useId();
  const [text, setText] = useState('');
  const [checkedText, setCheckedText] = useState<string | null>(null);
  const [fileError, setFileError] = useState('');
  const [reading, setReading] = useState(false);
  const readSequence = useRef(0);
  useEffect(() => () => { readSequence.current++; }, []);
  const preview = useMemo(() => checkedText === null ? null : previewUwDeckImport(checkedText, catalog), [checkedText, catalog]);
  const editText = (value: string) => { readSequence.current++; setReading(false); setText(value); setCheckedText(null); setFileError(''); };
  const readFile = async (file?: File) => {
    if (!file) return;
    const sequence = ++readSequence.current;
    setCheckedText(null); setFileError(''); setReading(true);
    if (file.size > UW_IMPORT_MAX_BYTES) { setFileError('文件超过 256 KiB，请选择牌组 JSON 文件。'); setReading(false); return; }
    try {
      const contents = await file.text();
      if (sequence === readSequence.current) setText(contents);
    } catch { if (sequence === readSequence.current) setFileError('文件无法读取，原文本仍保留。'); }
    finally { if (sequence === readSequence.current) setReading(false); }
  };
  const load = () => {
    if (disabled || hasUnsavedChanges || reading || fileError || checkedText !== text) return;
    const current = previewUwDeckImport(text, catalog);
    if (!current.draft) return;
    const fresh = createDeckDraft(catalog);
    onImport({ ...current.draft, id: fresh.id, updatedAt: fresh.updatedAt });
  };
  return <details className="hg-uw-import">
    <summary>导入 UWPro 牌组</summary>
    <p>选择或粘贴 UWPro 牌组 JSON，先核对每张卡。当前核验红、黑、蓝、灰牌；未知、未开放或存在图标差异的卡只作预览。</p>
    <label className="hg-field" htmlFor={`${id}-file`}>选择牌组 JSON 文件</label>
    <input id={`${id}-file`} type="file" accept=".json,application/json" disabled={disabled} onChange={event => { void readFile(event.target.files?.[0]); event.target.value = ''; }} />
    <label className="hg-field" htmlFor={`${id}-text`}>UWPro 牌组 JSON</label>
    <textarea id={`${id}-text`} value={text} rows={5} spellCheck={false} disabled={disabled} onChange={event => editText(event.target.value)} />
    <div className="hg-uw-import-actions">
      <button type="button" className="hg-button hg-button-quiet" disabled={disabled || reading || !!fileError || !text.trim()} onClick={() => setCheckedText(text)}>解析并预览</button>
      <button type="button" className="hg-button hg-button-quiet" disabled={disabled || reading || !text} onClick={() => editText('')}>清空导入文本</button>
    </div>
    {reading && <p role="status">正在读取牌组文件。</p>}
    {fileError && <p role="alert">{fileError}</p>}
    {preview && <section aria-label="UWPro 导入预览">
      <h3>{preview.name || '牌组预览'}</h3>
      <p role="status">原牌组 {preview.totalCards === null ? '数量待修正' : `${preview.totalCards} 张`}；已核验映射 {preview.mappedCards} 张。
        {preview.draft ? preview.validation?.valid ? ' 构筑校验通过；载入后仍需保存并选择入席。' : ' 暂不可开局，可载入为草稿继续修正。' : ' 仅可预览，暂不能载入草稿或开局。'}</p>
      {preview.issues.length > 0 && <ul aria-label="导入问题">{preview.issues.map((message, index) => <li key={index}>{message}</li>)}</ul>}
      {preview.validation && !preview.validation.valid && <ul aria-label="导入构筑问题">{preview.validation.issues.map((issue, index) => <li key={index}>{issue.message}</li>)}</ul>}
      {preview.rows.length > 0 && <div className="hg-uw-import-rows" tabIndex={0} aria-label="逐项映射与差异">
        <table><thead><tr><th scope="col">来源与卡牌</th><th scope="col">数量</th><th scope="col">核验结果</th></tr></thead><tbody>
          {preview.rows.map(row => <tr key={`${row.zone}-${row.index}`}>
            <td><strong>{row.name || (row.zone === 'councils' ? '待核验秘社' : '待核验卡牌')}{row.cardId ? ` · ${row.cardId}` : ''}</strong><code>{row.sourcePath}</code></td>
            <td>{row.suppliedCopies}</td>
            <td><strong>{row.status === 'mapped' ? '已映射' : row.status === 'quarantined' ? '图标差异，已隔离' : row.status === 'unavailable' ? '当前未开放' : row.status === 'changed' ? '目录变化，待复核' : row.status === 'invalid' ? '格式或数量无效' : '尚未核验映射'}</strong>{row.messages.map((message, index) => <p key={index}>{message}</p>)}</td>
          </tr>)}
        </tbody></table>
      </div>}
      <p>原文件不会被修改。只有全部条目可转换时才能载入新草稿；重复条目的数量合计保留，不会自动补牌、补秘社或删除缺卡。</p>
      {hasUnsavedChanges && <p role="status">当前草稿有未保存修改，请先保存后再载入导入牌组。</p>}
      <button type="button" className="hg-button hg-button-primary" disabled={disabled || reading || !!fileError || hasUnsavedChanges || !preview.draft || checkedText !== text} onClick={load}>载入为新草稿</button>
    </section>}
  </details>;
}
