import { loadFixture, compare } from '../../tools/jc054-oracle.mjs';
import { kernel, moduleBytes } from './current81-kernel.mjs';
console.log(JSON.stringify(compare(loadFixture(),kernel,moduleBytes)));
