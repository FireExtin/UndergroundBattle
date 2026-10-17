import {loadFixture,compare} from '../../tools/msjc0405-oracle.mjs';
import {kernel,moduleBytes} from './current83-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
