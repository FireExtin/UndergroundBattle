// Independent server-side foundation. Not connected to RoomService or enabled
// in production yet. Authority must come from persisted room metadata, never
// from the client's action. The opaque Rust room state is not accepted here.
const MAX_U64 = (1n << 64n) - 1n;
const difficultyNames = new Set(['easy', 'normal', 'hard']);
const actionKeys = new Set(['kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected','deckDraft']);
function fail(message) { throw new Error(`Bot policy: ${message}`); }
function exact(value, keys) {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).some(k => !keys.includes(k))) fail('unexpected configuration fields');
}
function uint64(value) {
  if (typeof value !== 'string' || value.length > 20 || !/^(0|[1-9][0-9]*)$/.test(value) || BigInt(value) > MAX_U64) fail('uint64 must be a canonical decimal string');
  return BigInt(value);
}
function seats(value) {
  if (!Array.isArray(value) || value.some(n => !Number.isInteger(n) || n < 0 || n > 3)
      || new Set(value).size !== value.length || value.some((n, i) => i > 0 && n <= value[i - 1])) fail('seats must be distinct and ordered');
  return value;
}
function freeze(value) {
  if (value && typeof value === 'object') { Object.values(value).forEach(freeze); Object.freeze(value); }
  return value;
}
function clone(value) { return structuredClone(value); }
function roomIdentity(roomId) { if (typeof roomId !== 'string' || !/^[a-f0-9]{24}$/.test(roomId)) fail('room identity is required'); }
function validate(policy) {
  exact(policy, ['schema','roomId','roomKind','difficulty','humanSeats','botSeats','advantages','seed','random','counter']);
  roomIdentity(policy.roomId);
  if (policy.schema !== 1 || policy.roomKind !== 'bot' || !difficultyNames.has(policy.difficulty)) fail('explicit Bot room and known difficulty required');
  seats(policy.humanSeats); seats(policy.botSeats);
  // Initial slice is solo practice: a host and one/three server-owned Bots.
  const all = [...policy.humanSeats, ...policy.botSeats].sort((a,b) => a-b);
  if (policy.humanSeats.length !== 1 || policy.humanSeats[0] !== 0 || ![2,4].includes(all.length)
      || all.some((n,i) => n !== i)) fail('solo Bot-room seat partition is required');
  const advantages = policy.difficulty === 'hard' ? ['peekHumanHands'] : [];
  if (JSON.stringify(policy.advantages) !== JSON.stringify(advantages)) fail('advantages must match the disclosed difficulty');
  uint64(policy.seed); uint64(policy.random); uint64(policy.counter);
  return policy;
}
export function createBotPolicy(input) {
  exact(input, ['roomId','difficulty','botSeats','seed']);
  const { roomId, difficulty, botSeats, seed } = input;
  return freeze(validate({ schema:1, roomId, roomKind:'bot', difficulty, humanSeats:[0], botSeats:clone(botSeats),
    advantages:difficulty === 'hard' ? ['peekHumanHands'] : [], seed, random:seed, counter:'0' }));
}
export function restoreBotPolicy(serialized) {
  if (typeof serialized !== 'string') fail('persisted policy must be JSON text');
  return freeze(validate(JSON.parse(serialized)));
}
export function assertBotAuthority(policy, authority, seat) {
  validate(policy);
  exact(authority, ['roomId','roomKind','difficulty','humanSeats','botSeats']);
  if (authority.roomKind !== 'bot' || authority.roomId !== policy.roomId || authority.difficulty !== policy.difficulty
      || JSON.stringify(authority.humanSeats) !== JSON.stringify(policy.humanSeats)
      || JSON.stringify(authority.botSeats) !== JSON.stringify(policy.botSeats) || !policy.botSeats.includes(seat)) {
    fail('persisted room mode, difficulty and Bot seat must match');
  }
}
export function publicBotDisclosure(policy) {
  validate(policy);
  return freeze({ roomKind:'bot', difficulty:policy.difficulty, advantages:clone(policy.advantages),
    description:policy.difficulty === 'hard' ? '困难Bot可以读取你的当前手牌，用于选择合法行动。' : 'Bot仅依据自己合法可见的玩家视图选择行动。' });
}
function readView(getView, seat, roomId, revision) {
  const view = getView(seat);
  if (!view || view.roomId !== roomId || view.you !== `p${seat}` || !Number.isSafeInteger(view.version) || view.version < 0
      || !['lobby','playing','finished'].includes(view.status)
      || revision !== undefined && view.version !== revision || !Array.isArray(view.hand) || !Array.isArray(view.legalActions)) fail('same-room, same-revision legal player view required');
  return view;
}
export function botDecisionContext(policy, authority, seat, getView) {
  assertBotAuthority(policy, authority, seat);
  if (typeof getView !== 'function') fail('legal-view capability required');
  const own = readView(getView, seat, policy.roomId);
  // Never forward arbitrary server properties, another player's complete view,
  // or a raw-state field. Normal modes never call the human-view capability.
  const keys = ['roomId','you','version','status','mode','players','hand','graveyard','assets','regions','attachments',
    'societyZones','pendingChoice','waitingChoice','responseWindow','pause','legalActions','firstTeam','activeTeam','priorityTeam'];
  const view = Object.fromEntries(keys.filter(k => Object.hasOwn(own,k)).map(k => [k,clone(own[k])]));
  const humanHands = policy.difficulty === 'hard' ? policy.humanSeats.map(s => {
    const other = readView(getView, s, policy.roomId, own.version);
    return { seat:s, cards:clone(other.hand) };
  }) : [];
  return freeze({ view, humanHands });
}
function randomStep(policy) {
  const counter = uint64(policy.counter);
  if (counter === MAX_U64) fail('decision counter exhausted');
  const random = (uint64(policy.random) + 0x9e3779b97f4a7c15n) & MAX_U64;
  let value = random;
  value = ((value ^ (value >> 30n)) * 0xbf58476d1ce4e5b9n) & MAX_U64;
  value = ((value ^ (value >> 27n)) * 0x94d049bb133111ebn) & MAX_U64;
  value ^= value >> 31n;
  return { value, policy:freeze({ ...clone(policy), random:random.toString(), counter:(counter+1n).toString() }) };
}
function commandAction(action) {
  return Object.fromEntries(Object.entries(action).filter(([k,v]) => actionKeys.has(k) && v != null));
}
function score(action, context) {
  const base = {asset:70, deploy:60, reveal:55, play:50, activate:45, ready:90, start:100, choose:80, pass:-100};
  let value = base[action.kind] ?? -50;
  if (context.humanHands.some(h => `p${h.seat}` === action.targetId && h.cards.some(c => (c.cost ?? 0) >= 4))) value += 40;
  return value;
}
function choiceCandidate(base, choice, easy) {
  const ids = choice.options?.map(o => o.id) || [];
  if (choice.kind === 'damage') {
    if (!Number.isSafeInteger(choice.amount) || choice.amount < 0 || !ids.length) return null;
    return { ...base, choiceId:choice.id, allocations:{ [ids[0]]:choice.amount } };
  }
  if (choice.kind === 'order' || choice.kind === 'investigation') return { ...base, choiceId:choice.id, top:ids, bottom:[] };
  const min = choice.min ?? 0, max = choice.max ?? ids.length;
  if (!Number.isSafeInteger(min) || !Number.isSafeInteger(max) || min < 0 || max < min || ids.length < min) return null;
  const n = easy || choice.kind === 'mulligan' ? min : Math.max(min, Math.min(max,1));
  return { ...base, choiceId:choice.id, selected:ids.slice(0,n) };
}
// Returns a candidate, never an executed action or a claim of rule acceptance.
// RoomService must validate/commit through Rust, CAS and original-command
// receipts. Complex allocations may be rejected; no direct state edits exist.
export function chooseBotCandidate(policy, authority, seat, getView) {
  const context = botDecisionContext(policy, authority, seat, getView);
  const view = context.view;
  if (view.pause || view.status === 'finished') return null;
  const decisionId = `bot:${policy.roomId}:${seat}:${policy.counter}`;
  const window = view.responseWindow;
  let action;
  if (window && !view.pendingChoice) {
    const member = window.members?.find(m => m.playerId === `p${seat}`);
    if (!member || member.status === 'passed') return null;
    if (member.status === 'undecided') {
      action = window.canBegin && window.previewSources?.length && policy.difficulty !== 'easy'
        ? { kind:'beginResponse', windowId:window.id, intentId:`${decisionId}:i` }
        : { kind:'passResponse', windowId:window.id };
    }
  }
  const step = randomStep(policy);
  if (!action) {
    const candidates = view.legalActions.filter(a => a.kind !== 'inspect-hidden');
    if (!candidates.length) return null;
    const sorted = policy.difficulty === 'easy' ? candidates : [...candidates].sort((a,b) => score(b,context)-score(a,context));
    const eligible = policy.difficulty === 'easy' ? sorted : sorted.filter(a => score(a,context) === score(sorted[0],context));
    let candidate = commandAction(eligible[Number(step.value % BigInt(eligible.length))]);
    if (candidate.kind === 'choose') {
      if (!view.pendingChoice || view.pendingChoice.playerId !== `p${seat}`) return null;
      candidate = choiceCandidate(candidate, view.pendingChoice, policy.difficulty === 'easy');
      if (!candidate) return null;
    }
    action = window?.myIntentId && !view.pendingChoice
      ? candidate.kind === 'pass' ? { kind:'cancelAndPass', windowId:window.id, intentId:window.myIntentId }
        : { kind:'submitResponse', windowId:window.id, intentId:window.myIntentId, action:candidate }
      : { kind:'game', action:candidate };
  }
  return freeze({ commandId:decisionId, expectedVersion:view.version, action, nextPolicy:step.policy,
    mustValidateInRust:true, peekedHumanHands:context.humanHands.length > 0 });
}
export function assertHumanBotCommandId(commandId) {
  if (typeof commandId !== 'string' || !commandId || commandId.length > 64 || commandId.startsWith('bot:')) fail('Bot decision IDs are server-reserved');
}
