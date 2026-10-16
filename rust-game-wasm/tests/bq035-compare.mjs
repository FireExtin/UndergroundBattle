import { compareBq035Fixture, loadBq035Fixture } from '../../tools/bq035-oracle.mjs';
import { kernel, moduleBytes } from './current78-kernel.mjs';
console.log(JSON.stringify(compareBq035Fixture(loadBq035Fixture(process.argv[2]), kernel, moduleBytes)));
