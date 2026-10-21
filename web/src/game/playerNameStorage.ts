import { newCommandId } from './api';
const PLAYER_NAME_KEY = 'hegemony.playerName.v1';

export function readPlayerName(): string {
  try { return (localStorage.getItem(PLAYER_NAME_KEY) || '').trim().slice(0, 24); }
  catch { return ''; }
}

/** Resolve once per submission; callers retain the result for receipt retries. */
export function rememberPlayerName(value: string): string {
  const name = value.trim().slice(0, 24) || `旅人·${newCommandId().slice(0, 6).toUpperCase()}`;
  try { localStorage.setItem(PLAYER_NAME_KEY, name); } catch { /* The current entry still uses this name. */ }
  return name;
}
