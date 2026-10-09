//! Three printed death observers. Eligibility is frozen for each removal wave;
//! the existing event queue and stack retain all declaration/resolution timing.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, Event}};

fn relevant_source(source: &SourceSnapshot) -> bool { source.observed_death.is_some() }
fn relevant_frame(frame: &ResolutionFrame) -> bool {
    relevant_source(&frame.source) || frame.ability_key.starts_with("death-observer-")
        || frame.steps.iter().any(|s| rules::death_observer_ops(std::slice::from_ref(&s.op)))
}
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
impl Game {
    pub(crate) fn remove_death_batch(&mut self, deaths: Vec<(String, RemovalCause, SourceSnapshot)>) {
        let previous_effects = self.effects.len();
        let mut observers = Vec::new();
        for (r, region) in self.regions.iter().enumerate() {
            for c in &region.cards {
                if !c.face_down && matches!(c.definition.as_str(), "JC045" | "JC031") {
                    observers.push(self.source_snapshot(c, Some(r)));
                }
            }
        }
        for a in &self.attachments {
            if a.card.definition == "JC090" && !a.card.face_down && self.attachment_host_valid(a) {
                if let Some(r) = self.regions.iter().position(|r| r.card.id == a.host_id) {
                    let mut source = self.source_snapshot(&a.card, Some(r));
                    source.source_region_instance = Some(a.host_id.clone());
                    observers.push(source);
                }
            }
        }
        // Capture actual subtypes before host departure can remove conversion.
        let deaths = deaths.into_iter().map(|(id, cause, source)| {
            let vampire = self.board(&id).is_some_and(|(_, c)|
                self.current_subtypes(c).iter().any(|s| s == "吸血鬼"));
            (id, cause, source, vampire)
        }).collect::<Vec<_>>();
        for (id, cause, source, vampire) in deaths {
            let dead = ObservedCharacterDeath { card: source.card.clone(), vampire };
            if self.remove_dead_with_snapshot(&id, cause, Some(source)) {
                for observer in &observers {
                    if observer.card.definition != "JC045"
                        && (dead.card.controller != observer.card.controller
                            || (observer.card.definition == "JC031" && dead.card.id == observer.card.id))
                    { continue; }
                    let mut source = observer.clone();
                    source.observed_death = Some(dead.clone());
                    self.emit_event(source.card.controller, source, Event::CharacterDeathObserved);
                }
            }
        }
        let mut simultaneous = self.effects.split_off(previous_effects).into_iter().collect::<Vec<_>>();
        simultaneous.sort_by_key(|e| match e {
            Effect::Declare { declaration } if matches!(declaration.ability.event,
                Some(Event::Death | Event::CharacterDeathObserved)) =>
                (self.team(declaration.actor) != self.first_team, declaration.actor),
            _ => (true, usize::MAX),
        });
        self.effects.extend(simultaneous);
    }
    pub(crate) fn death_observer_limit_reached(&self, d: &Declaration) -> bool {
        d.ability.event == Some(Event::CharacterDeathObserved) && d.source.card.definition == "JC031"
            && self.turn_ability_usage.iter().any(|u| u.source_instance == d.source.card.id
                && u.ability_key == d.ability.key && u.turn == self.turn && u.uses >= 1)
    }
    fn validate_observer_source(&self, actor: usize, source: &SourceSnapshot) -> RuleResult<()> {
        let c = &source.card;
        let dead = source.observed_death.as_ref().ok_or("死亡旁观缺少死亡快照")?;
        if rules::death_observer_definition(&c.definition).is_none()
            || actor >= self.players.len() || actor != c.controller || c.owner >= self.players.len()
            || c.id.is_empty() || c.face_down || source.play_source.is_some()
            || !source.region.is_some_and(|r| r < self.regions.len())
            || dead.card.id.is_empty() || dead.card.face_down
            || dead.card.owner >= self.players.len() || dead.card.controller >= self.players.len()
            || !catalog::catalog().cards.iter().any(|d| d.id == dead.card.definition && d.kind == "character")
            || (c.definition != "JC045" && dead.card.controller != actor)
            || (c.definition == "JC031" && c.id == dead.card.id)
            || (c.id == dead.card.id && (c.definition != "JC045" || !same(c, &dead.card)))
            || (dead.card.time_markers > 0 && dead.card.definition != "JC045")
            || (c.definition != "JC045" && c.time_markers != 0)
        { return Err("死亡旁观来源、死亡资格或冻结行动者无效".into()); }
        if c.definition == "JC090" {
            if !source.attachment_host_instance.as_ref().is_some_and(|id| !id.is_empty())
                || source.attachment_host_instance != source.source_region_instance {
                return Err("灵魂契约原地区实例无效".into());
            }
        } else if source.attachment_host_instance.is_some() || source.source_region_instance.is_some() {
            return Err("角色死亡旁观含非准入宿主或地区绑定".into());
        }
        Ok(())
    }
    fn validate_observer_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if !relevant_source(&d.source) && d.ability.event != Some(Event::CharacterDeathObserved)
            && !d.ability.key.starts_with("death-observer-") && !rules::death_observer_ops(&d.ability.ops)
            && !d.ability.modes.iter().any(|m| rules::death_observer_ops(&m.ops)) { return Ok(()); }
        self.validate_observer_source(d.actor, &d.source)?;
        rules::validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event != Some(Event::CharacterDeathObserved) {
            return Err("死亡上下文不可移植到其他声明".into());
        }
        Ok(())
    }
    fn validate_observer_frame(&self, f: &ResolutionFrame, discard_seat: Option<usize>) -> RuleResult<()> {
        if !relevant_frame(f) { return Ok(()); }
        self.validate_observer_source(f.actor, &f.source)?;
        let spec = rules::death_observer_definition(&f.source.card.definition).unwrap()
            .abilities.into_iter().find(|a| a.event == Some(Event::CharacterDeathObserved)).unwrap();
        if f.frame_id.is_empty() || f.ability_key != spec.key || f.steps.len() != 1 || f.steps[0].context != f.actor
            || !same(&f.steps[0].op, &spec.ops[0]) || !f.already_paid.is_empty()
            || f.chosen_region.is_some() || f.targets.len() != spec.targets.len()
        { return Err("死亡旁观帧程序或绑定无效".into()); }
        for (target, slot) in f.targets.iter().zip(&spec.targets) {
            if !same(&target.spec, slot) || target.region_instance.is_some()
                || !target.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok())
                    .is_some_and(|seat| seat < self.players.len() && target.id == format!("p{seat}"))
                || target.public.instance_id != target.id {
                return Err("唤灵师目标玩家绑定无效".into());
            }
        }
        if f.source.card.definition == "JC031" && !self.turn_ability_usage.iter().any(|u|
            u.source_instance == f.source.card.id && u.ability_key == f.ability_key
                && u.turn == self.turn && u.uses == 1) {
            return Err("唤灵师已声明帧缺少本回合限次记录".into());
        }
        let valid_guard = if let Some(seat) = discard_seat {
            f.source.card.definition == "JC031" && !f.source.observed_death.as_ref().unwrap().vampire
                && f.targets[0].id == format!("p{seat}") && matches!(f.guard, GuardState::Accepted) && f.cursor == 1
        } else { matches!(f.guard, GuardState::Unchecked) && f.cursor == 0 };
        if !valid_guard { return Err("死亡旁观帧守卫或游标无效".into()); }
        Ok(())
    }
    pub(crate) fn validate_death_observers(&self) -> RuleResult<()> {
        for c in self.regions.iter().flat_map(|r| &r.cards) {
            if c.time_markers > 0 && (c.definition != "JC045" || c.face_down) {
                return Err("时间标志只能存在于明置钟摆祭司".into());
            }
        }
        for c in self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
            .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter()))
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.regions.iter().map(|r| &r.card)).chain(&self.world)
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref())) {
            if c.time_markers != 0 { return Err("离场或非角色实例不能保留时间标志".into()); }
        }
        for effect in &self.effects {
            if let Effect::Bury { card } = effect {
                if card.time_markers != 0 { return Err("离场结算牌不能保留时间标志".into()); }
            }
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_observer_frame(f, None)?;
                if relevant_frame(f) && (item.id != f.frame_id || item.controller != f.actor
                    || item.card.is_some() || item.deploy_region.is_some() || item.reveal
                    || item.target != f.targets.first().map(|t| t.id.clone())) {
                    return Err("死亡旁观堆栈容器无效".into());
                }
            }
        }
        for effect in &self.effects {
            match effect {
                Effect::Declare { declaration } => self.validate_observer_declaration(declaration)?,
                Effect::Frame { frame } => self.validate_observer_frame(frame, None)?,
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, stage } => {
                    self.validate_observer_declaration(d)?;
                    if relevant_source(&d.source) {
                        let target = d.source.card.definition == "JC031";
                        if p.seat != d.actor || p.choice.player_id != format!("p{}", d.actor)
                            || p.choice.kind != "trigger" || p.choice.min != Some(0) || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
                            || !(if target { matches!(stage, DeclareChoice::Target) } else { matches!(stage, DeclareChoice::Accept) })
                            || self.death_observer_limit_reached(d) {
                            return Err("死亡旁观声明待选无效".into());
                        }
                        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
                        let expected = if target { self.players.iter().filter(|p| !p.eliminated)
                            .map(|p| format!("p{}", p.seat)).collect::<Vec<_>>() } else { vec!["accept".into()] };
                        if ids != expected { return Err("死亡旁观声明选项无效".into()); }
                    }
                }
                ChoiceResolution::Frame { frame, choice } if relevant_frame(frame) => {
                    let FrameChoice::Discard { seat, redraw: false } = choice else {
                        return Err("死亡旁观不支持此暂停容器".into());
                    };
                    self.validate_observer_frame(frame, Some(*seat))?;
                    if *seat >= self.players.len() || p.seat != *seat || p.choice.player_id != format!("p{seat}")
                        || self.players[*seat].eliminated || p.choice.kind != "discard"
                        || p.choice.min != Some(1) || p.choice.max != Some(1)
                        || p.choice.options.is_empty() || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
                        || p.choice.allow_decline != Some(false)
                    { return Err("唤灵师弃牌待选无效".into()); }
                    let expected = self.players[*seat].hand.iter().map(|c| self.option(c, *seat, None, None)).collect::<Vec<_>>();
                    if !same(&p.choice.options, &expected) { return Err("唤灵师弃牌选项与真实手牌不符".into()); }
                }
                _ => {},
            }
        }
        Ok(())
    }
}
