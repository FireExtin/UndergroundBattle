//! XG21's printed, bounded library search. X is the paid source snapshot's
//! advancement count, not a client parameter or a replacement plan's markers.
use crate::{catalog, engine::RuleResult, model::*, rules::Op};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-xg21-renown-search";
const KIND: &str = "xg21-renown-search";
const TITLE: &str = "检索印刷费用小于原计划推进数的声望角色，置于地区一并洗牌";

pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::XG21RenownSearch, .. })
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("幻影英雄检索只能选择原牌库实例，不能改写推进数、地区或费用".into());
    }
    Ok(())
}

fn search_op(op: &Op) -> bool {
    match op {
        Op::XG21RenownSearch => true,
        Op::ForEachLivingPlayer(ops) | Op::ForEachLivingPlayerFromActor(ops) => ops.iter().any(search_op),
        Op::IfTargetExhausted { exhausted, ready, .. } => search_op(exhausted) || search_op(ready),
        _ => false,
    }
}

impl Game {
    fn phantom_search_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_plan_frame(f)?;
        if f.source.card.definition != "XG21" || f.ability_key != KEY || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted) {
            return Err("幻影英雄检索须保留已支付、已接受的完整原程序".into());
        }
        Ok(())
    }
    pub(crate) fn phantom_validate_activation(&self, source: &SourceSnapshot) -> RuleResult<()> {
        self.phantom_supported_library(source.card.controller, source.card.advancement_markers)
    }
    fn phantom_supported_library(&self, actor: usize, markers: u32) -> RuleResult<()> {
        if self.players[actor].deck.iter().any(|c| matches!(c.definition.as_str(), "WM041" | "XG17")
            && catalog::card(&c.definition).cost < markers) {
            return Err("幻影英雄检索的牌库自身条件声望边界尚待明确裁定".into());
        }
        Ok(())
    }
    fn phantom_candidates(&self, f: &ResolutionFrame) -> RuleResult<Vec<ChoiceOption>> {
        self.phantom_supported_library(f.actor, f.source.card.advancement_markers)?;
        // Off-board character cards use their printed keyword. Battlefield
        // grants/denials and internal private-zone orientation do not apply.
        Ok(
        self.players[f.actor].deck.iter().filter(|c| {
            let d = catalog::card(&c.definition);
            d.kind == "character" && d.cost < f.source.card.advancement_markers
                && crate::rules::definition(&c.definition).traits.renown
        }).map(|c| self.option(c, f.actor, None, None)).collect())
    }
    pub(crate) fn validate_phantom_search_state(&self) -> RuleResult<()> {
        let mut sacrifices = BTreeSet::new();
        let mut remember = |f: &ResolutionFrame| -> RuleResult<()> {
            if f.ability_key == KEY && (!sacrifices.insert(f.source.card.id.clone())
                || self.batch3_physical_cards().any(|c| c.id == f.frame_id)) {
                return Err("同一次计划牺牲不能重复检索，物理牌UID不能冒充程序帧".into());
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame { remember(f)?; } }
        for e in &self.effects { if let Effect::Frame { frame } = e { remember(frame)?; } }
        if let Some(Pending { resolution: ChoiceResolution::Frame { frame, .. }, .. }) = &self.pending { remember(frame)?; }
        // A queued executable frame is always before its single search step.
        // Its consumed cursor exists only inside the actor-private pending.
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                if frame.ability_key == KEY && frame.cursor != 0 {
                    return Err("已消费的幻影英雄检索不能重新进入效果队列".into());
                }
            }
        }
        let Some(p) = &self.pending else { return Ok(()); };
        let related = p.choice.kind == KIND || matches!(&p.resolution, ChoiceResolution::Frame { frame, choice }
            if frame.source.card.definition == "XG21" || frame.ability_key == KEY
                || frame.steps.iter().any(|s| search_op(&s.op))
                || matches!(choice, FrameChoice::XG21RenownSearch));
        if !related { return Ok(()); }
        let ChoiceResolution::Frame { frame, choice: FrameChoice::XG21RenownSearch } = &p.resolution else {
            return Err("幻影英雄待选必须使用其固定续体".into());
        };
        self.phantom_search_frame(frame)?;
        let options = self.phantom_candidates(frame)?;
        let count = usize::from(!options.is_empty());
        if p.choice.id.is_empty() || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || p.choice.description != "选择符合数量限制的选项，然后确认。"
            || self.players[frame.actor].eliminated || p.seat != frame.actor
            || p.choice.player_id != format!("p{}", frame.actor) || p.choice.kind != KIND
            || p.choice.title != TITLE || p.choice.min != Some(count) || p.choice.max != Some(count)
            || p.choice.allow_decline != Some(false)
            || serde_json::to_value(&p.choice.options).unwrap() != serde_json::to_value(options).unwrap() {
            return Err("幻影英雄私有候选、数量或原行动者不符".into());
        }
        Ok(())
    }
    pub(crate) fn phantom_search_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        self.phantom_search_frame(&frame)?;
        if self.players[frame.actor].eliminated { return Ok(()); }
        let options = self.phantom_candidates(&frame)?;
        let count = usize::from(!options.is_empty());
        // Keep even an empty search private and explicitly completable, like
        // existing bounded searches. JZ50's optional-failure ruling is not used.
        self.choice(frame.actor, KIND, TITLE.into(), options, count, count, None,
            ChoiceResolution::Frame { frame: Box::new(frame), choice: FrameChoice::XG21RenownSearch });
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn phantom_search_complete(&mut self, f: &ResolutionFrame, selected: &[String]) -> RuleResult<()> {
        self.phantom_search_frame(f)?;
        if self.players[f.actor].eliminated { return Err("幻影英雄检索行动者已退出".into()); }
        let options = self.phantom_candidates(f)?;
        let count = usize::from(!options.is_empty());
        if selected.len() != count || selected.iter().any(|id| !options.iter().any(|o| o.id == *id)) {
            return Err("幻影英雄须选择一张当前合格原牌库实例；无候选时选择为空".into());
        }
        if self.region_live(0) {
            if let Some(id) = selected.first() {
                let i = self.players[f.actor].deck.iter().position(|c| c.id == *id).unwrap();
                let c = self.players[f.actor].deck.remove(i);
                let mut c = self.reset_zone_card(c);
                c.controller = f.actor;
                let (definition, id) = (c.definition.clone(), c.id.clone());
                self.regions[0].cards.push(c);
                self.enter_triggers(f.actor, &definition, &id, false);
            }
        }
        self.shuffle_player(f.actor);
        self.note(format!("{} 完成幻影英雄检索并洗牌", self.players[f.actor].name));
        Ok(())
    }
}
