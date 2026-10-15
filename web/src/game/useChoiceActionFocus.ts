import { useLayoutEffect, useRef } from 'react';
import type { RefObject } from 'react';

type EditFocus = { control: HTMLElement; rowAttribute: string; optionId: string; controlIndex: number };
const rowSelector = '[data-choice-order-option], [data-choice-damage-option]';
const editControls = (row: HTMLElement) => Array.from(row.querySelectorAll<HTMLElement>('button, input'))
  .filter(control => !control.matches('.hg-order-read, .hg-small-read'));
const available = (control: HTMLElement) => !control.matches(':disabled')
  && !control.closest('[inert], [hidden], [aria-hidden="true"]');

/** Keep an edit's focus with its candidate when its control becomes disabled or moves piles. */
export function useChoiceActionFocus(panel: RefObject<HTMLElement | null>, active: boolean, editState: unknown) {
  const pending = useRef<EditFocus | null>(null);

  useLayoutEffect(() => {
    const previous = pending.current;
    pending.current = null;
    const root = panel.current;
    if (!previous || !active || !root || !available(root)) return;
    const current = document.activeElement;
    if (current === previous.control && available(previous.control)) return;
    if (current && current !== document.body && current !== previous.control) return;
    const row = Array.from(root.querySelectorAll<HTMLElement>(`[${previous.rowAttribute}]`))
      .find(element => element.getAttribute(previous.rowAttribute) === previous.optionId);
    const controls = row ? editControls(row) : [];
    const matching = controls[previous.controlIndex];
    const target = matching && available(matching) ? matching : controls.find(available) || root;
    target.focus({ preventScroll: true });
    target.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }, [active, editState, panel]);

  return (changed: boolean) => {
    // A repeated transfer can be a no-op in the same batch as the first edit.
    // Keep that edit's pending recovery until its render removes the old control.
    if (!changed) return;
    pending.current = null;
    const control = document.activeElement;
    if (!active || !(control instanceof HTMLElement) || !panel.current?.contains(control)) return;
    const row = control.closest<HTMLElement>(rowSelector);
    if (!row) return;
    const controlIndex = editControls(row).indexOf(control);
    if (controlIndex < 0) return;
    const rowAttribute = row.hasAttribute('data-choice-order-option') ? 'data-choice-order-option' : 'data-choice-damage-option';
    pending.current = { control, controlIndex, rowAttribute, optionId: row.getAttribute(rowAttribute)! };
  };
}
