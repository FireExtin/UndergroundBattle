import type { ActionRejectedEnvelope } from "../protocol";

// Purpose: Presents the most recent structured legality failure without collapsing it into plain text.
type LegalityFailurePanelProps = {
  rejection?: ActionRejectedEnvelope;
};

export function LegalityFailurePanel({ rejection }: LegalityFailurePanelProps) {
  const legality = rejection?.payload.legality;
  const contextEntries = legality?.context ? Object.entries(legality.context) : [];

  return (
    <section className="panel">
      <h2>Legality Failure</h2>
      {legality ? (
        <>
          <p>
            <strong>{legality.reasonCode}</strong>
          </p>
          <p>{legality.messageKey}</p>
          <p>{legality.hook}</p>
          <ul className="simple-list">
            {contextEntries.map(([key, value]) => (
              <li key={key}>
                {key}: {value}
              </li>
            ))}
          </ul>
        </>
      ) : (
        <p className="muted">No legality failures.</p>
      )}
    </section>
  );
}
