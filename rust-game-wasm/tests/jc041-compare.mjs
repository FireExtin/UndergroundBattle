import {loadFixture,compare} from '../../tools/jc041-oracle.mjs';
import {kernel,moduleBytes} from './current-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
