// D1 exceptions can contain SQL and bound private state. Log only fixed labels.
const errors = [
  ['network_lost', 'Network connection lost.', true],
  ['replica_disconnected', 'Replica disconnected from primary.', true],
  ['code_reset', 'D1 DB reset because its code was updated.', true],
  ['startup_reset', 'Internal error while starting up D1 DB storage caused object to be reset.', true],
  ['storage_reset', 'Internal error in D1 DB storage caused object to be reset.', true],
  ['remote_node', 'Cannot resolve D1 DB due to transient issue on remote node.', true],
  ['timeout_reset', 'D1 DB storage operation exceeded timeout which caused object to be reset.', true],
  ['queue_timeout', 'D1 DB is overloaded. Requests queued for too long.', true],
  ['queue_full', 'D1 DB is overloaded. Too many requests queued.', true],
  ['memory_reset', "D1 DB's isolate exceeded its memory limit and was reset.", false],
  ['cpu_reset', 'D1 DB exceeded its CPU time limit and was reset.', false],
  ['row_read_quota', "Your account has exceeded D1's free tier daily row read limit.", false],
  ['row_write_quota', "Your account has exceeded D1's free tier daily row write limit.", false],
  ['database_size', 'Exceeded maximum DB size.', false],
  ['account_storage', "Your account has exceeded D1's maximum account storage limit", false],
  ['unique_constraint', 'UNIQUE constraint failed', false],
  ['foreign_key', 'FOREIGN KEY constraint failed', false],
  ['value_size', 'string or blob too big', false],
];
export function storageFailure(error) {
  const messages = []; let current = error;
  for (let i = 0; current && i < 4; i++, current = current.cause) {
    if (typeof current === 'string') messages.push(current);
    else if (typeof current.message === 'string') messages.push(current.message);
  }
  const match = errors.find(([, phrase]) => messages.some(message => message.includes(phrase)));
  return { code: match?.[0] ?? 'unclassified', retryable: match?.[2] ?? false };
}
export async function retryOriginalCommand(operation, { wait = ms => new Promise(resolve => setTimeout(resolve, ms)), log = console.warn } = {}) {
  for (let attempt = 0; ; attempt++) {
    try { return await operation(); }
    catch (error) {
      if (typeof error?.status === 'number') throw error;
      const failure = storageFailure(error);
      if (!failure.retryable || attempt >= 2) throw error;
      log('hegemony storage retry', { ...failure, attempt: attempt + 1 });
      await wait(150 * 2 ** attempt + Math.floor(Math.random() * 100));
    }
  }
}
