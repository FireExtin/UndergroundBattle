//! Fixed JZ50-only private death-domain search. No configurable search binding.
use crate::{
    catalog::{self, card, CardDefinition},
    engine::RuleResult,
    model::*,
    rules::{MagicIcon, Op},
};
use std::collections::BTreeSet;
pub(crate) fn jz50_search_match(d: &CardDefinition) -> bool {
    d.kind == "character" && d.magic_icon == MagicIcon::Death
}

impl Game {
    fn jz50_candidate_ids(&self, actor: usize) -> RuleResult<BTreeSet<&str>> {
        let mut ids = BTreeSet::new();
        for c in &self.players[actor].deck {
            let d = catalog::catalog()
                .cards
                .iter()
                .find(|d| d.id == c.definition)
                .ok_or("检索牌库含未准入定义")?;
            if jz50_search_match(d) {
                ids.insert(c.id.as_str());
            }
        }
        Ok(ids)
    }
    fn jz50_search_actor(
        &self,
        frame: &ResolutionFrame,
        source: &str,
        key: &str,
    ) -> RuleResult<usize> {
        if frame.source.card.definition != source
            || frame.ability_key != key
            || frame.source.card.controller != frame.actor
            || frame.actor >= self.players.len()
            || frame.source.card.owner >= self.players.len()
            || frame.source.card.id.is_empty()
            || !matches!(frame.guard, GuardState::Accepted)
            || frame.cursor != 1
            || frame.steps.len() != 1
            || frame.steps[0].context != frame.actor
            || !matches!(frame.steps[0].op, Op::JZ50SearchDeathToGraveyard)
            || !frame.targets.is_empty()
            || !frame.already_paid.is_empty()
        {
            return Err("有限检索来源或声明行动者不符".into());
        }
        Ok(frame.actor)
    }
    pub(crate) fn validate_jz50_search_choice(&self) -> RuleResult<()> {
        let Some(p) = &self.pending else {
            return Ok(());
        };
        let is_search = matches!(
            &p.resolution,
            ChoiceResolution::Frame {
                choice: FrameChoice::JZ50DeathSearch,
                ..
            }
        );
        if !is_search && p.choice.kind != "jz50_death_search" {
            return Ok(());
        }
        let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::JZ50DeathSearch,
        } = &p.resolution
        else {
            return Err("墓穴食尸鬼待选程序无效".into());
        };
        let actor = self.jz50_search_actor(frame, "JZ50", "jz50-reveal-death-search")?;
        let actual = self.jz50_candidate_ids(actor)?;
        let options = p
            .choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<BTreeSet<_>>();
        if self.players[actor].eliminated
            || p.seat != actor
            || p.choice.player_id != format!("p{actor}")
            || p.choice.kind != "jz50_death_search"
            || p.choice.min != Some(0)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(false)
            || options.len() != p.choice.options.len()
            || options != actual
        {
            return Err("墓穴食尸鬼待选快照与行动者牌库不符".into());
        }
        Ok(())
    }
    fn jz50_search_selected(
        &self,
        actor: usize,
        selected: &[String],
        max: usize,
        matches: fn(&CardDefinition) -> bool,
    ) -> RuleResult<()> {
        let distinct = selected.iter().collect::<BTreeSet<_>>();
        if selected.len() > max || selected.len() != distinct.len() {
            return Err("有限检索选择数量或重复实例不符".into());
        }
        // Validate the whole batch before removing a single card or consuming RNG.
        for id in selected {
            let c = self.players[actor]
                .deck
                .iter()
                .find(|c| {
                    &c.id == id
                        && catalog::catalog()
                            .cards
                            .iter()
                            .find(|d| d.id == c.definition)
                            .is_some_and(matches)
                })
                .ok_or("检索的牌库实例或印刷条件已失效")?;
            if c.owner >= self.players.len() {
                return Err("检索牌拥有者无效".into());
            }
        }
        Ok(())
    }
    pub(crate) fn jz50_search_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        let actor = self.jz50_search_actor(&frame, "JZ50", "jz50-reveal-death-search")?;
        if self.players[actor].eliminated {
            return Ok(());
        }
        let candidates = self.jz50_candidate_ids(actor)?;
        let options = self.players[actor]
            .deck
            .iter()
            .filter(|c| candidates.contains(c.id.as_str()))
            .map(|c| self.option(c, actor, None, None))
            .collect::<Vec<_>>();
        // Even an empty candidate list uses this same actor-private choice.
        // Other seats must not infer hidden-library hits from a skipped prompt.
        self.choice(
            actor,
            "jz50_death_search",
            "检索零或一张死亡领域角色置墓并洗牌".into(),
            options,
            0,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::JZ50DeathSearch,
            },
        );
        // Choosing zero completes the accepted search, rather than declining
        // its trigger. The remaining deck must still be shuffled exactly once.
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn jz50_search_complete(
        &mut self,
        frame: &ResolutionFrame,
        selected: &[String],
    ) -> RuleResult<()> {
        let actor = self.jz50_search_actor(frame, "JZ50", "jz50-reveal-death-search")?;
        if self.players[actor].eliminated {
            return Err("检索行动者已退出".into());
        }
        self.jz50_search_selected(actor, selected, 1, jz50_search_match)?;
        if let Some(id) = selected.first() {
            let i = self.players[actor]
                .deck
                .iter()
                .position(|c| &c.id == id)
                .unwrap();
            let c = self.players[actor].deck.remove(i);
            let owner = c.owner;
            let c = self.reset_zone_card(c);
            self.note(format!(
                "{} 将检索的 {} 置于其拥有者墓地",
                self.players[actor].name,
                card(&c.definition).name
            ));
            // Same zone reset/owner-graveyard path as mill. Deck -> graveyard is
            // not a death, so neither remove_dead nor Death event is involved.
            self.players[owner].graveyard.push(c);
        }
        self.shuffle_player(actor);
        self.note(format!(
            "{} 完成墓穴食尸鬼检索并洗牌",
            self.players[actor].name
        ));
        Ok(())
    }
}
