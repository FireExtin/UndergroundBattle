import { compareBq035Fixture, loadBq035Fixture } from '../../tools/bq035-oracle.mjs';
import { kernel, moduleBytes } from './current-kernel.mjs';
console.log(JSON.stringify(compareBq035Fixture(loadBq035Fixture(process.argv[2]), kernel, moduleBytes)));
