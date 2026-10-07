// Experimental Site: retired support contract is replaced by fail-closed preservation.
import { test } from 'node:test';
import { rejectHistoricalRoom } from './fixtures/reject-historical-room.mjs';
test('historical fire-scholar fixture rejects without altering persisted data', t => rejectHistoricalRoom(t, new URL('./fixtures/prepared-fire-scholar-v028.json', import.meta.url)));
