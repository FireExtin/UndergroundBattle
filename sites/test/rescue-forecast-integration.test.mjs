// Experimental Site: retired support contract is replaced by fail-closed preservation.
import { test } from 'node:test';
import { rejectHistoricalRoom } from './fixtures/reject-historical-room.mjs';
test('historical rescue-forecast fixture rejects without altering persisted data', t => rejectHistoricalRoom(t, new URL('./fixtures/prepared-rescue-forecast-v028.json', import.meta.url)));
