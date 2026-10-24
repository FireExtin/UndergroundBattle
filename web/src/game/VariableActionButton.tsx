import { useEffect, useState } from 'react';
import type { Action, LegalAction } from './types';

/** A bounded server-authored numeric cost; the server rechecks X on submit. */
export function VariableActionButton({ action, label, disabled, onAction }: {
  action: LegalAction; label: string; disabled?: boolean; onAction: (a: Action) => void;
}) {
  const range = action.amountRange!;
  const [value, setValue] = useState(String(action.amount ?? range.min));
  useEffect(() => setValue(String(action.amount ?? range.min)), [action.id, action.amount, range.min, range.max]);
  const amount = Number(value);
  const valid = value.trim() !== '' && Number.isSafeInteger(amount) && amount >= range.min && amount <= range.max;
  return <div className="hg-variable-action">
    <label>X（{range.min}～{range.max}）<input type="number" aria-label="选择X" min={range.min} max={range.max} step={1}
      value={value} disabled={disabled} onChange={e => setValue(e.target.value)} /></label>
    <button type="button" className="hg-button hg-button-action" data-action-id={action.id} data-action-kind={action.kind}
      disabled={disabled || !valid} onClick={() => onAction({ ...action, amount })}>{label} · X={valid ? amount : '请选择'}</button>
  </div>;
}
