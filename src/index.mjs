import { kernel } from './kernel.mjs';
import { HttpError, RoomService } from './service.mjs';
import { storageFailure } from './storage-errors.mjs';

const json = (body, status = 200) => new Response(JSON.stringify(body), {
  status, headers: { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store' },
});
async function body(request) {
  if (Number(request.headers.get('content-length')) > 65536) throw new HttpError(413, '请求过大');
  const text = await request.text();
  if (new TextEncoder().encode(text).length > 65536) throw new HttpError(413, '请求过大');
  try {
    const value = JSON.parse(text);
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error();
    return value;
  } catch { throw new HttpError(400, '请求格式不正确'); }
}
export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (!url.pathname.startsWith('/api/')) return env.ASSETS.fetch(request);
    try {
      if (request.method === 'GET' && url.pathname === '/api/health') return json({ ok: true, service: 'hegemony-worker', engineVersion: JSON.parse(kernel.catalog()).engineVersion, transport: 'polling' });
      if (request.method === 'GET' && url.pathname === '/api/catalog') return json({ ...JSON.parse(kernel.catalog()), transport: 'polling', entryIdempotency: true });
      // Ordinary DB binding queries go to the primary; no read-replica Sessions are used.
      const service = new RoomService(env.DB, kernel);
      if (request.method === 'POST' && url.pathname === '/api/rooms') return json(await service.create(await body(request)));
      if (request.method === 'POST' && url.pathname === '/api/rooms/join') return json(await service.join(await body(request)));
      const match = url.pathname.match(/^\/api\/rooms\/([a-f0-9]{24})\/(state|commands|events|catalog)$/);
      if (!match) throw new HttpError(404, '接口不存在');
      const [, id, operation] = match;
      if (operation === 'catalog' && request.method === 'GET') return json({ ...await service.catalog(id, request.headers.get('authorization')), transport: 'polling', entryIdempotency: true });
      if (operation === 'state' && request.method === 'GET') {
        const view = await service.state(id, request.headers.get('authorization'), url.searchParams.get('afterVersion'));
        return view ? json(view) : new Response(null, { status: 204, headers: { 'Cache-Control': 'no-store' } });
      }
      if (operation === 'commands' && request.method === 'POST') return json(await service.command(id, request.headers.get('authorization'), await body(request)));
      if (operation === 'events') throw new HttpError(410, '本服务按版本轮询同步，请刷新客户端。');
      throw new HttpError(405, '请求方法不支持');
    } catch (error) {
      if (error instanceof HttpError) return json({ error: error.status === 401 ? 'invalid_seat_token' : 'request_failed', message: error.message, ...(error.view ? { view: error.view } : {}) }, error.status);
      // No room state, command body, token, invite, or database error details are logged.
      console.error('hegemony persistence operation failed', storageFailure(error));
      return json({ error: 'storage_error', message: '牌桌服务暂不可用，未确认此操作。请使用原请求重试。' }, 503);
    }
  },
};
