import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Help } from './Help';
import { testCatalog } from './testFixtures';

describe('help keyboard dismissal', () => {
  it('closes with Escape and returns focus to the guide opener', () => {
    const opener = document.createElement('button'); document.body.append(opener); opener.focus();
    const close = vi.fn();
    const { unmount } = render(<Help catalog={testCatalog} onClose={close} />);
    expect(screen.getByRole('button', { name: '关闭指南' })).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Escape' }); expect(close).toHaveBeenCalledOnce();
    unmount(); expect(opener).toHaveFocus(); opener.remove();
    fireEvent.keyDown(document, { key: 'Escape' }); expect(close).toHaveBeenCalledOnce();
  });
});
