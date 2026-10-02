import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Lobby } from './Lobby';
import { testCatalog } from './testFixtures';

afterEach(() => history.replaceState({}, '', '/'));
describe('Chinese invitation lobby', () => {
  it('lets a player choose a 50-card deck and create a four-seat team room', () => {
    const create = vi.fn(); const catalog = { ...testCatalog, decks: [...testCatalog.decks, { ...testCatalog.decks[0], id: 'hunters', name: '公路猎手' }] };
    render(<Lobby catalog={catalog} busy={false} onCreate={create} onJoin={vi.fn()} retry={vi.fn()} />);
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '队长' } });
    fireEvent.click(screen.getByRole('button', { name: /公路猎手/ }));
    fireEvent.click(screen.getByRole('button', { name: /四人协作/ }));
    fireEvent.click(screen.getByRole('button', { name: '创建牌桌 →' }));
    expect(create).toHaveBeenCalledWith('队长', 'teams', 'hunters');
    expect(screen.getByText(/并非官方四套预组/)).toBeInTheDocument();
  });
  it('opens invitation links directly in join mode and submits only a room code', () => {
    history.replaceState({}, '', '/?invite=ROOM-ONLY'); const join = vi.fn();
    render(<Lobby catalog={testCatalog} busy={false} onCreate={vi.fn()} onJoin={join} retry={vi.fn()} />);
    expect(screen.getByRole('textbox', { name: '邀请码' })).toHaveValue('ROOM-ONLY');
    fireEvent.change(screen.getByRole('textbox', { name: '你的称呼' }), { target: { value: '伙伴' } });
    fireEvent.click(screen.getByRole('button', { name: '加入牌桌 →' }));
    expect(join).toHaveBeenCalledWith('ROOM-ONLY', '伙伴', 'watchers');
    expect(location.href).not.toContain('token');
  });
});
