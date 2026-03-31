import type { ActionAcceptedEnvelope, ActionRejectedEnvelope } from "../protocol";

// Purpose: Displays accepted and rejected actions in protocol order for debugger inspection.
type ActionLogPanelProps = {
  entries: Array<ActionAcceptedEnvelope | ActionRejectedEnvelope>;
};

export function ActionLogPanel({ entries }: ActionLogPanelProps) {
  return (
    <section className="panel">
      <h2>Action Log</h2>
      <ol className="simple-list" aria-label="Action Log Entries">
        {entries.map((entry) => (
          <li key={entry.messageId}>
            <strong>{entry.name}</strong>
            <span className="muted">
              {" "}
              {entry.payload.action.id} / {entry.payload.action.kind}
            </span>
          </li>
        ))}
      </ol>
    </section>
  );
}
