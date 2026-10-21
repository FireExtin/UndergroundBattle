//! WM035 adds a live personal subtype and observes the existing pre-removal
//! batch snapshot. The observation keeps its original controller and region UID.
use crate::{black_next20::BlackDeathObservation, catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

const KEY: &str = "batch3-wm035-host-death";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rules, unit_support::*};

    fn fill(g: &mut Game) {
        let deck = (0..8).map(|_| g.make_card("JC125", 0)).collect();
        g.players[0].deck = deck;
    }
    fn drain_rewards(g: &mut Game) -> usize {
        let mut accepted = 0;
        for _ in 0..128 {
            if let Some(p) = &g.pending {
                assert!(
                    matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. }
                    if declaration.source.card.definition == "WM035")
                );
                choose(g, vec!["accept".into()]);
                accepted += 1;
            } else if !g.stack.is_empty() {
                pass(g);
            } else if g.effects.is_empty() {
                restore(g);
                return accepted;
            } else {
                g.drive().unwrap();
            }
        }
        panic!("host rewards must terminate");
    }
    fn host_declaration(g: &Game) -> Declaration {
        g.effects
            .iter()
            .find_map(|e| match e {
                Effect::Declare { declaration }
                    if declaration.source.card.definition == "WM035" =>
                {
                    Some(declaration.clone())
                }
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn batch3_wm035_personal_live_human_subtype_aura_survives_exhaustion_and_tracks_control() {
        let mut g = game();
        let source = field(&mut g, "WM035", 0);
        let own = field(&mut g, "JC125", 0);
        let ally = field(&mut g, "JC125", 1);
        let enemy = field(&mut g, "JC125", 2);
        let vampire = field(&mut g, "JC029", 0);
        let host = |g: &Game, id: &str| {
            g.current_subtypes(g.board(id).unwrap().1)
                .iter()
                .any(|s| s == "宿主")
        };
        assert!(host(&g, &source) && host(&g, &own));
        assert!(!host(&g, &ally) && !host(&g, &enemy) && !host(&g, &vampire));
        g.board_mut(&source).unwrap().exhausted = true;
        assert!(host(&g, &own));
        g.move_on_board_instance(&own, 1);
        assert!(host(&g, &own));
        g.board_mut(&enemy).unwrap().controller = 0;
        assert!(host(&g, &enemy));
        g.board_mut(&source).unwrap().face_down = true;
        assert!(!host(&g, &own));
        g.board_mut(&source).unwrap().face_down = false;
        assert_eq!(
            g.current_subtypes(g.board(&source).unwrap().1)
                .iter()
                .filter(|s| s.as_str() == "宿主")
                .count(),
            1
        );
        let private = g.make_card("JC125", 0);
        assert!(!g.current_subtypes(&private).iter().any(|s| s == "宿主"));
        restore(&mut g);
    }

    #[test]
    fn batch3_wm035_real_death_freezes_original_controller_and_rejects_duplicate_or_changed_observations(
    ) {
        let mut g = game();
        fill(&mut g);
        let source = field(&mut g, "WM035", 0);
        let human = field(&mut g, "JC125", 2);
        g.board_mut(&human).unwrap().controller = 0;
        g.remove_dead(&human, RemovalCause::Destroy);
        let d = host_declaration(&g);
        assert!(
            matches!(&d.ability.ops[0], Op::WM035HostDeathDraw { observation: Some(o) }
            if o.dead.owner == 2 && o.dead.controller == 0 && o.subtypes.iter().any(|s| s == "宿主"))
        );
        let mut duplicate = g.clone();
        duplicate.effects.push_back(Effect::Declare {
            declaration: d.clone(),
        });
        assert!(Game::from_persisted(&serde_json::to_string(&duplicate).unwrap()).is_err());
        let mut changed = g.clone();
        if let Effect::Declare { declaration } = changed.effects.front_mut().unwrap() {
            if let Op::WM035HostDeathDraw {
                observation: Some(o),
            } = &mut declaration.ability.ops[0]
            {
                o.subtypes.retain(|s| s != "宿主");
            }
        }
        assert!(Game::from_persisted(&serde_json::to_string(&changed).unwrap()).is_err());
        g.drive().unwrap();
        restore(&mut g);
        let mut changed = g.clone();
        changed.pending.as_mut().unwrap().choice.min = Some(1);
        assert!(Game::from_persisted(&serde_json::to_string(&changed).unwrap()).is_err());
        choose(&mut g, vec!["accept".into()]);
        g.board_mut(&source).unwrap().controller = 2;
        top(&mut g);
        assert_eq!(g.players[0].hand.len(), 1);
        assert!(g.players[2].hand.is_empty());
        restore(&mut g);
    }

    #[test]
    fn batch3_wm035_simultaneous_source_and_human_deaths_keep_pre_wave_types_in_both_orders() {
        for source_first in [false, true] {
            let mut g = game();
            fill(&mut g);
            let source = field(&mut g, "WM035", 0);
            let human = field(&mut g, "JC125", 0);
            let ids = if source_first {
                vec![source.clone(), human]
            } else {
                vec![human, source.clone()]
            };
            let deaths = ids
                .iter()
                .map(|id| {
                    let (r, c) = g.in_play(id).unwrap();
                    (id.clone(), RemovalCause::Destroy, g.source_snapshot(c, r))
                })
                .collect();
            g.remove_death_batch(deaths);
            g.drive().unwrap();
            assert_eq!(drain_rewards(&mut g), 2);
            assert_eq!(g.players[0].hand.len(), 2);
            assert!(g.board(&source).is_none());
        }
    }

    #[test]
    fn batch3_wm035_real_embrace_removes_human_before_host_aura_and_death_capture() {
        let mut g = game();
        fill(&mut g);
        field(&mut g, "WM035", 0);
        let human = field(&mut g, "JC125", 2);
        fund(&mut g, 0, "JC036", 5);
        let embrace = held(&mut g, "JC036", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(embrace),
                target_id: Some(human.clone()),
                ..Action::new("play")
            },
        );
        top(&mut g);
        let subtypes = g.current_subtypes(g.board(&human).unwrap().1);
        assert!(subtypes.iter().any(|s| s == "吸血鬼"));
        assert!(!subtypes
            .iter()
            .any(|s| matches!(s.as_str(), "人类" | "宿主")));
        g.remove_dead(&human, RemovalCause::Destroy);
        g.drive().unwrap();
        assert!(g.pending.is_none());
        assert!(g.players[0].hand.is_empty());
        restore(&mut g);
    }

    #[test]
    fn batch3_wm035_hidden_and_allied_humans_do_not_create_personal_host_death_rewards() {
        for (seat, hidden) in [(0, true), (1, false), (2, false)] {
            let mut g = game();
            fill(&mut g);
            field(&mut g, "WM035", 0);
            let target = field(&mut g, "JC125", seat);
            g.board_mut(&target).unwrap().face_down = hidden;
            g.remove_dead(&target, RemovalCause::Destroy);
            g.drive().unwrap();
            assert!(g.pending.is_none());
            assert!(g.players[0].hand.is_empty());
            restore(&mut g);
        }
    }

    #[test]
    fn batch3_wm035_private_source_and_program_transplants_are_refused() {
        let mut g = game();
        fill(&mut g);
        field(&mut g, "WM035", 0);
        let target = field(&mut g, "JC125", 0);
        g.remove_dead(&target, RemovalCause::Destroy);
        let mut d = host_declaration(&g);
        let held_source = g.make_card("WM035", 0);
        g.players[0].hand.push(held_source.clone());
        d.source.card = held_source;
        g.effects.clear();
        g.effects.push_back(Effect::Declare { declaration: d });
        assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
        let a = crate::batch3::definition("WM035").unwrap().abilities[0].clone();
        assert!(rules::validate_ability("JC125", &a).is_err());
        let mut definitions = rules::definitions().clone();
        definitions.get_mut("WM035").unwrap().modifiers.clear();
        assert!(rules::validate_definitions(&definitions).is_err());
    }
}

impl Game {
    pub(crate) fn wm035_grants_host(&self, c: &Card) -> bool {
        !c.face_down
            && self.board(&c.id).is_some_and(|(_, live)| {
                live.controller == c.controller
                    && live.definition == c.definition
                    && !live.face_down
            })
            && self
                .regions
                .iter()
                .filter(|r| !r.vacant)
                .flat_map(|r| &r.cards)
                .any(|source| {
                    source.definition == "WM035"
                        && !source.face_down
                        && source.controller == c.controller
                })
    }
    pub(crate) fn wm035_death_sources(&self) -> Vec<SourceSnapshot> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(|c| {
                        c.definition == "WM035"
                            && !c.face_down
                            && self
                                .players
                                .get(c.controller)
                                .is_some_and(|p| !p.eliminated)
                    })
                    .map(move |c| self.source_snapshot(c, Some(r)))
            })
            .collect()
    }
    pub(crate) fn wm035_emit_host_death(
        &mut self,
        sources: &[SourceSnapshot],
        observation: &BlackDeathObservation,
    ) {
        if !observation.subtypes.iter().any(|s| s == "宿主") {
            return;
        }
        for source in sources
            .iter()
            .filter(|source| source.card.controller == observation.dead.controller)
        {
            let mut ability = crate::batch3::definition("WM035").unwrap().abilities[0].clone();
            ability.ops = vec![Op::WM035HostDeathDraw {
                observation: Some(observation.clone()),
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source: source.clone(),
                    ability,
                },
            });
        }
    }
    fn wm035_retired_instance(&self, id: &str) -> bool {
        self.in_play(id).is_none()
            && !self.players.iter().any(|p| {
                p.hand
                    .iter()
                    .chain(&p.deck)
                    .chain(&p.assets)
                    .chain(&p.graveyard)
                    .chain(&p.score_cards)
                    .chain(p.society_zone.card.iter())
                    .any(|c| c.id == id)
            })
            && !self.sealed_cards.iter().any(|s| s.card.id == id)
            && !self
                .stack
                .iter()
                .filter_map(|s| s.card.as_ref())
                .any(|c| c.id == id)
            && !self.world.iter().any(|c| c.id == id)
    }
    pub(crate) fn wm035_observation_key(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        ops: &[Op],
    ) -> RuleResult<Option<(String, String)>> {
        let [Op::WM035HostDeathDraw { observation }] = ops else {
            return Ok(None);
        };
        let o = observation.as_ref().ok_or("宿主死亡观察缺失")?;
        let d = &o.dead;
        let printed = catalog::catalog()
            .cards
            .iter()
            .find(|c| c.id == d.definition && c.kind == "character")
            .ok_or("宿主死亡牌不是已登记角色")?;
        let human = o.subtypes.iter().any(|s| s == "人类");
        let printed_human = printed.subtypes.iter().any(|s| s == "人类");
        let printed_host = printed.subtypes.iter().any(|s| s == "宿主");
        let event_number = o
            .event_id
            .strip_prefix('i')
            .and_then(|s| s.parse::<u64>().ok());
        let subtype_set = o.subtypes.iter().collect::<BTreeSet<_>>();
        if actor >= self.players.len()
            || source.card.definition != "WM035"
            || source.card.controller != actor
            || source.card.face_down
            || source.play_source.is_some()
            || source.region.is_none()
            || source.source_region_instance.is_none()
            || d.id.is_empty()
            || d.face_down
            || d.owner >= self.players.len()
            || d.controller != actor
            || o.turn != self.turn
            || o.region >= self.regions.len()
            || o.region_instance.is_empty()
            || o.region_instance == d.id
            || o.event_id == d.id
            || o.event_id == source.card.id
            || event_number.is_none_or(|n| n == 0 || n > self.sequence)
            || !o.subtypes.iter().any(|s| s == "宿主")
            || (!printed_host && !(printed_human && human))
            || (human && !printed_human)
            || subtype_set.len() != o.subtypes.len()
            || o.subtypes.iter().any(|s| {
                !printed.subtypes.contains(s)
                    && !matches!(
                        s.as_str(),
                        "宿主" | "吸血鬼" | "奴仆" | "邪教徒" | "不死生物" | "骷髅"
                    )
            })
            || !self.wm035_retired_instance(&d.id)
            || (source.card.id == d.id && !same(&source.card, d))
            || (source.card.id != d.id
                && self.in_play(&source.card.id).is_none()
                && !self.wm035_retired_instance(&source.card.id))
            || self
                .regions
                .iter()
                .enumerate()
                .any(|(r, region)| region.card.id == o.region_instance && r != o.region)
        {
            return Err("宿主死亡的类别、原操控者、来源或地区快照不符".into());
        }
        self.mobility_immutable_identity(d)?;
        Ok(Some((source.card.id.clone(), o.event_id.clone())))
    }
    pub(crate) fn validate_wm035_pending(
        &self,
        p: &Pending,
        d: &Declaration,
        stage: &DeclareChoice,
    ) -> RuleResult<()> {
        self.validate_batch3_declaration(d)?;
        if d.source.card.definition != "WM035"
            || d.ability.key != KEY
            || !matches!(stage, DeclareChoice::Accept)
            || p.seat != d.actor
            || p.choice.player_id != player_id(d.actor)
            || p.choice.id.is_empty()
            || p.choice.title != format!("{}：{}", catalog::card("WM035").name, d.ability.label)
            || p.choice.kind != "trigger"
            || p.choice.min != Some(0)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(true)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || !same(
                &p.choice.options,
                &vec![ChoiceOption {
                    id: "accept".into(),
                    label: "发动触发能力".into(),
                    card: None,
                }],
            )
        {
            return Err("宿主死亡的可选声明身份或选择阶段不符".into());
        }
        Ok(())
    }
}
