import type { CardView } from "../protocol";

// Purpose: Shows the currently selected player or spectator projection while preserving hidden-information differences.
type PlayerViewPanelProps = {
  viewerLabel: string;
  cards: CardView[];
};

export function PlayerViewPanel({ viewerLabel, cards }: PlayerViewPanelProps) {
  return (
    <section className="panel">
      <h2>{viewerLabel} View</h2>
      <ul className="card-list" aria-label="Viewer Cards">
        {cards.map((card, index) => {
          const key = card.cardId ?? `${card.ownerId}-${card.zone}-${index}`;
          const isVisible = card.visibility === "visible";
          const shield = card.counters.shield ?? 0;
          const markerSummary = Object.entries(card.markers ?? {})
            .sort(([left], [right]) => left.localeCompare(right))
            .map(([markerType, amount]) => `${markerType} ${amount}`)
            .join(", ");

          return (
            <li key={key} className="card-row">
              <div>
                <strong>{isVisible ? card.name : "hidden"}</strong>
                <p className="muted">{card.zone}</p>
                {isVisible ? (
                  <p className="card-details">
                    keywords: {card.keywords?.join(", ") || "none"} | stats: {card.stats.combat}/
                    {card.stats.defense}/{card.stats.influence}/{card.stats.investigation} | counters: dmg{" "}
                    {card.counters.damage}, inf {card.counters.influence}, shd {shield} | face-down:{" "}
                    {card.faceDown ? "yes" : "no"} | markers: {markerSummary || "none"}
                  </p>
                ) : null}
              </div>
              <div className="card-meta">
                <span>{card.visibility}</span>
                {isVisible ? <span>{card.cardId}</span> : null}
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
