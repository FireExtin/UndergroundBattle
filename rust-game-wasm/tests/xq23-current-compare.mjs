import test from 'node:test';
import {loadCurrentFixture,compare} from '../../tools/xq23-current-oracle.mjs';
import {kernel,moduleBytes} from './current-kernel.mjs';
test('current93 executes fresh Native93 XQ23 trajectories and all31 restore refusals without rewriting historical states',t=>{
 const f=loadCurrentFixture();
 t.diagnostic(JSON.stringify({...compare(f,kernel,moduleBytes),nativeInputs:Object.keys(f.sourceBindings.files).length}));
});
