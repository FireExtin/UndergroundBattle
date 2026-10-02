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
  entryReceipt(requestHash) { return this.stmt('SELECT intent_hash,response FROM entry_receipts WHERE request_hash=?', requestHash).first(); }
  async create({ id, invite, state, nonce, tokenHash, requestHash, intentHash, response }) {
    await this.db.batch([
      this.stmt('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,0,?)', id, invite, state, state, nonce),
      this.stmt('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,0,?)', id, tokenHash),
      this.stmt('INSERT INTO entry_receipts(request_hash,intent_hash,response,room_id) VALUES(?,?,?,?)', requestHash, intentHash, response, id),
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
