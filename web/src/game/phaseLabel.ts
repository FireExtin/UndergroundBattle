const phaseNames: Record<string, string> = { start: '开始阶段', beginning: '开始阶段', action: '行动阶段', confrontation: '对抗阶段', conflict: '对抗阶段', end: '结束阶段', finished: '对局结束', lobby: '准备入席', investigation: '调查', combat: '战斗', influence: '势力', claim: '赢区窗口', win: '赢区窗口', fast: '快速行动窗口', draw: '抓牌', prepare: '重置与准备', mobility: '机动窗口', mulligan: '再调度', ready: '准备' };
export function phaseLabel(phase: string) {
  if (phaseNames[phase]) return phaseNames[phase];
  const region = /^region:(\d+):([^:]+)(?::(before|after))?$/.exec(phase);
  if (region) return `地区 ${Number(region[1]) + 1} · ${phaseNames[region[2]] || '对抗'}${region[3] ? ` · ${region[3] === 'before' ? '奖励前' : '奖励后'}快速窗口` : ''}`;
  const team = /^team:(\d+)$/.exec(phase);
  if (team) return `团队 ${Number(team[1]) + 1} 行动`;
  return /[\u4e00-\u9fff]/.test(phase) ? phase : '进行中';
}
