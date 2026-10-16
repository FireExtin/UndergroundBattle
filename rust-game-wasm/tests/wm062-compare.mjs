import { loadFixture, compare } from '../../tools/wm062-oracle.mjs';
import { kernel, moduleBytes } from './current80-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
