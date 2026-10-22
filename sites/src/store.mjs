// D1 alone arbitrates room writes. No isolate-local mutex or authoritative cache.
export class RoomStore {
  constructor(db) { this.db = db; }
  stmt(sql, ...args) { return this.db.prepare(sql).bind(...args); }
  room(id) { return this.stmt('SELECT * FROM rooms WHERE id=?', id).first(); }
  version(id) { return this.stmt('SELECT version FROM rooms WHERE id=?', id).first(); }
  byInvite(invite) { return this.stmt('SELECT * FROM rooms WHERE invite=?', invite).first(); }
  seat(id, hash) { return this.stmt('SELECT seat FROM seats WHERE room_id=? AND token_hash=?', id, hash).first(); }
  receipt(id, commandId) {
    return this.stmt('SELECT intent_hash,response FROM commands WHERE room_id=? AND command_id=?', id, commandId).first();
  }
  async publicPasses(id, version) {
    // Metadata is committed in the existing journal CAS transaction. Older rows
    // remain unchanged; no schema migration or opaque-state rewrite is needed.
    const rows = await this.stmt("SELECT version,coalesce(json_extract(entry,'$.SessionEvents.publicPasses'),json_extract(entry,'$.Command.publicPasses')) AS events FROM journal WHERE room_id=? AND version<=? AND (json_type(entry,'$.SessionEvents.publicPasses')='array' OR json_type(entry,'$.Command.publicPasses')='array') ORDER BY version DESC LIMIT 50", id, version).all();
    return rows.results.reverse().flatMap(row => JSON.parse(row.events).map(record => ({ record, version: row.version }))).slice(-50);
  }
  entryReceipt(requestHash) { return this.stmt('SELECT intent_hash,response FROM entry_receipts WHERE request_hash=?', requestHash).first(); }
  async botState(id, version = Number.MAX_SAFE_INTEGER) {
    // Immutable genesis authorizes Bot mode. Later policy checkpoints share the
    // exact CAS transaction with the accepted Rust command; state stays opaque.
    return this.stmt("SELECT (SELECT json_extract(entry,'$.SessionEvents.botState') FROM journal WHERE room_id=? AND version=0) AS genesis, (SELECT json_extract(entry,'$.SessionEvents.botState') FROM journal WHERE room_id=? AND version<=? AND json_type(entry,'$.SessionEvents.botState')='object' ORDER BY version DESC LIMIT 1) AS current", id, id, version).first();
  }
  async create({ id, invite, state, initialState = state, version = 0, entries = [], nonce, tokenHash, requestHash, intentHash, response }) {
    await this.db.batch([
      this.stmt('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)', id, invite, initialState, state, version, nonce),
      this.stmt('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,0,?)', id, tokenHash),
      this.stmt('INSERT INTO entry_receipts(request_hash,intent_hash,response,room_id) VALUES(?,?,?,?)', requestHash, intentHash, response, id),
      ...entries.map(e => this.stmt('INSERT INTO journal(room_id,version,entry) VALUES(?,?,?)', id, e.version, e.entry)),
    ]);
  }
  async join({ id, expectedVersion, state, version, nonce, seat, tokenHash, requestHash, intentHash, response, entry }) {
    const result = await this.db.batch([
      this.stmt('UPDATE rooms SET state=?,version=?,attempt_nonce=? WHERE id=? AND version=? AND NOT EXISTS(SELECT 1 FROM entry_receipts WHERE request_hash=?)',
        state, version, nonce, id, expectedVersion, requestHash),
      this.stmt('INSERT INTO seats(room_id,seat,token_hash) SELECT id,?,? FROM rooms WHERE id=? AND version=? AND attempt_nonce=?', seat, tokenHash, id, version, nonce),
      this.stmt('INSERT INTO journal(room_id,version,entry) SELECT id,?,? FROM rooms WHERE id=? AND version=? AND attempt_nonce=?', version, entry, id, version, nonce),
      this.stmt('INSERT INTO entry_receipts(request_hash,intent_hash,response,room_id) SELECT ?,?,?,id FROM rooms WHERE id=? AND version=? AND attempt_nonce=?', requestHash, intentHash, response, id, version, nonce),
    ]);
    return result[0].meta.changes === 1;
  }
  async command({ id, seat, commandId, intentHash, expectedVersion, state, version, nonce, response, entry }) {
    const result = await this.db.batch([
      this.stmt('UPDATE rooms SET state=?,version=?,attempt_nonce=? WHERE id=? AND version=? AND NOT EXISTS(SELECT 1 FROM commands WHERE room_id=? AND command_id=?)',
        state, version, nonce, id, expectedVersion, id, commandId),
      this.stmt('INSERT INTO journal(room_id,version,entry) SELECT id,?,? FROM rooms WHERE id=? AND version=? AND attempt_nonce=?', version, entry, id, version, nonce),
      this.stmt('INSERT INTO commands(room_id,seat,command_id,intent_hash,response,version) SELECT id,?,?,?,?,? FROM rooms WHERE id=? AND version=? AND attempt_nonce=?',
        seat, commandId, intentHash, response, version, id, version, nonce),
    ]);
    return result[0].meta.changes === 1;
  }
  async system({ id, expectedVersion, state, version, nonce, entry }) {
    const result = await this.db.batch([
      this.stmt('UPDATE rooms SET state=?,version=?,attempt_nonce=? WHERE id=? AND version=?',
        state, version, nonce, id, expectedVersion),
      this.stmt('INSERT INTO journal(room_id,version,entry) SELECT id,?,? FROM rooms WHERE id=? AND version=? AND attempt_nonce=?',
        version, entry, id, version, nonce),
    ]);
    return result[0].meta.changes === 1;
  }
}
