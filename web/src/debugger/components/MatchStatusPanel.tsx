import type { StatePatchedEnvelope } from "../protocol";

// Purpose: Shows the current revision, phase, step, and priority for the selected viewer's latest patch.
type MatchStatusPanelProps = {
  patch?: StatePatchedEnvelope;
};

export function MatchStatusPanel({ patch }: MatchStatusPanelProps) {
  const turn = patch?.payload.playerView?.turn ?? patch?.payload.spectatorView?.turn;
  const revision = patch?.payload.revision.number ?? "-";

  return (
    <section className="panel">
      <h2>Match Status</h2>
      <dl className="status-grid">
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
          <dt>Priority Holder</dt>
          <dd>{turn?.priority.currentPlayerId ?? "-"}</dd>
        </div>
        <div>
          <dt>Priority Window</dt>
          <dd>{turn?.priority.windowKind ?? "-"}</dd>
        </div>
      </dl>
    </section>
  );
}
