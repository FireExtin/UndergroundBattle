import test from 'node:test';
import {loadOwnerFixture,compareOwner} from '../../tools/xq23-current-owner-oracle.mjs';
import {kernel,moduleBytes} from './current-kernel.mjs';
test('current93 independently re-executes the complete actual Native93 owner-elimination trajectory',t=>{
 t.diagnostic(JSON.stringify(compareOwner(loadOwnerFixture(),kernel,moduleBytes)));
});
