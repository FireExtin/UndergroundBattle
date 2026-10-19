import test from 'node:test';
import {loadCurrentFixture,compare} from '../../tools/xq23-current-oracle.mjs';
import {kernel,moduleBytes} from './current89-kernel.mjs';
test('current89 executes fresh Native141 XQ23 trajectories and all31 restore refusals without re-signing88',t=>{
 t.diagnostic(JSON.stringify({...compare(loadCurrentFixture(),kernel,moduleBytes),nativeInputs:141}));
});
