import test from 'node:test';
import {rejectHistoricalRoom} from './fixtures/reject-historical-room.mjs';

test('ABI73 rejects a genuine source-built ABI72 lobby before operations and preserves D1 after reopen',
  t=>rejectHistoricalRoom(t,new URL('./fixtures/source-built-native72-lobby.json',import.meta.url)));
