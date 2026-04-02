import type { CardView } from "../../debugger/protocol";
import type { BattlePlayerId, BattleState } from "../model";

// Purpose: Renders a table-style board with opponent/local areas and the central contest regions.

type BattleTableProps = {
  battle: BattleState;
  localPlayerId: BattlePlayerId;
  onLocalPlayerChanged: (playerId: BattlePlayerId) => void;
  onCardPicked: (picked: BattleCardPick) => void;
};

export type BattleCardPick = {
  cardId: string;
  intent: "source" | "target";
};

export function BattleTable({ battle, localPlayerId, onLocalPlayerChanged, onCardPicked }: BattleTableProps) {
  return (
    <section className="battle-table" aria-label="战场桌面">
      <header className="battle-table__header">
        <div>
          <p className="eyebrow">隐秘世界</p>
          <h1>可玩牌桌原型</h1>
          <p className="muted">
            Turn {battle.turn.turnNumber} | Active {battle.turn.activePlayerId} | Priority {battle.turn.priority.currentPlayerId}
          </p>
        </div>
        <div className="battle-table__viewer-switch" role="group" aria-label="视角切换">
          <button
            type="button"
            className={localPlayerId === "P1" ? "viewer-button viewer-button--active" : "viewer-button"}
            onClick={() => onLocalPlayerChanged("P1")}
          >
            P1 视角
          </button>
          <button
            type="button"
            className={localPlayerId === "P2" ? "viewer-button viewer-button--active" : "viewer-button"}
            onClick={() => onLocalPlayerChanged("P2")}
          >
            P2 视角
          </button>
        </div>
      </header>

      <section className="battle-zone battle-zone--opponent">
        <h2>对方玩家区域</h2>
        <div className="battle-zone__summary">
          <ZoneStat label="对方玩家手牌" value={battle.opponent.handCount} />
          <ZoneStat label="对方牌库" value={battle.opponent.deckCount} />
          <ZoneStat label="对方墓地" value={battle.opponent.discardCount} />
          <ZoneStat label="对方计分区" value={battle.opponent.scoreCount} />
          <ZoneStat
            label="对方秘社标记"
            value={battle.opponent.markers.secret_society ?? 0}
          />
        </div>
        <CardStrip cards={battle.opponent.handPreview} fallback="(无可见手牌)" />
      </section>

      <section className="battle-zone battle-zone--contest">
        <h2>争夺区</h2>
        <div className="contest-grid">
          {battle.contest.regions.map((slot) => (
            <article key={slot.slotId} className="contest-slot">
              <header>
                <p>地区 {slot.order}</p>
                {slot.regionCard?.cardId ? (
                  <button
                    type="button"
                    className="battle-region-button"
                    onClick={() => {
                      onCardPicked({ cardId: slot.regionCard!.cardId!, intent: "target" });
                    }}
                  >
                    <strong>{slot.regionCard.name ?? slot.regionCard.cardId}</strong>
                  </button>
                ) : (
                  <strong>空地区槽</strong>
                )}
              </header>
              <p className="muted">regionId: {slot.regionCard?.cardId ?? "-"}</p>
              <div className="contest-slot__rows">
                <div>
                  <p className="muted">对方驻场</p>
                  <CardStrip
                    cards={slot.opponentCards}
                    fallback="(空)"
                    compact
                    onVisibleCardPicked={(card) => pickTargetCard(card, onCardPicked)}
                  />
                </div>
                <div>
                  <p className="muted">本方驻场</p>
                  <CardStrip
                    cards={slot.localCards}
                    fallback="(空)"
                    compact
                    onVisibleCardPicked={(card) => pickSourceCard(card, onCardPicked)}
                  />
                </div>
              </div>
            </article>
          ))}
        </div>
        <div className="contest-free-zone">
          <p className="muted">未分配到地区的争夺区卡牌</p>
          <CardStrip
            cards={battle.contest.unassignedTableCards}
            fallback="(空)"
            onVisibleCardPicked={(card) => {
              if ((card.ownerId ?? "") === localPlayerId) {
                pickSourceCard(card, onCardPicked);
                return;
              }
              pickTargetCard(card, onCardPicked);
            }}
          />
        </div>
      </section>

      <section className="battle-zone battle-zone--local">
        <h2>本方玩家区域</h2>
        <div className="battle-zone__summary">
          <ZoneStat label="玩家手牌" value={battle.local.hand.length} />
          <ZoneStat label="玩家牌库" value={battle.local.deck.length} />
          <ZoneStat label="墓地" value={battle.local.discard.length} />
          <ZoneStat label="计分区" value={battle.local.score.length} />
          <ZoneStat label="秘社" value={battle.local.markers.secret_society ?? 0} />
        </div>
        <div className="battle-zone__stacks">
          <div>
            <p className="muted">本方手牌</p>
            <CardStrip
              cards={battle.local.hand}
              fallback="(空)"
              onVisibleCardPicked={(card) => pickSourceCard(card, onCardPicked)}
            />
          </div>
          <div>
            <p className="muted">本方桌面</p>
            <CardStrip
              cards={battle.local.table}
              fallback="(空)"
              onVisibleCardPicked={(card) => {
                if (card.kind === "region") {
                  pickTargetCard(card, onCardPicked);
                  return;
                }
                pickSourceCard(card, onCardPicked);
              }}
            />
          </div>
          <div>
            <p className="muted">本方墓地</p>
            <CardStrip
              cards={battle.local.discard}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => pickSourceCard(card, onCardPicked)}
            />
          </div>
          <div>
            <p className="muted">本方计分区</p>
            <CardStrip
              cards={battle.local.score}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => pickSourceCard(card, onCardPicked)}
            />
          </div>
        </div>
      </section>
    </section>
  );
}

function ZoneStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="zone-stat">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

type CardStripProps = {
  cards: CardView[];
  fallback: string;
  compact?: boolean;
  onVisibleCardPicked?: (card: CardView) => void;
};

function CardStrip({ cards, fallback, compact = false, onVisibleCardPicked }: CardStripProps) {
  if (cards.length === 0) {
    return <p className="muted">{fallback}</p>;
  }

  return (
    <ul className={compact ? "battle-card-strip battle-card-strip--compact" : "battle-card-strip"}>
      {cards.map((card, index) => {
        const key = card.cardId ?? `${card.ownerId}-${card.zone}-${index}`;
        const isVisible = card.visibility === "visible";
        const canPick = isVisible && !!card.cardId && !!onVisibleCardPicked;

        return (
          <li key={key} className={isVisible ? "battle-card" : "battle-card battle-card--hidden"}>
            {canPick ? (
              <button
                type="button"
                className="battle-card__button"
                onClick={() => {
                  onVisibleCardPicked(card);
                }}
              >
                <strong>{card.name ?? card.cardId}</strong>
                <p>{card.cardId ?? "-"}</p>
              </button>
            ) : (
              <>
                <strong>{isVisible ? card.name ?? card.cardId : "Card Back"}</strong>
                <p>{isVisible ? card.cardId ?? "-" : "hidden"}</p>
              </>
            )}
          </li>
        );
      })}
    </ul>
  );
}

function pickSourceCard(card: CardView, onCardPicked: (picked: BattleCardPick) => void) {
  if (!card.cardId) {
    return;
  }
  onCardPicked({ cardId: card.cardId, intent: "source" });
}

function pickTargetCard(card: CardView, onCardPicked: (picked: BattleCardPick) => void) {
  if (!card.cardId) {
    return;
  }
  onCardPicked({ cardId: card.cardId, intent: "target" });
}
