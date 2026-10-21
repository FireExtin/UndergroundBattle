// Fresh current probe; the original ABI87 opaque archive stays untouched.
import {loadP2Fixture,compareP2} from '../../tools/jc041-oracle.mjs';
import {kernel,moduleBytes} from './current-kernel.mjs';
console.log(JSON.stringify(compareP2(loadP2Fixture(),kernel,moduleBytes)));
