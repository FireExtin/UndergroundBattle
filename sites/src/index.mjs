import { kernel } from './kernel.mjs';
import { createWorker } from './http.mjs';

export default createWorker(kernel);
