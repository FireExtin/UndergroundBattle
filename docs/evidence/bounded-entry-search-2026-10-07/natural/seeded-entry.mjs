// LOCAL QA ONLY: fixed factory RNG seed; never modify a room or command state.
// Delegate all routes, storage and rules to the exact published compiled Worker.
import publishedWorker from '../backend/candidate-worker/dist/index.js';

const originalRandom = crypto.getRandomValues.bind(crypto);
let nextFactorySeed = null;
let consumedFactorySeed = null;
Object.defineProperty(crypto, 'getRandomValues', { value(array) {
  if (nextFactorySeed !== null && array.byteLength === 8) {
    const hex = BigInt(nextFactorySeed).toString(16).padStart(16, '0');
    for (let i = 0; i < 8; i++) array[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    consumedFactorySeed = nextFactorySeed;
    nextFactorySeed = null;
    return array;
  }
  return originalRandom(array);
}});

export default {
  async fetch(request, env, context) {
    const create = request.method === 'POST' && new URL(request.url).pathname === '/api/rooms';
    const seed = create ? request.headers.get('X-Local-QA-Factory-Seed') : null;
    if (seed !== null && !/^(?:[1-9]|[1-5][0-9]|6[0-4])$/.test(seed)) {
      return new Response('Local QA factory seed must be1..64', { status: 400 });
    }
    if (!create || seed === null) return publishedWorker.fetch(request, env, context);
    nextFactorySeed = seed;
    consumedFactorySeed = null;
    try {
      const result = await publishedWorker.fetch(request, env, context);
      const response = new Response(result.body, result);
      if (consumedFactorySeed !== null) response.headers.set('X-Local-QA-Factory-Seed-Applied', consumedFactorySeed);
      return response;
    } finally {
      nextFactorySeed = null;
      consumedFactorySeed = null;
    }
  },
};
