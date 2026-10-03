import { useLayoutEffect, useRef } from 'react';
import type { RefObject } from 'react';

const focusable = 'button, input, select, textarea, a[href], summary, [tabindex]';

/** The owning component chooses the active layer; no global modal stack is needed. */
export function useDialogFocus(dialog: RefObject<HTMLElement | null>, active: boolean, onEscape?: () => void) {
  const previous = useRef<HTMLElement | null>(null);
  const captured = useRef(false);
  const lastFocus = useRef<HTMLElement | null>(null);
  const dismiss = useRef(onEscape);
  dismiss.current = onEscape;

  useLayoutEffect(() => {
    if (!active || !dialog.current) return;
    const root = dialog.current;
    if (!captured.current) {
      previous.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      captured.current = true;
    }
    const saved = new Map<HTMLElement, { inert: string | null; hidden: string | null }>();
    const parents: HTMLElement[] = [];
    let branch: HTMLElement = root;
    while (branch.parentElement) {
      parents.push(branch.parentElement);
      branch = branch.parentElement;
      if (branch === document.body) break;
    }
    const isolate = () => {
      let current: HTMLElement = root;
      for (const parent of parents) {
        for (const sibling of parent.children) {
          if (sibling === current || !(sibling instanceof HTMLElement) || saved.has(sibling)) continue;
          saved.set(sibling, { inert: sibling.getAttribute('inert'), hidden: sibling.getAttribute('aria-hidden') });
          sibling.setAttribute('inert', '');
          sibling.setAttribute('aria-hidden', 'true');
        }
        current = parent;
      }
    };
    const controls = () => Array.from(root.querySelectorAll<HTMLElement>(focusable)).filter(element =>
      element.tabIndex >= 0 && !element.matches(':disabled') && !element.closest('[inert], [hidden], [aria-hidden="true"]')
      && getComputedStyle(element).display !== 'none' && getComputedStyle(element).visibility !== 'hidden');
    const focusInside = () => {
      const available = controls();
      const target = lastFocus.current && available.includes(lastFocus.current) ? lastFocus.current : available[0] || root;
      target.focus();
    };
    const keydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault(); event.stopPropagation();
        dismiss.current?.();
      } else if (event.key === 'Tab') {
        const available = controls();
        const index = available.indexOf(document.activeElement as HTMLElement);
        event.preventDefault(); event.stopPropagation();
        const next = index < 0 ? (event.shiftKey ? available.length - 1 : 0)
          : (index + (event.shiftKey ? -1 : 1) + available.length) % available.length;
        (available[next] || root).focus();
      }
    };
    const focusin = (event: FocusEvent) => {
      if (event.target instanceof HTMLElement && root.contains(event.target)) lastFocus.current = event.target;
      else focusInside();
    };
    // Focus the dialog before hiding its opener from the accessibility tree.
    if (!root.contains(document.activeElement)) focusInside();
    isolate();
    const observer = new MutationObserver(isolate);
    parents.forEach(parent => observer.observe(parent, { childList: true }));
    document.addEventListener('keydown', keydown, true);
    document.addEventListener('focusin', focusin, true);
    return () => {
      observer.disconnect();
      document.removeEventListener('keydown', keydown, true);
      document.removeEventListener('focusin', focusin, true);
      saved.forEach((attributes, element) => {
        if (attributes.inert === null) element.removeAttribute('inert'); else element.setAttribute('inert', attributes.inert);
        if (attributes.hidden === null) element.removeAttribute('aria-hidden'); else element.setAttribute('aria-hidden', attributes.hidden);
      });
    };
  }, [active, dialog]);

  // Pausing for a nested reader releases isolation without moving focus back to the table.
  useLayoutEffect(() => () => { if (previous.current?.isConnected) previous.current.focus(); }, []);
}
