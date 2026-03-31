import type { StatePatchedEnvelope } from "../protocol";

// Purpose: Shows the current revision, phase, step, and priority for the selected viewer's latest patch.
type MatchStatusPanelProps = {
  patch?: StatePatchedEnvelope;
};

export function MatchStatusPanel({ patch }: MatchStatusPanelProps) {
  const match = patch?.payload.playerView?.match ?? patch?.payload.spectatorView?.match;
  const turn = patch?.payload.playerView?.turn ?? patch?.payload.spectatorView?.turn;
  const score = patch?.payload.playerView?.score ?? patch?.payload.spectatorView?.score;
  const revision = patch?.payload.revision.number ?? "-";
  const scoreEntries = Object.entries(score?.byPlayer ?? {}).sort(([left], [right]) =>
    left.localeCompare(right)
  );

  return (
    <section className="panel">
      <h2>Match Status</h2>
      <dl className="status-grid">
        <div>
          <dt>Match State</dt>
          <dd>{match?.status ?? "-"}</dd>
        </div>
        <div>
          <dt>Current Revision</dt>
          <dd>{revision}</dd>
        </div>
        <div>
          <dt>Current Phase</dt>
          <dd>{turn?.phase.name ?? "-"}</dd>
        </div>
        <div>
          <dt>Current Step</dt>
          <dd>{turn?.phase.step ?? "-"}</dd>
        </div>
        <div>
          <dt>Active Player</dt>
          <dd>{turn?.activePlayerId ?? "-"}</dd>
        </div>
        <div>
          <dt>Priority Holder</dt>
          <dd>{turn?.priority.currentPlayerId ?? "-"}</dd>
        </div>
        <div>
          <dt>Priority Window</dt>
          <dd>{turn?.priority.windowKind ?? "-"}</dd>
        </div>
        <div>
          <dt>Score</dt>
          <dd>{scoreEntries.length === 0 ? "-" : scoreEntries.map(([playerId, value]) => `${playerId}: ${value}`).join(" | ")}</dd>
        </div>
        <div>
          <dt>Winner</dt>
          <dd>{score?.winnerPlayerId ?? "-"}</dd>
        </div>
        <div>
          <dt>End Reason</dt>
          <dd>{match?.endReason ?? "-"}</dd>
        </div>
      </dl>
    </section>
  );
}
