import {loadFixture,compare} from '../../tools/xq23-oracle.mjs';
import {kernel,moduleBytes} from './current88-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
