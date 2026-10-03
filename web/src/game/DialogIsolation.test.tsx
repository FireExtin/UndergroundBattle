import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Help } from './Help';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Choice, View } from './types';

const mulligan: Choice = {
  id: 'modal-mulligan', kind: 'mulligan', title: '起手再调度', description: '选牌后明确确认',
  playerId: 'p0', min: 0, max: 1, allowDecline: true,
  options: [{ id: testCard.instanceId, label: testCard.name, card: testCard }],
};
const choose = { id: 'modal-choose', kind: 'choose', label: '选择', choiceId: mulligan.id };
const playing: View = { ...testView, status: 'playing', legalActions: [] };
const choosing: View = { ...playing, pendingChoice: mulligan, legalActions: [choose] };

describe('table choice modal isolation', () => {
  it('removes background AutoPass and navigation from the accessible tree and confines Tab in both directions', () => {
    const submit = vi.fn();
    const content = (view: View) => <div><header aria-hidden="false"><button>牌桌导航</button></header><Table view={view} catalog={testCatalog} busy={false} onAction={submit} /></div>;
    const { rerender, container } = render(content(playing));
    const autoPass = screen.getByRole('switch', { name: '无可用行动时自动让过' });
    autoPass.focus();
    rerender(content(choosing));
    expect(screen.queryByRole('switch', { name: '无可用行动时自动让过' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '牌桌导航' })).not.toBeInTheDocument();
    expect(autoPass.closest('[inert]')).not.toBeNull();
    const dialog = screen.getByRole('dialog', { name: '待完成的选择' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog.contains(document.activeElement)).toBe(true);
    const confirm = within(dialog).getByRole('button', { name: '确认选择' });
    confirm.focus(); fireEvent.keyDown(document, { key: 'Tab' });
    expect(within(dialog).getByRole('region', { name: '选择选项，可滚动查看' })).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Tab', shiftKey: true });
    expect(confirm).toHaveFocus();
    autoPass.focus();
    expect(dialog.contains(document.activeElement)).toBe(true);
    expect(autoPass).not.toBeChecked(); expect(submit).not.toHaveBeenCalled();
    rerender(content(playing));
    expect(autoPass).toHaveFocus();
    expect(autoPass.closest('[inert]')).toBeNull();
    expect(container.querySelector('header')).toHaveAttribute('aria-hidden', 'false');
    expect(screen.getByRole('button', { name: '牌桌导航' })).toBeInTheDocument();
  });

  it('keeps Escape inert and submits zero-selection keep only through the explicit button', () => {
    const submit = vi.fn();
    render(<Table view={choosing} catalog={testCatalog} busy={false} onAction={submit} />);
    const dialog = screen.getByRole('dialog', { name: '待完成的选择' });
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(dialog).toBeInTheDocument(); expect(submit).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: '保留全部手牌' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: [] });
  });

  it('makes the nested reader the sole active layer and returns to its choice opener without losing selection', () => {
    const submit = vi.fn();
    render(<Table view={choosing} catalog={testCatalog} busy={false} onAction={submit} />);
    const choiceDialog = screen.getByRole('dialog', { name: '待完成的选择' });
    const option = choiceDialog.querySelector<HTMLButtonElement>('[data-choice-option]')!;
    fireEvent.click(option);
    const opener = within(choiceDialog).getByRole('button', { name: '放大阅读无知路人' });
    opener.focus(); fireEvent.click(opener);
    const reader = screen.getByRole('dialog', { name: '放大阅读无知路人' });
    expect(reader.closest('[inert]')).toBeNull();
    expect(choiceDialog.closest('[inert]')).not.toBeNull();
    expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
    expect(within(reader).getByRole('button', { name: '关闭放大阅读' })).toHaveFocus();
    fireEvent.click(within(reader).getByRole('button', { name: '原始牌面' }));
    expect(within(reader).getByRole('img', { name: '无知路人原始牌面' })).toHaveAttribute('src', '/cards/JC125.jpg');
    for (let i = 0; i < 12; i++) {
      fireEvent.keyDown(document, { key: 'Tab', shiftKey: i % 2 === 0 });
      expect(reader.contains(document.activeElement)).toBe(true);
    }
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: '放大阅读无知路人' })).not.toBeInTheDocument();
    expect(opener).toHaveFocus(); expect(choiceDialog.closest('[inert]')).toBeNull();
    expect(option).toHaveAttribute('aria-pressed', 'true');
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(choiceDialog).toBeInTheDocument(); expect(submit).not.toHaveBeenCalled();
    fireEvent.click(within(choiceDialog).getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...choose, selected: [testCard.instanceId] });
  });

  it('keeps a late-arriving choice beneath the open guide and restores the choice when the guide closes', async () => {
    const content = (view: View, help: boolean) => <div><header><button>打开指南</button></header><Table key={view.version} view={view} catalog={testCatalog} busy={false} modalPaused={help} onAction={vi.fn()} />{help && <Help catalog={testCatalog} onClose={vi.fn()} />}</div>;
    const { rerender } = render(content(playing, false));
    const opener = screen.getByRole('button', { name: '打开指南' }); opener.focus();
    rerender(content(playing, true));
    const updated = { ...choosing, version: 2 };
    rerender(content(updated, true));
    const guide = screen.getByRole('dialog', { name: '上手指南' });
    expect(guide.closest('[inert]')).toBeNull();
    await waitFor(() => expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument());
    for (let i = 0; i < 6; i++) { fireEvent.keyDown(document, { key: 'Tab' }); expect(guide.contains(document.activeElement)).toBe(true); }
    rerender(content(updated, false));
    const choiceDialog = screen.getByRole('dialog', { name: '待完成的选择' });
    expect(choiceDialog.closest('[inert]')).toBeNull();
    expect(choiceDialog.contains(document.activeElement)).toBe(true);
  });

  it('leaves normal browsing accessible when another seat is making a choice', () => {
    render(<Table view={{ ...playing, waitingChoice: { playerId: 'p1', kind: 'mulligan', title: '起手再调度' } }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByRole('switch', { name: '无可用行动时自动让过' }).closest('[inert]')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: '查看无知路人' }));
    expect(screen.getByRole('complementary', { name: '选牌行动' })).toBeInTheDocument();
  });
});
