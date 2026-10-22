import type { View } from './types';
import './bot-match.css';
const names = { easy:'简单', normal:'普通', hard:'困难' };
export function BotMatchNotice({ view }: { view: View | null }) {
  const match = view?.botMatch;
  if (!view || !match) return null;
  return <section className="hg-bot-match-notice" aria-label="Bot对局规则">
    <strong>Bot练习 · {names[match.difficulty]}</strong>
    <p>{match.description}</p>
    {!match.enabled && <p role="status">此服务当前关闭Bot调度，房间和进度已保留。</p>}
    {view.botAttention && <p role="status">{view.botAttention}</p>}
  </section>;
}
