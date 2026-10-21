import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { UwDeckImportPanel } from './UwDeckImportPanel';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, readDeckLibrary, saveDeckDraft } from './deckLibrary';
import { importCatalog, entry, uwText, validImportText } from '../test/uwDeckImportFixtures';

beforeEach(() => localStorage.clear());
afterEach(() => localStorage.clear());
const open = () => fireEvent.click(screen.getByText('导入 UWPro 牌组', { selector: 'summary' }));
const preview = (text: string) => {
  fireEvent.change(screen.getByRole('textbox', { name: 'UWPro 牌组 JSON' }), { target: { value: text } });
  fireEvent.click(screen.getByRole('button', { name: '解析并预览' }));
};

describe('UWPro deck preview UI', () => {
  it('loads a fresh draft through the existing editor without saving, selecting, or overwriting an old draft', () => {
    const select = vi.fn(); const old = { ...createDeckDraft(importCatalog), name: '原有草稿' }; saveDeckDraft(old);
    render(<DeckLibrary catalog={importCatalog} onSelectDraft={select} />); open(); preview(validImportText());
    expect(readDeckLibrary().drafts).toHaveLength(1); expect(select).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '载入为新草稿' }));
    expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('UW导入牌组');
    expect(readDeckLibrary().drafts).toHaveLength(1); expect(select).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(select).toHaveBeenCalledTimes(1);
    const imported = select.mock.calls[0][0];
    expect(imported.id).not.toBe(old.id); expect(imported.id).not.toBe('uwpro-import-preview');
    expect(imported.cards.reduce((sum: number, card: { count: number }) => sum + card.count, 0)).toBe(50);
    expect(readDeckLibrary().drafts.map(item => item.name)).toEqual(expect.arrayContaining(['原有草稿', 'UW导入牌组']));
  });
  it('keeps unknown entries visible as escaped text and never imports the mapped subset', () => {
    const onImport = vi.fn(); render(<UwDeckImportPanel catalog={importCatalog} onImport={onImport} />); open();
    const path = '<img src=x onerror=alert(1)>';
    preview(uwText([entry('JC084'), { templatePath: path, copies: 49 }]));
    expect(screen.getByText(path)).toBeInTheDocument(); expect(document.querySelector('img[src="x"]')).toBeNull();
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '载入为新草稿' })); expect(onImport).not.toHaveBeenCalled();
    expect(screen.getByRole('textbox', { name: 'UWPro 牌组 JSON' })).toHaveValue(uwText([entry('JC084'), { templatePath: path, copies: 49 }]));
  });
  it('may load a completely mapped incomplete draft but keeps existing entry selection disabled', () => {
    const select = vi.fn(); render(<DeckLibrary catalog={importCatalog} onSelectDraft={select} />); open(); preview(uwText([entry('JC084', 4)]));
    expect(within(screen.getByRole('list', { name: '导入构筑问题' })).getByText(/同名最多 3 张/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '载入为新草稿' }));
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled(); expect(select).not.toHaveBeenCalled();
    expect(screen.getByRole('spinbutton', { name: '街头混混（JC084）张数' })).toHaveValue(4);
  });
  it('protects unsaved edits and blocks loading when the parent is disabled', () => {
    const onImport = vi.fn(); const view = render(<UwDeckImportPanel catalog={importCatalog} onImport={onImport} hasUnsavedChanges />); open(); preview(validImportText());
    expect(screen.getByText(/当前草稿有未保存修改/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeDisabled();
    view.rerender(<UwDeckImportPanel catalog={importCatalog} onImport={onImport} disabled />);
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeDisabled(); expect(onImport).not.toHaveBeenCalled();
  });
  it('invalidates the preview after editing and rechecks a changed catalog before loading', () => {
    const onImport = vi.fn(); const view = render(<UwDeckImportPanel catalog={importCatalog} onImport={onImport} />); open(); preview(validImportText());
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeEnabled();
    view.rerender(<UwDeckImportPanel catalog={{ ...importCatalog, cards: [] }} onImport={onImport} />);
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeDisabled();
    fireEvent.change(screen.getByRole('textbox', { name: 'UWPro 牌组 JSON' }), { target: { value: '{}' } });
    expect(screen.queryByRole('region', { name: 'UWPro 导入预览' })).not.toBeInTheDocument(); expect(onImport).not.toHaveBeenCalled();
  });
  it('does not let a slow file read replace newly pasted input or revive an old preview', async () => {
    render(<UwDeckImportPanel catalog={importCatalog} onImport={vi.fn()} />); open();
    let finish!: (value: string) => void;
    const file = new File(['{}'], 'old.json', { type: 'application/json' });
    Object.defineProperty(file, 'text', { value: () => new Promise<string>(resolve => { finish = resolve; }) });
    fireEvent.change(screen.getByLabelText('选择牌组 JSON 文件'), { target: { files: [file] } });
    preview(validImportText());
    await act(async () => { finish('{"deckName":"old","cards":[]}'); });
    expect(screen.getByRole('textbox', { name: 'UWPro 牌组 JSON' })).toHaveValue(validImportText());
    expect(screen.getByRole('button', { name: '载入为新草稿' })).toBeEnabled();
  });
});
