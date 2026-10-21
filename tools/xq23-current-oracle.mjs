// Both active XQ23 entry points share one real ABI93 Native byte archive.
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {packCoreFixture} from './xq23-oracle.mjs';
export {loadFixture as loadCurrentFixture,validateNative,compare} from './xq23-oracle.mjs';
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))await packCoreFixture(...process.argv.slice(2));
