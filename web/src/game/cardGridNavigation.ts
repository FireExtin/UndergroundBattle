import type { KeyboardEvent } from 'react';

/** Navigate the rendered card rows, including resized grids and an incomplete last row. */
export function moveCardGridFocus(event: KeyboardEvent<HTMLButtonElement>, grid: HTMLElement | null, selector = '.hg-library-card-preview') {
  if (!grid || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
  const keys = ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End'];
  if (!keys.includes(event.key)) return;
  const cards = Array.from(grid.querySelectorAll<HTMLButtonElement>(selector)).filter(card => !card.disabled);
  const index = cards.indexOf(event.currentTarget);
  if (index < 0) return;
  let target = event.key === 'Home' ? cards[0] : event.key === 'End' ? cards.at(-1)
    : event.key === 'ArrowLeft' ? cards[index - 1] : event.key === 'ArrowRight' ? cards[index + 1] : undefined;
  if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
    const current = event.currentTarget.getBoundingClientRect();
    const direction = event.key === 'ArrowDown' ? 1 : -1;
    const candidates = cards.map(card => ({ card, rect: card.getBoundingClientRect() }))
      .filter(({ rect }) => (rect.top - current.top) * direction > 1)
      .sort((a, b) => Math.abs(a.rect.top - current.top) - Math.abs(b.rect.top - current.top)
        || Math.abs(a.rect.left + a.rect.width / 2 - current.left - current.width / 2)
        - Math.abs(b.rect.left + b.rect.width / 2 - current.left - current.width / 2));
    target = candidates[0]?.card;
  }
  event.preventDefault();
  if (target) {
    target.focus({ preventScroll: true });
    target.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }
}
