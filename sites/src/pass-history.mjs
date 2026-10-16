// Public presentation metadata only. Never deserialize or rewrite the opaque game state.
const reasons = new Set(['pass', 'response', 'cancelResponse', 'timeout']);

export function passRecord(view, seat, version, ordinal, reason) {
  const player = view.players?.find(player => player.seat === seat);
  if (!player || !Number.isSafeInteger(view.turn) || typeof view.phase !== 'string' || typeof view.step !== 'string') {
    throw Error('Missing public context for an accepted pass');
  }
  const match = /^region:(\d+):(?:investigation|combat|influence|win)(?::(?:before|after))?$/.exec(view.step);
  const index = match ? Number(match[1]) : null;
  const region = index === null ? null : view.regions?.find(region => region.index === index);
  // A vacant public slot can have an empty name/id. Keep its index; never infer a card.
  if (index !== null && !region) throw Error('Missing public region for an accepted pass');
  return {
    id: `${version}:${ordinal}`, version, playerId: player.id, playerName: player.name,
    turn: view.turn, phase: view.phase, step: view.step,
    region: region ? { index, instanceId: region.id, name: region.name } : null,
    ...(view.responseWindow ? { responseWindowId: view.responseWindow.id } : {}), reason,
  };
}

export function capturePasses(kernel, state, next) {
  const events = next.journal || [];
  const isPass = action => action?.kind === 'passResponse' || action?.kind === 'cancelAndPass'
    || action?.kind === 'game' && action.action?.kind === 'pass';
  if (!events.some(event => event.Tick || isPass(event.Command?.command?.action))) return [];
  let before = JSON.parse(kernel.view(state, next.seat));
  const records = [];
  for (const event of events) {
    if (event.Tick) {
      const now = Math.max(event.Tick.server_now_ms, before.serverNowMs || 0);
      for (const member of before.responseWindow?.members || []) {
        if (member.status !== 'undecided' || !(member.deadlineMs <= now)) continue;
        const player = before.players.find(player => player.id === member.playerId);
        if (!player) throw Error('Unknown public response member');
        records.push(passRecord(before, player.seat, next.version, records.length, 'timeout'));
      }
      if (!records.length) throw Error('Accepted timeout has no public expired member');
      // A Tick followed by a command can cross a phase or region. Obtain the
      // command's actual post-Tick public context from the same frozen reducer.
      // This is a read-only deterministic call; its state is never persisted.
      if (events.some(event => event.Command && isPass(event.Command.command.action))) {
        const tick = JSON.parse(kernel.pollRoom(state, next.seat, String(event.Tick.server_now_ms)));
        if (!tick.changed || tick.version !== next.version) throw Error('Timeout projection disagrees with the accepted transition');
        before = tick.view;
      }
    } else if (isPass(event.Command?.command?.action)) {
      const { seat, command } = event.Command;
      const kind = command.action.kind;
      const reason = kind === 'passResponse' ? 'response' : kind === 'cancelAndPass' ? 'cancelResponse' : 'pass';
      records.push(passRecord(before, seat, next.version, records.length, reason));
    }
  }
  return records;
}

// Read only this explicit public whitelist, even if a journal row has extra fields.
export function publicPassRecord(record, version) {
  if (!record || record.version !== version || !Number.isSafeInteger(version)
    || !/^\d+:\d+$/.test(record.id) || !/^p[0-3]$/.test(record.playerId)
    || typeof record.playerName !== 'string' || !Number.isSafeInteger(record.turn)
    || typeof record.phase !== 'string' || typeof record.step !== 'string' || !reasons.has(record.reason)) return null;
  const region = record.region;
  if (region !== null && (!Number.isSafeInteger(region?.index) || region.index < 0
    || typeof region.instanceId !== 'string' || typeof region.name !== 'string')) return null;
  return {
    id: record.id, version, playerId: record.playerId, playerName: record.playerName,
    turn: record.turn, phase: record.phase, step: record.step,
    region: region === null ? null : { index: region.index, instanceId: region.instanceId, name: region.name },
    ...(typeof record.responseWindowId === 'string' ? { responseWindowId: record.responseWindowId } : {}),
    reason: record.reason,
  };
}
