//! Hegemony P15: sealed cards are public, blank and outside play.
//! The first producer binds one character and selects one hand card after drawing.
use crate::{catalog, engine::RuleResult, model::*, rules::Event};
use std::collections::BTreeSet;

impl Game {
    pub(crate) fn seal_host_valid(&self, host_id: &str) -> bool {
        self.board(host_id).is_some_and(|(_, c)| {
            !c.face_down
                && catalog::catalog()
                    .cards
                    .iter()
                    .any(|d| d.id == c.definition && d.kind == "character")
        })
    }

    pub(crate) fn choose_hand_seal(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        let seat = frame.actor;
        let host_id = frame.targets.first().ok_or("缺少封印载体目标")?.id.clone();
        if self.players[seat].eliminated || !self.seal_host_valid(&host_id) {
            return Ok(());
        }
        let options = self.players[seat]
            .hand
            .iter()
            .map(|c| self.option(c, seat, None, None))
            .collect::<Vec<_>>();
        if !options.is_empty() {
            self.choice(
                seat,
                "handSeal",
                "选择一张手牌封印在目标角色上".into(),
                options,
                1,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(frame),
                    choice: FrameChoice::HandSeal { seat, host_id },
                },
            );
        }
        Ok(())
    }

    pub(crate) fn seal_hand_card(
        &mut self,
        holder: usize,
        id: &str,
        host_id: &str,
    ) -> RuleResult<()> {
        if self.players[holder].eliminated || !self.seal_host_valid(host_id) {
            return Err("原封印载体已离场或翻暗".into());
        }
        let index = self.players[holder]
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("原手牌实例已失效")?;
        let c = self.players[holder].hand.remove(index);
        self.seal_card_on_valid_host(c, holder, host_id);
        Ok(())
    }

    // The selection and source-zone checks stay with each closed producer.
    // This primitive only performs a successful, already validated sealing.
    pub(crate) fn seal_card_on_valid_host(&mut self, c: Card, actor: usize, host_id: &str) {
        debug_assert!(actor < self.players.len() && !self.players[actor].eliminated);
        debug_assert!(self.seal_host_valid(host_id));
        // Capture the source and its holder before the zone transition. The
        // sealed card is blank; an already produced explicit trigger survives it.
        let mut source = self.source_snapshot(&c, None);
        source.card.controller = actor;
        let name = catalog::card(&c.definition).name.clone();
        let card = self.reset_zone_card(c);
        self.sealed_cards.push(SealedCard {
            card,
            host_id: host_id.into(),
        });
        self.note(format!("{name} 被封印在目标角色上"));
        self.emit_event(actor, source, Event::Sealed);
    }

    pub(crate) fn return_sealed_cards(&mut self, host_id: &str) {
        let (returning, staying): (Vec<_>, Vec<_>) = std::mem::take(&mut self.sealed_cards)
            .into_iter()
            .partition(|s| s.host_id == host_id);
        self.sealed_cards = staying;
        for s in returning {
            let owner = s.card.owner;
            if !self.players[owner].eliminated {
                let name = catalog::card(&s.card.definition).name.clone();
                let card = self.reset_zone_card(s.card);
                self.players[owner].hand.push(card);
                self.note(format!("{name}：封印载体离场或翻暗，回到拥有者手牌"));
            }
        }
    }

    pub(crate) fn sealed_card_view(&self, card: &Card, viewer: usize) -> CardView {
        let mut view = self.card_view(card, viewer, None, Some("sealed"));
        view.icons = None;
        view.defense = None;
        view.magic = None;
        view.damage = None;
        view.wounds = None;
        view.shield = None;
        view.cost = None;
        view.effective_cost = None;
        view.text = Some("封印牌：公开、场外、空白；载体离场或翻暗时回到拥有者手牌".into());
        view
    }

    pub(crate) fn validate_sealed_cards(&self) -> RuleResult<()> {
        if let Some(p) = &self.pending {
            if let ChoiceResolution::Frame {
                frame,
                choice: FrameChoice::HandSeal { seat, host_id },
            } = &p.resolution
            {
                let actual_hand = self
                    .players
                    .get(*seat)
                    .filter(|player| !player.eliminated)
                    .map(|player| {
                        player
                            .hand
                            .iter()
                            .map(|c| c.id.as_str())
                            .collect::<BTreeSet<_>>()
                    });
                let options = p
                    .choice
                    .options
                    .iter()
                    .map(|o| o.id.as_str())
                    .collect::<BTreeSet<_>>();
                if p.seat != *seat
                    || p.choice.player_id != format!("p{seat}")
                    || frame.actor != *seat
                    || frame.source.card.controller != *seat
                    || frame.source.card.definition != "XQ40"
                    || frame.ability_key != "draw-hand-seal"
                    || !matches!(frame.guard, GuardState::Accepted)
                    || frame.cursor != 2
                    || frame.steps.len() != 2
                    || frame.steps.iter().any(|step| step.context != *seat)
                    || frame.steps.first().is_none_or(|step| {
                        !matches!(
                            step.op,
                            crate::rules::Op::Draw {
                                player: crate::rules::PlayerRef::Actor,
                                count: 1,
                                end: crate::rules::DeckEnd::Top
                            }
                        )
                    })
                    || frame
                        .steps
                        .get(frame.cursor.saturating_sub(1))
                        .is_none_or(|step| {
                            !matches!(step.op, crate::rules::Op::SealOneActorHandCardOnTarget)
                        })
                    || frame.targets.len() != 1
                    || frame.targets[0].id != *host_id
                    || !self.seal_host_valid(host_id)
                    || p.choice.kind != "handSeal"
                    || p.choice.min != Some(1)
                    || p.choice.max != Some(1)
                    || p.choice.allow_decline == Some(true)
                    || options.is_empty()
                    || options.len() != p.choice.options.len()
                    || actual_hand.as_ref() != Some(&options)
                {
                    return Err("封印手牌的待选快照与真实手牌、选择者或载体不符".into());
                }
            }
        }
        // This validates the added zone only, without changing historical state
        // schemas or creating a second identity/target registry.
        let mut ids = BTreeSet::new();
        for c in self
            .players
            .iter()
            .flat_map(|p| {
                p.hand
                    .iter()
                    .chain(&p.deck)
                    .chain(&p.assets)
                    .chain(&p.graveyard)
                    .chain(&p.score_cards)
            })
            .chain(
                self.regions
                    .iter()
                    .flat_map(|r| std::iter::once(&r.card).chain(&r.cards)),
            )
            .chain(&self.world)
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
        {
            ids.insert(c.id.as_str());
        }
        for s in &self.sealed_cards {
            if s.card.owner >= self.players.len()
                || s.card.controller != s.card.owner
                || self.players[s.card.owner].eliminated
                || s.card.face_down
                || s.card.exhausted
                || s.card.damage != 0
                || s.card.wounds != 0
                || s.card.shield != 0
                || !catalog::catalog()
                    .cards
                    .iter()
                    .any(|c| c.id == s.card.definition)
                || s.card.id.is_empty()
                || !ids.insert(s.card.id.as_str())
                || !self.seal_host_valid(&s.host_id)
            {
                return Err("封印区的真实载体、牌或拥有者状态无效".into());
            }
        }
        Ok(())
    }
}
