// Historical timed/composing/choice layouts remain opaque and unsupported.
import { test } from 'node:test';
import { rejectHistoricalRoom } from './fixtures/reject-historical-room.mjs';
for (const minor of [5,6,7,8]) test(`historical response${minor} rejects before pacing and survives reopen byte-for-byte`, t => rejectHistoricalRoom(t, new URL(`./fixtures/prepared-response-v02${minor}.json`, import.meta.url)));
