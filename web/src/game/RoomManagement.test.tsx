import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { RoomManagement } from './RoomManagement';
import { testView } from './testFixtures';

afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it('offers pause and resume as distinct confirmed server commands, gated by busy state', () => {
  const onAction = vi.fn(), refresh = vi.fn();
  const view = { ...testView, canPause: true };
  const { rerender } = render(<RoomManagement view={view} blocked={false} onAction={onAction} refresh={refresh} />);
  fireEvent.click(screen.getByRole('button', { name: '暂停并保存' }));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ kind: 'pauseRoom' });
  rerender(<RoomManagement view={{ ...view, pause: { pausedAtMs: 2000, pausedBy: 1 } }} blocked={true} onAction={onAction} refresh={refresh} />);
  fireEvent.click(screen.getByRole('button', { name: '恢复对局' })); expect(onAction).toHaveBeenCalledTimes(1);
  rerender(<RoomManagement view={{ ...view, pause: { pausedAtMs: 2000, pausedBy: 1 } }} blocked={false} onAction={onAction} refresh={refresh} />);
  fireEvent.click(screen.getByRole('button', { name: '恢复对局' })); expect(onAction).toHaveBeenLastCalledWith({ kind: 'resumeRoom' });
  fireEvent.click(screen.getByRole('button', { name: '查看最新状态' })); expect(refresh).toHaveBeenCalledOnce();
});
it('copies only the room locator and never a seat capability', async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: copy } });
  render(<RoomManagement view={{ ...testView, canPause: true, inviteCode: 'SAME-TABLE' }} blocked={false} onAction={vi.fn()} refresh={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '复制此桌邀请链接' }));
  await waitFor(() => expect(copy).toHaveBeenCalledExactlyOnceWith(`${location.origin}/?invite=SAME-TABLE`));
  expect(screen.getByText('此桌邀请链接已复制')).toBeInTheDocument();
});
it('does not offer pause against a kernel without that capability', () => {
  render(<RoomManagement view={testView} blocked={false} onAction={vi.fn()} refresh={vi.fn()} />);
  expect(screen.queryByRole('button')).not.toBeInTheDocument();
});
