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
  ownerId: string;
  zone: CardView["zone"];
  kind?: string;
  regionCardId?: string;
  name?: string;
};

export function BattleTable({ battle, localPlayerId, onLocalPlayerChanged, onCardPicked }: BattleTableProps) {
  const localCharacters = battle.local.table.filter((card) => card.kind === "character");
  const localAssets = battle.local.assets;
  const opponentCharacters = battle.opponent.table.filter((card) => card.kind === "character");
  const opponentAssets = battle.opponent.assets;
  const playerOrder = derivePlayerOrder(battle);

  return (
    <section className="battle-table" aria-label="战场桌面">
      <header className="battle-table__header">
        <div>
          <p className="eyebrow">隐秘世界</p>
          <h1>可玩牌桌原型</h1>
          <p className="muted">
            回合 {battle.turn.turnNumber} | 当前玩家 {battle.turn.activePlayerId} | 优先权 {battle.turn.priority.currentPlayerId}
          </p>
          <p className="muted">
            资源：{formatPlayerValueSummary(battle.turn.resources, playerOrder, formatResource)}
          </p>
          <p className="muted">
            分数：{formatPlayerValueSummary(battle.score.byPlayer, playerOrder)} | 胜利阈值 {battle.score.victoryThreshold}
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
          <ZoneStat label="对方资产区" value={battle.opponent.assetCount} />
          <ZoneStat label="对方墓地" value={battle.opponent.discardCount} />
          <ZoneStat label="对方计分区" value={battle.opponent.scoreCount} />
          <ZoneStat
            label="对方秘社标记"
            value={battle.opponent.markers.secret_society ?? 0}
          />
        </div>
        <CardStrip cards={battle.opponent.handPreview} fallback="(无可见手牌)" />
        <div className="battle-zone__stacks">
          <div>
            <p className="muted">对方角色区</p>
            <CardStrip
              cards={opponentCharacters}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
            />
          </div>
          <div>
            <p className="muted">对方资产区</p>
            <CardStrip
              cards={opponentAssets}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
            />
          </div>
        </div>
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
                      const region = slot.regionCard;
                      if (!region?.cardId) {
                        return;
                      }
                      onCardPicked(toCardPick(region));
                    }}
                  >
                    <strong>{slot.regionCard.name ?? slot.regionCard.cardId}</strong>
                    <p className="muted">
                      势力值 {slot.regionCard.stats.influence} · 分值 {slot.regionCard.regionScore ?? 0}
                    </p>
                    <p className="muted">
                      当前控制：{formatRegionController(slot.regionCard.controllerId)}
                    </p>
                    <p className="muted">
                      地区势力：{formatPlayerValueSummary(slot.regionCard.influenceByPlayer, playerOrder)}
                    </p>
                  </button>
                ) : (
                  <strong>空地区槽</strong>
                )}
              </header>
              <p className="muted">地区卡 ID：{slot.regionCard?.cardId ?? "-"}</p>
              {slot.regionCard?.description ? (
                <details className="battle-region-meta">
                  <summary>查看地区说明</summary>
                  <p>{slot.regionCard.description}</p>
                  {slot.regionCard.faq ? <p className="muted">FAQ：{slot.regionCard.faq}</p> : null}
                </details>
              ) : null}
              <div className="contest-slot__rows">
                <div>
                  <p className="muted">对方驻场</p>
                  <CardStrip
                    cards={slot.opponentCards}
                    fallback="(空)"
                    compact
                    onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
                  />
                </div>
                <div>
                  <p className="muted">本方驻场</p>
                  <CardStrip
                    cards={slot.localCards}
                    fallback="(空)"
                    compact
                    onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
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
              onCardPicked(toCardPick(card));
            }}
          />
        </div>
      </section>

      <section className="battle-zone battle-zone--local">
        <h2>本方玩家区域</h2>
        <div className="battle-zone__summary">
          <ZoneStat label="玩家手牌" value={battle.local.hand.length} />
          <ZoneStat label="玩家牌库" value={battle.local.deck.length} />
          <ZoneStat label="资产区" value={battle.local.assets.length} />
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
              onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
            />
          </div>
          <div>
            <p className="muted">本方角色区</p>
            <CardStrip
              cards={localCharacters}
              fallback="(空)"
              onVisibleCardPicked={(card) => {
                onCardPicked(toCardPick(card));
              }}
            />
          </div>
          <div>
            <p className="muted">本方资产区</p>
            <CardStrip
              cards={localAssets}
              fallback="(空)"
              onVisibleCardPicked={(card) => {
                onCardPicked(toCardPick(card));
              }}
            />
          </div>
          <div>
            <p className="muted">本方墓地</p>
            <CardStrip
              cards={battle.local.discard}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
            />
          </div>
          <div>
            <p className="muted">本方计分区</p>
            <CardStrip
              cards={battle.local.score}
              fallback="(空)"
              compact
              onVisibleCardPicked={(card) => onCardPicked(toCardPick(card))}
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
          <li
            key={key}
            className={cardClasses(card, isVisible)}
            style={cardStyle(card, isVisible)}
          >
            {canPick ? (
              <button
                type="button"
                className="battle-card__button"
                onClick={() => {
                  onVisibleCardPicked(card);
                }}
                >
                <div className="battle-card__main">
                  <strong>{card.name ?? card.cardId}</strong>
                  <p className="battle-card__sub">{cardLine(card)}</p>
                </div>
                {isVisible && card.kind === "character" && (
                  <div className="battle-card__stats">
                    <span className="stat-combat" title="战斗力">{card.stats.combat}</span>
                    <span className="stat-defense" title="防御力">{card.stats.defense}</span>
                    <span className="stat-investigation" title="调查力">{card.stats.investigation}</span>
                  </div>
                )}
              </button>
            ) : (
              <>
                <div className="battle-card__main">
                  <strong>{isVisible ? card.name ?? card.cardId : "卡背"}</strong>
                  <p className="battle-card__sub">{isVisible ? cardLine(card) : "隐藏"}</p>
                </div>
                {isVisible && card.kind === "character" && (
                  <div className="battle-card__stats">
                    <span className="stat-combat" title="战斗力">{card.stats.combat}</span>
                    <span className="stat-defense" title="防御力">{card.stats.defense}</span>
                    <span className="stat-investigation" title="调查力">{card.stats.investigation}</span>
                  </div>
                )}
              </>
            )}
          </li>
        );
      })}
    </ul>
  );
}

function cardClasses(card: CardView, isVisible: boolean) {
  const classes = ["battle-card"];
  if (!isVisible) {
    classes.push("battle-card--hidden");
  }
  if (card.exhausted) {
    classes.push("battle-card--exhausted");
  }
  if (card.faceDown) {
    classes.push("battle-card--face-down");
  }
  if (card.destroyed) {
    classes.push("battle-card--destroyed");
  }
  return classes.join(" ");
}

const colorMap: Record<string, string> = {
  "方碑序列": "#4a90e2",
  "帷幕守望": "#50e3c2",
  "王座会": "#d0021b",
  "国家机构": "#f5a623",
  "中立": "#9b9b9b"
};

function cardStyle(card: CardView, isVisible: boolean) {
  if (!isVisible || !card.color) {
    return {};
  }
  const baseColor = colorMap[card.color] || "#ffffff";
  return {
    "--card-accent": baseColor,
    borderLeft: `4px solid ${baseColor}`
  } as React.CSSProperties;
}

function toCardPick(card: CardView): BattleCardPick {
  return {
    cardId: String(card.cardId ?? ""),
    ownerId: card.ownerId,
    zone: card.zone,
    kind: card.kind,
    regionCardId: card.regionCardId,
    name: card.name
  };
}

function cardLine(card: CardView) {
  const parts: string[] = [];
  if (typeof card.cost === "number") {
    parts.push(`费用 ${card.cost}`);
  }
  if (card.color) {
    parts.push(card.color);
  }
  if (card.kind === "region") {
    parts.push(`分值 ${card.regionScore ?? 0}`);
  }
  if (parts.length === 0) {
    return card.cardId ?? "-";
  }
  return parts.join(" · ");
}

function formatRegionController(controllerId: string | undefined) {
  if (!controllerId) {
    return "无人控制";
  }
  return controllerId;
}

function derivePlayerOrder(battle: BattleState): string[] {
  const ordered: string[] = [battle.localPlayerId, battle.opponentPlayerId];

  for (const playerId of Object.keys(battle.score.byPlayer ?? {})) {
    if (!ordered.includes(playerId)) {
      ordered.push(playerId);
    }
  }

  for (const playerId of Object.keys(battle.turn.resources ?? {})) {
    if (!ordered.includes(playerId)) {
      ordered.push(playerId);
    }
  }

  return ordered;
}

export function formatPlayerValueSummary<T>(
  values: Record<string, T> | undefined,
  preferredOrder: string[],
  formatter?: (value: T | undefined) => string
) {
  const format = formatter ?? ((value: T | undefined) => String(value ?? 0));
  const seen = new Set<string>();
  const orderedEntries: Array<[string, T | undefined]> = [];

  for (const playerId of preferredOrder) {
    if (seen.has(playerId)) {
      continue;
    }
    seen.add(playerId);
    orderedEntries.push([playerId, values?.[playerId]]);
  }

  for (const [playerId, value] of Object.entries(values ?? {})) {
    if (seen.has(playerId)) {
      continue;
    }
    seen.add(playerId);
    orderedEntries.push([playerId, value]);
  }

  return orderedEntries.map(([playerId, value]) => `${playerId} ${format(value)}`).join(" · ");
}

function formatResource(resource: { current: number; max: number } | undefined) {
  if (!resource) {
    return "-/-";
  }
  return `${resource.current}/${resource.max}`;
}
