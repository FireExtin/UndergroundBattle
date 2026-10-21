// Recorded opaque legacy lobbies and routing identities, not executable kernels.
// Preserve forty captured records and fifteen archived headers unchanged.
// The former current ABI93 is an additional identity-only rejection sample.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
const fixture = JSON.parse(readFileSync(new URL('./historical-rooms.json', import.meta.url), 'utf8'));
const digest = value => createHash('sha256').update(value).digest('hex');
export const retiredIdentities = [...fixture.retiredKernelIdentities, JSON.parse(readFileSync(new URL('./retired-abi93-header.json', import.meta.url), 'utf8'))];
export const historicalIdentities = [...fixture.kernels, ...retiredIdentities];
export function historicalLobbies() {
  for (const row of fixture.lobbies) {
    assert.equal(digest(row.room.state), row.stateSha256);
    assert.equal(digest(JSON.stringify(row.room.view)), row.viewSha256);
  }
  return fixture.lobbies;
}
