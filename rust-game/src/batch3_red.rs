//! Closed third-batch red continuations. A graveyard selection is made when
//! resolving, without introducing a target absent from the printed text.
use crate::{catalog, engine::RuleResult, model::*};

const WM033_KEY: &str = "batch3-wm033-return";
const WM033_TITLE: &str = "钟楼怪客：选择一张法术或灾难牌移回手中";

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::*;

    fn until_pending(g: &mut Game) {
        for _ in 0..48 {
            if g.pending.is_some() {
                restore(g);
                return;
            }
            pass(g);
        }
        panic!("expected a bounded third-red choice");
    }
    fn activate(source: &str) -> Action {
        Action {
            card_id: Some(source.into()),
            cost_selected: Some(vec![source.into()]),
            ..Action::new("activate")
        }
    }

    #[test]
    fn batch3_wm033_entry_and_return_include_disaster_character_without_target_or_exhaust_cost() {
        let mut g = game();
        fund(&mut g, 0, "JC047", 4);
        let held = held(&mut g, "WM033", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(held),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        top(&mut g);
        let source = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "WM033")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.board(&source).unwrap().1.time_markers, 1);
        g.board_mut(&source).unwrap().exhausted = true;
        let disaster = g.make_card("WM034", 0);
        let selected = disaster.id.clone();
        let spell = g.make_card("JC047", 0);
        let spell_id = spell.id.clone();
        let magic_only = g.make_card("JC018", 0);
        let magic_only_id = magic_only.id.clone();
        g.players[0].graveyard = vec![disaster, spell, magic_only];
        let foreign = g.make_card("JC008", 2);
        let foreign_id = foreign.id.clone();
        g.players[2].graveyard.push(foreign);
        act(&mut g, 0, activate(&source));
        assert!(g
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .targets
            .is_empty());
        until_pending(&mut g);
        let p = g.pending.clone().unwrap();
        assert_eq!(
            p.choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>(),
            vec![selected.clone(), spell_id]
        );
        assert!(!p
            .choice
            .options
            .iter()
            .any(|o| o.id == magic_only_id || o.id == foreign_id));
        let choice = Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![selected.clone()]),
            ..Action::new("choose")
        };
        reject(&mut g, 1, choice.clone());
        let mut polluted = choice.clone();
        polluted.cost_selected = Some(vec![source.clone()]);
        reject(&mut g, 0, polluted);
        let mut forged = g.clone();
        forged.pending.as_mut().unwrap().choice.min = Some(0);
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        act(&mut g, 0, choice);
        let returned = g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "WM034")
            .unwrap();
        assert_ne!(returned.id, selected);
        assert_eq!(returned.owner, 0);
        assert_eq!(returned.controller, 0);
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
        assert!(g.board(&source).unwrap().1.exhausted);
        g.board_mut(&source).unwrap().time_markers = 1;
        reject(&mut g, 0, activate(&source));
    }

    #[test]
    fn batch3_wm033_freezes_actor_and_empty_graveyard_has_no_fake_selection() {
        let mut g = game();
        let source = field(&mut g, "WM033", 0);
        g.batch3_entry_keywords("WM033", &source);
        let spell = g.make_card("JC008", 0);
        let selected = spell.id.clone();
        g.players[0].graveyard.push(spell);
        let other = g.make_card("JC047", 2);
        g.players[2].graveyard.push(other);
        act(&mut g, 0, activate(&source));
        g.board_mut(&source).unwrap().controller = 2;
        until_pending(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        choose(&mut g, vec![selected]);
        assert!(g.players[0].hand.iter().any(|c| c.definition == "JC008"));
        assert!(g.players[2].hand.is_empty());

        let mut g = game();
        let source = field(&mut g, "WM033", 0);
        g.batch3_entry_keywords("WM033", &source);
        act(&mut g, 0, activate(&source));
        top(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
    }

    fn mill_wm033(personal_moon: bool, ally_moon: bool) -> (Game, String) {
        let mut g = game();
        let host = field(&mut g, "JC125", 2);
        let mill = g.make_card("BQ091", 0);
        let source = mill.id.clone();
        g.attachments.push(Attachment {
            card: mill,
            host_id: g.regions[0].card.id.clone(),
        });
        let rumor = g.make_card("WM033", 0);
        let next = g.make_card("JC125", 0);
        g.players[0].deck = vec![rumor, next];
        g.add_society_markers(0, 0, u32::from(personal_moon));
        g.add_society_markers(1, 0, u32::from(ally_moon));
        g.jc035_preparation = None;
        g.begin_window(Window::Prepare);
        g.drive().unwrap();
        choose(&mut g, vec![source]);
        (g, host)
    }

    #[test]
    fn batch3_wm033_rumor_uses_personal_moon_and_restores_real_mill_replacement() {
        let (mut g, host) = mill_wm033(true, false);
        until_pending(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.kind, "rumor");
        assert_eq!(p.choice.title, "钟楼怪客：可改为封印在目标角色上");
        let mut forged = g.clone();
        forged.players[0].society_zone.secrecy = 0;
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        choose(&mut g, vec![host]);
        assert_eq!(g.players[0].deck.len(), 1);
        assert_eq!(g.players[0].deck[0].definition, "JC125");
        assert!(g.players[0].graveyard.is_empty());
        assert_eq!(g.sealed_cards.len(), 1);
        assert_eq!(g.sealed_cards[0].card.definition, "WM033");
        restore(&mut g);
        let (mut g, _) = mill_wm033(false, true);
        top(&mut g);
        assert!(!g.pending.as_ref().is_some_and(|p| p.choice.kind == "rumor"));
        assert!(g.sealed_cards.is_empty());
        assert_eq!(g.players[0].graveyard[0].definition, "WM033");
        restore(&mut g);
    }
}

impl Game {
    fn wm033_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        self.players[f.actor]
            .graveyard
            .iter()
            .filter(|c| {
                // WM034 is a character with the printed 灾难 subtype and qualifies;
                // a magic-domain icon alone does not make a card a 法术 card.
                catalog::catalog()
                    .cards
                    .iter()
                    .find(|d| d.id == c.definition)
                    .is_some_and(|d| {
                        d.subtypes
                            .iter()
                            .any(|s| matches!(s.as_str(), "法术" | "灾难"))
                    })
            })
            .map(|c| self.option(c, f.actor, None, None))
            .collect()
    }
    pub(crate) fn wm033_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_batch3_frame(&f)?;
        if f.source.card.definition != "WM033" || f.ability_key != WM033_KEY || f.cursor != 1 {
            return Err("钟楼怪客的回收续体与固定程序不符".into());
        }
        if self.players[f.actor].eliminated {
            return Ok(());
        }
        let options = self.wm033_options(&f);
        if options.is_empty() {
            return Ok(());
        }
        self.choice(
            f.actor,
            "graveyard",
            WM033_TITLE.into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::WM033GraveSpell,
            },
        );
        Ok(())
    }
    pub(crate) fn validate_wm033_pending(
        &self,
        p: &Pending,
        f: &ResolutionFrame,
    ) -> RuleResult<()> {
        self.validate_batch3_frame(f)?;
        if f.source.card.definition != "WM033"
            || f.ability_key != WM033_KEY
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || self.players[f.actor].eliminated
        {
            return Err("钟楼怪客的待选来源、阶段或操控者不符".into());
        }
        let options = self.wm033_options(f);
        if p.seat != f.actor
            || p.choice.player_id != player_id(f.actor)
            || p.choice.id.is_empty()
            || p.choice.kind != "graveyard"
            || p.choice.title != WM033_TITLE
            || p.choice.min != Some(1)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(false)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || options.is_empty()
            || !same(&options, &p.choice.options)
        {
            return Err("钟楼怪客的回收选择、数量或费用续体不符".into());
        }
        Ok(())
    }
    pub(crate) fn wm033_finish(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_batch3_frame(f)?;
        if f.source.card.definition != "WM033"
            || f.ability_key != WM033_KEY
            || f.cursor != 1
            || chooser != f.actor
            || selected.len() != 1
            || !self.wm033_options(f).iter().any(|o| o.id == selected[0])
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: Some(selected.to_vec()),
                    ..Action::new("choose")
                })
        {
            return Err("钟楼怪客只能回收本方墓地中的一个真实合法选项".into());
        }
        let index = self.players[f.actor]
            .graveyard
            .iter()
            .position(|c| c.id == selected[0])
            .ok_or("所选牌已离开墓地")?;
        let card = self.take_grave_observed(f.actor,index);
        let card = self.reset_zone_card(card);
        self.players[f.actor].hand.push(card);
        Ok(())
    }
}
