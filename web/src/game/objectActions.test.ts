import { describe, expect, it } from 'vitest';
import { groupObjectActions } from './objectActions';
import type { LegalAction } from './types';

const action: LegalAction = { id: 'a', kind: 'play', cardId: 'source', targetId: 'target-a', abilityId: 'ability', label: '行动 → 目标甲' };
const variant = { ...action, id: 'b', targetId: 'target-b', label: '行动 → 目标乙' };

describe('destination grouping counterexamples', () => {
  it('collapses only destination variants and retains both complete original actions', () => {
    const groups = groupObjectActions([action, variant]);
    expect(groups).toHaveLength(1);
    expect(groups[0].actions[0]).toBe(action);
    expect(groups[0].actions[1]).toBe(variant);
  });

  it('keeps a display-only action distinction visible even with identical command fields', () => {
    expect(groupObjectActions([action, { ...action, id: 'b', label: '另一行动 → 目标甲' }])).toHaveLength(2);
  });

  it('keeps a description-only distinction visible even with identical labels and command fields', () => {
    expect(groupObjectActions([{ ...action, description: '支付甲' }, { ...action, id: 'b', description: '支付乙' }])).toHaveLength(2);
  });

  it.each([
    { abilityId: 'other-ability' },
    { option: 'other-mode' },
    { costSelected: ['other-sacrifice'] },
    { selected: ['private-option'] },
    { allocations: { 'damage-target': 2 } },
    { futureCommandField: 'preserve-unknown-payload' },
  ])('does not merge different command payloads hidden behind the same display text: %j', payload => {
    expect(groupObjectActions([action, { ...variant, ...payload }])).toHaveLength(2);
  });

  it('does not merge target and destination-region shapes', () => {
    expect(groupObjectActions([action, { ...action, id: 'b', targetId: undefined, region: 0 }])).toHaveLength(2);
  });
});
