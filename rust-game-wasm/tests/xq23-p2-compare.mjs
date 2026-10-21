import {loadP2Fixture,compareP2} from '../../tools/xq23-p2-oracle.mjs';
import {kernel,moduleBytes} from './current-kernel.mjs';
console.log(JSON.stringify(compareP2(loadP2Fixture(),kernel,moduleBytes)));
