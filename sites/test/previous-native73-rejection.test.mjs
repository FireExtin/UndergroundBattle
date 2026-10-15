import test from 'node:test';
import {rejectHistoricalRoom} from './fixtures/reject-historical-room.mjs';
test('ABI77 rejects a genuine source-built ABI73 lobby and preserves every D1 table after reopen',t=>rejectHistoricalRoom(t,new URL('./fixtures/source-built-native73-lobby.json',import.meta.url)));
