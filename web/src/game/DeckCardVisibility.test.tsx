import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DeckCardFace } from './DeckCardFace';

const card = { id: 'JC125', name: '无知路人', kind: 'character', cost: 0, text: '' };
afterEach(() => vi.unstubAllGlobals());

describe('viewport-gated original card images', () => {
  it('provides an image source only while the card is visible or near the viewport', () => {
    let notify: IntersectionObserverCallback;
    const observe = vi.fn(), unobserve = vi.fn(), disconnect = vi.fn();
    vi.stubGlobal('IntersectionObserver', class {
      constructor(callback: IntersectionObserverCallback) { notify = callback; }
      observe = observe; unobserve = unobserve; disconnect = disconnect;
    });
    const ui = render(<DeckCardFace card={card} />);
    const face = ui.container.querySelector('[data-deck-face]')!;
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(face).toHaveAttribute('data-face-state', 'deferred');
    act(() => notify([{ target: face, isIntersecting: true } as IntersectionObserverEntry], {} as IntersectionObserver));
    expect(screen.getByRole('img')).toHaveAttribute('src', '/cards/JC125.jpg');
    act(() => notify([{ target: face, isIntersecting: false } as IntersectionObserverEntry], {} as IntersectionObserver));
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    ui.unmount();
    expect(unobserve).toHaveBeenCalledWith(face);
    expect(disconnect).toHaveBeenCalledOnce();
  });

  it('loads an explicitly opened large preview immediately and preserves failed-image fallback', () => {
    vi.stubGlobal('IntersectionObserver', class { observe() {} unobserve() {} disconnect() {} });
    render(<DeckCardFace card={card} large />);
    const image = screen.getByRole('img', { name: '无知路人原始牌面' });
    expect(image).toHaveAttribute('loading', 'eager');
    fireEvent.error(image);
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(screen.getByText('暂无原始牌面')).toBeInTheDocument();
  });
});
