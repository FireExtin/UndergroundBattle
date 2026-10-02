import { sqliteTable, text, integer, primaryKey, uniqueIndex } from 'drizzle-orm/sqlite-core';

export const rooms = sqliteTable('rooms', {
  id: text('id').primaryKey(),
  invite: text('invite').notNull().unique(),
  initialState: text('initial_state').notNull(),
  state: text('state').notNull(),
  version: integer('version').notNull(),
  attemptNonce: text('attempt_nonce').notNull(),
});
export const seats = sqliteTable('seats', {
  roomId: text('room_id').notNull().references(() => rooms.id),
  seat: integer('seat').notNull(),
  tokenHash: text('token_hash').notNull(),
}, table => [primaryKey({ columns: [table.roomId, table.seat] }), uniqueIndex('seats_token').on(table.roomId, table.tokenHash)]);
export const commands = sqliteTable('commands', {
  roomId: text('room_id').notNull().references(() => rooms.id),
  seat: integer('seat').notNull(),
  commandId: text('command_id').notNull(),
  intentHash: text('intent_hash').notNull(),
  response: text('response').notNull(),
  version: integer('version').notNull(),
}, table => [primaryKey({ columns: [table.roomId, table.commandId] })]);
export const journal = sqliteTable('journal', {
  roomId: text('room_id').notNull().references(() => rooms.id),
  version: integer('version').notNull(),
  entry: text('entry').notNull(),
}, table => [primaryKey({ columns: [table.roomId, table.version] })]);
// A random client request key recovers a lost create/join acknowledgement.
// Only its hash is stored; receipts containing a seat credential are never logged.
export const entryReceipts = sqliteTable('entry_receipts', {
  requestHash: text('request_hash').primaryKey(),
  intentHash: text('intent_hash').notNull(),
  response: text('response').notNull(),
  roomId: text('room_id').notNull().references(() => rooms.id),
});
