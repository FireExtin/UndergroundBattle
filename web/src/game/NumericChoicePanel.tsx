import { useRef, useState } from 'react';
import type { ReactNode } from 'react';
import type { Action, Choice, LegalAction } from './types';
import { useDialogFocus } from './useDialogFocus';

/** Numeric Pending remains tied to the server's original choice and bounds. */
export function NumericChoicePanel({ choice, action, busy, onSubmit, modal = false, modalActive = true, onPauseRoom, recoveryNotices }: {
  choice: Choice; action?: LegalAction; busy: boolean; onSubmit: (a: Action) => void;
  modal?: boolean; modalActive?: boolean; onPauseRoom?: () => void; recoveryNotices?: ReactNode;
}) {
  const dialog = useRef<HTMLElement>(null);
  const [text, setText] = useState('');
  const amount = Number(text);
  const range = action?.amountRange;
  const valid = text.trim() !== '' && Number.isSafeInteger(amount) && !!range && amount >= range.min && amount <= range.max;
  useDialogFocus(dialog, modal && modalActive);
  return <section ref={dialog} className="hg-choice" data-choice-id={choice.id} role={modal ? 'dialog' : undefined} aria-modal={modal ? true : undefined} aria-label="待完成的选择" tabIndex={-1}>
    {recoveryNotices}
    {onPauseRoom && <button type="button" className="hg-button hg-button-quiet" disabled={busy} onClick={onPauseRoom}>暂停并保存此桌</button>}
    <h2>{choice.title}</h2>
    <p>输入要限制的印刷费用，包括0。此选择结算前，禁令尚未开始限制该费用。</p>
    <label>印刷费用<input type="number" aria-label="禁令印刷费用" min={range?.min} max={range?.max} step={1} value={text} disabled={busy} onChange={e => setText(e.target.value)} /></label>
    <button type="button" className="hg-button hg-button-action" disabled={busy || !valid || !action}
      onClick={() => { if (valid && action && !busy) onSubmit({ ...action, choiceId: choice.id, selected: [], amount }); }}>确认费用</button>
  </section>;
}
