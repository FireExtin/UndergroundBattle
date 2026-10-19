import {loadFixture,compare} from '../../tools/jc035-oracle.mjs';
import {kernel,moduleBytes} from './current89-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
