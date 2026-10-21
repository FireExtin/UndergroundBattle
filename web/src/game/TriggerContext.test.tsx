import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { testChoice } from './testFixtures';

const action = { id: 'choose-trigger', kind: 'choose', label: '选择' };

describe('printed trigger context from the room catalog', () => {
  it.each([true, false])('shows a win trigger source and printed effect while preserving acceptance/decline (%s)', accept => {
    const submit = vi.fn();
    const choice = { ...testChoice, kind: 'trigger', title: '切尔诺贝利：赢取触发', min: 0, max: 1,
      allowDecline: true, options: [{ id: 'accept', label: '发动触发能力' }],
      description: '请选择是否发动。' };
    const source = { id: 'DQJC113', name: '切尔诺贝利', kind: 'region', cost: 0,
      text: '赢取触发：切尔诺贝利对场上的所有角色造成1点伤害。' };
    render(<ChoicePanel choice={choice} action={action} definitions={new Map([[source.id, source]])} busy={false} onSubmit={submit} />);
    expect(screen.getByRole('heading', { name: '切尔诺贝利：赢取触发' })).toBeInTheDocument();
    expect(screen.getByLabelText('触发来源与卡牌文字')).toHaveTextContent('来源：切尔诺贝利');
    expect(screen.getByLabelText('触发来源与卡牌文字')).toHaveTextContent(source.text);
    if (accept) fireEvent.click(screen.getByRole('button', { name: '发动触发能力' }));
    fireEvent.click(screen.getByRole('button', { name: accept ? '确认选择' : '跳过此选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: accept ? ['accept'] : [] });
  });

  it('does not guess a trigger source from a partial or ambiguous printed name', () => {
    const source = { id: 'world-a', name: '切尔诺贝利', kind: 'region', cost: 0, text: '不可误显示的甲版本效果' };
    const definitions = new Map([[source.id, source], ['world-b', { ...source, id: 'world-b', text: '不可误显示的乙版本效果' }]]);
    const choice = { ...testChoice, kind: 'trigger', title: '切尔诺贝利：赢取触发' };
    const { rerender } = render(<ChoicePanel choice={choice} action={action} definitions={definitions} busy={false} onSubmit={vi.fn()} />);
    expect(screen.queryByLabelText('触发来源与卡牌文字')).not.toBeInTheDocument();
    rerender(<ChoicePanel choice={{ ...choice, title: '切尔诺贝利邻近地区：触发' }} action={action} definitions={new Map([[source.id, source]])} busy={false} onSubmit={vi.fn()} />);
    expect(screen.queryByText(source.text)).not.toBeInTheDocument();
  });

});
