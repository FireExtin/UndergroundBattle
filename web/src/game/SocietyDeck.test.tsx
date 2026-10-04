import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DeckLibrary } from './DeckLibrary';
import { createDeckDraft, DECK_LIBRARY_STORAGE_KEY, readDeckLibrary, saveDeckLibrary, validateDeckDraft } from './deckLibrary';
import { fixtureSociety, fixtureSocietyCatalog as catalog } from './societyTest.fixture';

const draft = () => ({ ...createDeckDraft(catalog), societyId: fixtureSociety.id, cards: [{ cardId: 'JC125', count: 48 }, { cardId: 'FIXTURE_RED', count: 2 }] });
afterEach(() => localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY));

describe('optional society construction from a synthetic registry', () => {
  it('keeps society outside the fifty-card count and enforces explicit color constraints', () => {
    expect(validateDeckDraft(draft(), catalog)).toEqual({ valid: true, total: 50, issues: [] });
    expect(validateDeckDraft({ ...draft(), cards: [{ cardId: 'JC125', count: 47 }, { cardId: 'FIXTURE_RED', count: 2 }] }, catalog).issues).toEqual(expect.arrayContaining([expect.objectContaining({ code: 'minimum' })]));
    const lacking = validateDeckDraft({ ...draft(), cards: [{ cardId: 'JC125', count: 50 }] }, catalog);
    expect(lacking.issues.find(issue => issue.code === 'society-color')?.message).toContain('当前 0 张');
    const withSuffix = { ...catalog, societies: [{ ...fixtureSociety, deckConstraints: [{ kind: 'minimumColor' as const, color: '红色', count: 2 }] }] };
    expect(validateDeckDraft(draft(), withSuffix).valid).toBe(true);
  });
  it('rejects unknown, closed, missing-registry and ordinary-card society selections', () => {
    for (const closed of [{ ...catalog, societies: undefined }, { ...catalog, deckBuildRules: { ...catalog.deckBuildRules!, societySupported: false } }, { ...catalog, societies: [{ ...fixtureSociety, supported: false }] }]) {
      expect(validateDeckDraft(draft(), closed).issues.some(issue => issue.code === 'society')).toBe(true);
    }
    expect(validateDeckDraft({ ...draft(), societyId: 'FIXTURE_UNKNOWN' }, catalog).valid).toBe(false);
    const ordinary = { ...catalog, cards: [...catalog.cards, fixtureSociety] };
    expect(validateDeckDraft({ ...draft(), societyId: null, cards: [...draft().cards, { cardId: fixtureSociety.id, count: 1 }] }, ordinary).issues.some(issue => issue.code === 'kind')).toBe(true);
    expect(validateDeckDraft({ ...draft(), societyId: null }, { ...catalog, societies: undefined }).valid).toBe(true);
  });
  it('does not certify a society whose required construction metadata is incomplete', () => {
    expect(validateDeckDraft(draft(), { ...catalog, societies: [{ ...fixtureSociety, startingHand: NaN }] }).issues.some(issue => issue.code === 'society-metadata')).toBe(true);
    expect(validateDeckDraft(draft(), { ...catalog, societies: [{ ...fixtureSociety, deckConstraints: [{ kind: 'minimumColor', color: '红', count: -1 }] }] }).valid).toBe(false);
  });
  it('selects, saves, reloads and clears a society without changing ordinary entries', () => {
    const select = vi.fn();
    saveDeckLibrary([{ ...draft(), societyId: null }]);
    const first = render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
    expect(screen.getByText('不选择秘社时，起手 6 张。')).toBeInTheDocument();
    fireEvent.change(screen.getByRole('combobox', { name: '秘社（可选）' }), { target: { value: fixtureSociety.id } });
    const area = screen.getByRole('region', { name: '可选秘社' });
    expect(within(area).getByText('选定秘社起手 6 张。')).toBeInTheDocument();
    expect(within(area).getByText('合成副标题')).toBeInTheDocument();
    expect(within(area).getByText(/至少 2 张红色卡 · 当前 2 张/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
    fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
    expect(select).toHaveBeenCalledWith(expect.objectContaining({ societyId: fixtureSociety.id, cards: draft().cards }));
    first.unmount(); render(<DeckLibrary catalog={catalog} />);
    expect(screen.getByRole('combobox', { name: '秘社（可选）' })).toHaveValue(fixtureSociety.id);
    fireEvent.click(screen.getByRole('button', { name: '清除秘社选择' }));
    fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
    expect(readDeckLibrary().drafts[0]).toMatchObject({ societyId: null, cards: draft().cards });
  });
  it('retains obsolete choices while allowing explicit removal under an old catalog', () => {
    saveDeckLibrary([draft()]);
    render(<DeckLibrary catalog={{ ...catalog, societies: undefined, deckBuildRules: { ...catalog.deckBuildRules!, societySupported: false } }} onSelectDraft={vi.fn()} />);
    expect(screen.getByRole('combobox', { name: '秘社（可选）' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
    expect(readDeckLibrary().drafts[0].societyId).toBe(fixtureSociety.id);
    fireEvent.click(screen.getByRole('button', { name: '清除秘社选择' }));
    expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
  });
  it('shows unmet requirements immediately and keeps a disabled editor read-only', () => {
    saveDeckLibrary([{ ...draft(), cards: [{ cardId: 'JC125', count: 50 }] }]);
    render(<DeckLibrary catalog={catalog} disabled onSelectDraft={vi.fn()} />);
    expect(screen.getByRole('combobox', { name: '秘社（可选）' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '清除秘社选择' })).toBeDisabled();
    expect(screen.getByRole('list', { name: '牌组校验问题' })).toHaveTextContent('当前 0 张');
  });
});
