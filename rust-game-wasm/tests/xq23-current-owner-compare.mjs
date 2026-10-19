import test from 'node:test';
import {loadOwnerFixture,compareOwner} from '../../tools/xq23-current-owner-oracle.mjs';
import {kernel,moduleBytes} from './current89-kernel.mjs';
test('current89 independently re-executes the complete actual Native141 owner-elimination trajectory',t=>{
 t.diagnostic(JSON.stringify(compareOwner(loadOwnerFixture(),kernel,moduleBytes)));
});
