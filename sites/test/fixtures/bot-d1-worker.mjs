// D1 host for the direct RoomService tests. Application routes are exercised
// through the real createWorker function; this fixture exposes no app route.
export default { fetch() { return new Response('D1 test fixture', {status:404}); } };
