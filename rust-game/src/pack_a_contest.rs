//! Pack A: five finite printed programs and their shared hidden-contest hooks.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::ContestOutcome, rules::*};
use serde::Serialize;
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["WM021", "WM022", "WM032", "WM061", "BQ082"];
pub(crate) const SHADOW_REVEAL: &str = "pack-a-wm022-reveal";
pub(crate) const SHADOW_WIN: &str = "pack-a-wm022-first-win";
pub(crate) const PREACHER_WIN: &str = "pack-a-wm032-win";
pub(crate) const EARTH_SHOWN: &str = "pack-a-wm061-shown";
pub(crate) const EARTH: &str = "pack-a-wm061-mill";
pub(crate) const POISON: &str = "pack-a-bq082-enter";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }

pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {
    if id==crate::mswm02::ID{return key==crate::mswm02::CONTEST&&bonus==0;}
    if id == crate::cq09::ID && key == crate::cq09::REVEAL && bonus == 0 { return true; }
    if id == "CQ25" && key == crate::next20_black_wraiths::CONTEST && bonus == 0 { return true; }
    if id == "WM057" && key == crate::import20_broker::ACT && bonus == 0 { return true; }
    if crate::import20_observers::admitted_start(id, key, bonus) { return true; }
    if crate::import20_contest::admitted_start(id, key, bonus) { return true; }
    (id=="WM037" && key==crate::batch4_clock_spell::PLAY && bonus==0) || crate::batch4_vampire_contest::admitted_start(id, key, bonus) || matches!((id,key,bonus), ("WM022",SHADOW_REVEAL,0) | ("BQ082",POISON,2))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let abilities = match id {
        "WM021" => vec![],
        "WM022" => vec![
            ability(SHADOW_REVEAL, "现身强制触发：进行隐秘对抗", Timing::Fast,vec![],vec![],vec![Op::MysticContest { actor_bonus: 0 }],Some(Event::Reveal)),
            ability(SHADOW_WIN, "本回合首次赢得隐秘对抗：目标玩家弃1", Timing::Fast,vec![],
                vec![target(Zone::Player,EntityKind::Any,Relation::Any,Range::Anywhere)],
                vec![Op::MysticWin { contest_id: None },Op::Discard { player: PlayerRef::Target(0),count:Some(1),redraw:false,optional:false }],Some(Event::MysticContestWon)),
        ],
        "WM032" => vec![ability(PREACHER_WIN,"赢得隐秘对抗：所有已毁灭地区放本方势力1",Timing::Fast,vec![],vec![],
            vec![Op::MysticWin { contest_id: None },Op::MysticDestroyedInfluence],Some(Event::MysticContestWon))],
        "WM061" => {
            let mut a = ability(EARTH,"行动：每位玩家顶五张牌置于其墓地",Timing::Standard,vec![],vec![],
                vec![Op::ForEachLivingPlayer(vec![Op::MoveDeckTopToGraveyard { player:PlayerRef::Context,count:5 }])],None);
            a.play_only = true; vec![a]
        },
        "BQ082" => {
            let mut t = target(Zone::Graveyard,EntityKind::Character,Relation::OwnedByActor,Range::Anywhere);
            t.predicate = Some(TargetPredicate::PackAOwnGraveCharacter);
            vec![ability(POISON,"进场：隐秘对抗本人+2，获胜则原墓地目标横置进原地区",Timing::Fast,vec![],vec![t],
                vec![Op::MysticContest { actor_bonus:2 }],Some(Event::Enter))]
        },
        _ => return None,
    };
    Some(with_abilities(abilities))
}
pub(crate) fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    if id=="WM037" { return crate::batch4_clock_spell::definition().abilities.into_iter().find(|b|b.key==a.key); }
    let mut expected = definition(id)?.abilities.into_iter().find(|b| b.key == a.key)?;
    if let Some(Op::MysticWin { contest_id }) = a.ops.first() {
        if matches!(a.key.as_str(),SHADOW_WIN|PREACHER_WIN) { expected.ops[0] = Op::MysticWin { contest_id:contest_id.clone() }; }
    }
    Some(expected)
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::MysticContest { .. } | Op::MysticWin { .. } | Op::MysticDestroyedInfluence => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("pack-a-") || a.event == Some(Event::MysticContestWon)
        || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
        || a.targets.iter().any(|t| t.predicate == Some(TargetPredicate::PackAOwnGraveCharacter))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if related(a) && canonical(id,a).is_none_or(|expected| !same(a,&expected)) {
        return Err("隐秘对抗能力只能保留已核对的完整原程序".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    let expected = match (id,a.kind.as_str()) {
        ("WM061","play") => Action { card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("play") },
        (_,"deploy"|"conceal") if id != "WM061" => Action { card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind) },
        (_,"reveal") if id != "WM061" => Action { card_id:a.card_id.clone(),..Action::new("reveal") },
        _ => return Err("本卡没有该行动程序".into()),
    };
    if *a != expected || id == "WM061" && a.ability_id.as_deref().is_some_and(|key| key != EARTH) {
        return Err("本卡行动不可追加目标、数量或费用载荷".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let related = match &p.resolution {
        ChoiceResolution::Declare { declaration:d,.. } => CARDS.contains(&d.source.card.definition.as_str()),
        ChoiceResolution::Frame { frame:f,.. } => CARDS.contains(&f.source.card.definition.as_str()), _ => false,
    };
    if related && *a != (Action { choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose") }) {
        return Err("隐秘卡待选只能提交当前选择，不能夹带其他行动字段".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn pack_a_grave_target(&self, actor: usize, source: &SourceSnapshot, c: &Card) -> bool {
        source.card.definition == "BQ082" && c.owner == actor && !c.face_down
            && catalog::card(&c.definition).kind == "character"
            && self.players.get(actor).is_some_and(|p| p.graveyard.iter().any(|live| live == c))
    }
    fn pack_a_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str()) || actor >= self.players.len() || c.owner >= self.players.len()
            || c.controller != actor || !self.mystic_instance_valid(&c.id) || s.death_echo.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.attachment_host_instance.is_some() || s.region.is_some_and(|r| r >= self.regions.len())
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)| self.regions.iter().enumerate().any(|(other,live)| live.card.id == *id && other != r))
            || s.play_source.is_none() && s.region.is_some() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none() {
            return Err("隐秘卡须保留原拥有者、操控者及来源实体".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn pack_a_win_record(&self, actor: usize, id: &str, ops: &[Op]) -> RuleResult<&ContestOutcome> {
        let Some(Op::MysticWin { contest_id:Some(event) }) = ops.first() else { return Err("隐秘获胜触发缺少实际结果".into()); };
        let o = self.mystic_history.iter().find(|o| o.contest_id == *event).ok_or("隐秘获胜触发引用不存在的对抗")?;
        if o.turn != self.turn || o.winner_team != Some(self.team(actor)) || !o.participants.contains(&actor)
            || id == "WM022" && !o.first_winners.contains(&actor) {
            return Err("隐秘获胜须属于原个人玩家；首次获胜不能转移或重置".into());
        }
        self.validate_mystic_outcome(o)?; Ok(o)
    }
    pub(crate) fn validate_pack_a_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if crate::cq09::related(&d.ability) { return self.validate_cq09_declaration(d); }
        // A proved granted program belongs to the granting mechanism, even
        // when its recipient also has a card-specific native program.
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if crate::next20_black_wraiths::related(&d.ability) { return self.validate_next20_black_declaration(d); }
        if crate::next20_blue_vampires::related(&d.ability) { return self.validate_next20_blue_declaration(d); }
        if crate::import20_tax_ban::related(&d.source.card.definition,&d.ability) { return self.validate_tax_ban_declaration(d); }
        if crate::import20_sentinel::related(&d.source.card.definition,&d.ability) { return self.validate_sentinel_pair_declaration(d); }
        if crate::import20_industry::related(&d.source.card.definition,&d.ability) { return self.validate_industry_declaration(d); }
        if crate::import20_carmen::related(&d.source.card.definition, &d.ability) { return self.validate_cq24_declaration(d); }
        if crate::import20_cultist::related(&d.source.card.definition, &d.ability) { return self.validate_xg13_declaration(d); }
        if crate::import20_broker::CARDS.contains(&d.source.card.definition.as_str()) || crate::import20_broker::related(&d.ability) {
            return self.validate_import20_broker_declaration(d);
        }
        if crate::import20_vampires::CARDS.contains(&d.source.card.definition.as_str()) || crate::import20_vampires::related(&d.ability) {
            return self.validate_import20_vampire_declaration(d);
        }
        if crate::import20_observers::related(&d.source.card.definition, &d.ability) {
            return self.validate_import20_observer_declaration(d);
        }
        if crate::batch4_intelligence::related(&d.ability) { return self.validate_wm044_declaration(d); }
        if crate::batch4_vampire_contest::key(&d.ability.key) || crate::batch4_vampire_contest::owns(&d.ability.ops) { return self.validate_b_contest_declaration(d); }
        if !CARDS.contains(&d.source.card.definition.as_str()) && !related(&d.ability) { return Ok(()); }
        self.pack_a_source(d.actor,&d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some() || d.source.region.is_none()
            || d.source.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id))
            || canonical(&d.source.card.definition,&d.ability).is_none_or(|expected| !same(&d.ability,&expected)) || d.ability.event.is_none() {
            return Err("隐秘卡触发声明须保留完整原程序与原地区".into());
        }
        if d.ability.event == Some(Event::MysticContestWon) { self.pack_a_win_record(d.actor,&d.source.card.definition,&d.ability.ops)?; }
        Ok(())
    }
    pub(crate) fn pack_a_forced_declare(&mut self, d: &Declaration) -> RuleResult<bool> {
        if d.source.card.definition != "WM022" || d.ability.key != SHADOW_REVEAL { return Ok(false); }
        self.validate_pack_a_declaration(d)?;
        if !self.players[d.actor].eliminated {
            let f = self.make_frame(d.actor,d.source.clone(),&d.ability,vec![],vec![],None);
            self.dispatch_frame(f,d.ability.label.clone(),None,ResponsePolicy::Respondable);
        }
        Ok(true)
    }
    pub(crate) fn validate_pack_a_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition==crate::test034::ID{return self.test034_frame(f);}
        if f.source.card.definition==crate::mswm02::ID||f.ability_key.starts_with("mswm02-"){return self.mswm02_frame(f);}
        if crate::cq09::frame_related(f) { return self.validate_cq09_frame(f); }
        self.validate_next20_black_play_frame(f)?;
        if crate::next20_black_wraiths::frame_related(f) { return self.validate_next20_black_program_frame(f); }
        self.validate_next20_blue_play_frame(f)?;
        if crate::next20_blue_vampires::frame_related(f) { return self.validate_next20_blue_frame(f); }
        self.validate_resource_frame(f)?;
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if crate::import20_tax_ban::frame_related(f) { return self.validate_tax_ban_frame(f); }
        if crate::import20_sentinel::frame_related(f) { return self.validate_sentinel_pair_frame(f); }
        if crate::import20_industry::frame_related(f) { return self.validate_industry_frame(f); }
        if crate::import20_carmen::frame_related(f) { return self.validate_cq24_frame(f); }
        if crate::import20_tattoo::frame_related(f) { return self.validate_wm064_frame(f); }
        if crate::import20_cultist::frame_related(f) { return self.validate_xg13_frame(f); }
        if crate::import20_broker::frame_related(f) { return self.validate_import20_broker_frame(f); }
        if crate::import20_vampires::frame_related(f) { return self.validate_import20_vampire_frame(f); }
        if crate::import20_observers::frame_related(f) { return self.validate_import20_observer_frame(f); }
        if crate::import20_contest::CARDS.contains(&f.source.card.definition.as_str())
            || f.ability_key.starts_with("import20-")
            || f.steps.iter().any(|s| crate::import20_contest::owns(std::slice::from_ref(&s.op))) {
            return self.validate_import20_frame(f);
        }
        if f.source.card.definition=="WM037" {return self.validate_wm037_frame(f);}
        if f.ability_key == crate::wm043::SHOWN || f.steps.iter().any(|s| matches!(s.op, Op::WM043ShownLock { .. })) { return self.validate_wm043_post(f); }
        if crate::batch4_intelligence::frame_related(f) { return self.validate_wm044_frame(f); }
        if crate::batch4_vampire_contest::related(f) { return self.validate_b_contest_frame(f); }
        let id = f.source.card.definition.as_str();
        if !CARDS.contains(&id) && !f.ability_key.starts_with("pack-a-") && !f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op))) { return Ok(()); }
        self.pack_a_source(f.actor,&f.source)?;
        if !self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(source, frame)| source < frame) || f.cursor > f.steps.len()
            || f.cursor == 0 && !matches!(f.guard,GuardState::Unchecked) || f.cursor > 0 && !matches!(f.guard,GuardState::Accepted) {
            return Err("隐秘卡执行帧身份、阶段或守卫无效".into());
        }
        if matches!(f.ability_key.as_str(),"deploy"|"reveal") {
            if id == "WM061" || !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some())
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len() <= catalog::card(id).cost as usize && ids.iter().all(|n| self.mystic_instance_valid(n) && *n != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("隐秘角色派遣或现身须保留原地区及真实费用收据".into());
            }
            return Ok(());
        }
        if f.ability_key == EARTH_SHOWN {
            let Some(crate::mystic_contest::MysticContest { phase: crate::mystic_contest::MysticPhase::After { outcome,next_shown,post_frame },.. }) = self.mystic_states().find(|s|
                matches!(&s.phase, crate::mystic_contest::MysticPhase::After { post_frame:Some(p),.. } if p.frame_id == f.frame_id))
                else { return Err("土归土展示后置缺少原对抗".into()); };
            let shown = next_shown.checked_sub(1).and_then(|i| outcome.shown.get(i)).ok_or("土归土展示后置游标无效")?;
            let expected = self.source_snapshot(&shown.card,None);
            if id != "WM061" || !same(&f.source,&expected) || f.actor != shown.seat || f.cursor != 1 || !matches!(f.guard,GuardState::Accepted)
                || !f.targets.is_empty() || !f.already_paid.is_empty() || f.chosen_region.is_some()
                || !matches!(f.steps.as_slice(),[Step { context,op: Op::MoveDeckTopToGraveyard { player:PlayerRef::Actor,count:1 } }] if *context == shown.seat)
                || post_frame.as_ref().is_none_or(|p| !same(p.as_ref(),f)) { return Err("土归土后置仅能消费本次本人展示一次".into()); }
            return Ok(());
        }
        let mut a = definition(id).and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key)).ok_or("隐秘卡未知执行程序")?;
        if matches!(f.ability_key.as_str(),SHADOW_WIN|PREACHER_WIN) {
            if let Some(Step { op:Op::MysticWin { contest_id },.. }) = f.steps.first() { a.ops[0] = Op::MysticWin { contest_id:contest_id.clone() }; }
        }
        if id == "WM061" {
            if f.source.card.face_down || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.play_source.is_some()
                || f.chosen_region.is_some() || !f.targets.is_empty() || f.steps.len() != self.players.len()
                || f.steps.iter().enumerate().any(|(seat, s)| s.context != seat)
                || f.steps.iter().any(|s| s.context >= self.players.len() || !matches!(s.op,Op::MoveDeckTopToGraveyard { player:PlayerRef::Context,count:5 }))
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len() <= 2 && ids.iter().all(|n| self.mystic_instance_valid(n) && *n != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("土归土只能保留实际费用与按席位执行的顶五张入墓程序".into());
            }
            return Ok(());
        }
        self.validate_pack_a_declaration(&Declaration { actor:f.actor,source:f.source.clone(),ability:a.clone() })?;
        if !f.already_paid.is_empty() || f.chosen_region.is_some() || f.steps.len() != a.ops.len()
            || f.steps.iter().zip(&a.ops).any(|(s,op)| s.context != f.actor || !same(&s.op,op)) || f.targets.len() != a.targets.len() {
            return Err("隐秘触发只能消费原获胜记录、目标及完整程序".into());
        }
        for (t,spec) in f.targets.iter().zip(&a.targets) {
            if t.id.is_empty() || !same(&t.spec,spec) || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid"
                || t.public.invalid_reason.is_some() || t.region_instance.is_some() || t.public.region.is_some() {
                return Err("隐秘卡冻结目标摘要被改写".into());
            }
            if spec.zone == Zone::Player {
                let seat = t.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("隐秘触发玩家目标无效")?;
                if t.id != player_id(seat) || t.public.kind != "player" || t.public.controller.as_deref() != Some(t.id.as_str()) || t.public.owner.is_some() { return Err("隐秘触发玩家目标必须绑定原玩家".into()); }
            } else if spec.zone != Zone::Graveyard || t.public.kind != "character" || t.public.owner != Some(player_id(f.actor)) {
                return Err("毒药专家只绑定本人墓地的原角色牌".into());
            }
        }
        Ok(())
    }
    pub(crate) fn pack_a_rumor_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        if f.source.card.definition == "BQ093" && f.ability_key == crate::import20_industry::MILL {
            self.validate_industry_frame(f)?;
            return Ok(true);
        }
        if !matches!(f.ability_key.as_str(), EARTH_SHOWN | SHADOW_WIN | crate::import20_broker::SHOWN) { return Ok(false); }
        self.validate_pack_a_frame(f)?; Ok(true)
    }
    pub(crate) fn pack_a_observe_mystic_win(&mut self, outcome: &ContestOutcome) {
        let Some(team) = outcome.winner_team else { return; };
        let mut sources = vec![];
        for (r, region) in self.regions.iter().enumerate().filter(|(_,r)| !r.vacant) {
            for c in &region.cards {
                if !c.face_down && self.team(c.controller) == team && outcome.participants.contains(&c.controller)
                    && (c.definition == "WM032" || c.definition == "WM022" && outcome.first_winners.contains(&c.controller)) {
                    sources.push(self.source_snapshot(c,Some(r)));
                }
            }
        }
        for source in sources {
            let key = if source.card.definition == "WM022" { SHADOW_WIN } else { PREACHER_WIN };
            let mut a = definition(&source.card.definition).unwrap().abilities.into_iter().find(|a| a.key == key).unwrap();
            a.ops[0] = Op::MysticWin { contest_id:Some(outcome.contest_id.clone()) };
            self.effects.push_back(Effect::Declare { declaration:Declaration { actor:source.card.controller,source,ability:a } });
        }
    }
    pub(crate) fn pack_a_destroyed_influence(&mut self, f: &ResolutionFrame) {
        let regions = self.regions.iter().enumerate().filter(|(_,r)| !r.vacant && r.destroyed).map(|(r,_)| r).collect::<Vec<_>>();
        for region in regions { self.place_influence(f.actor,region,1); }
    }
    pub(crate) fn pack_a_mystic_after(&mut self, f: &ResolutionFrame, outcome: &ContestOutcome) -> RuleResult<()> {
        if f.ability_key == POISON && outcome.winner_team == Some(self.team(f.actor)) {
            let Some(region) = self.original_region(&f.source) else { return Ok(()); };
            let Some(target) = f.targets.first().filter(|t| self.valid_bound_target(f.actor,&f.source,t)) else { return Ok(()); };
            let Some(index) = self.players[f.actor].graveyard.iter().position(|c| c.id == target.id) else { return Ok(()); };
            let c = self.players[f.actor].graveyard.remove(index); let mut c = self.reset_zone_card(c); c.exhausted = true;
            let id = c.id.clone(); let definition = c.definition.clone();
            self.regions[region].cards.push(c);
            self.enter_triggers(f.actor,&definition,&id,false);
        }
        Ok(())
    }
    pub(crate) fn pack_a_transaction_matches(&self, f: &ResolutionFrame, c: &Card) -> bool {
        let mut expected = f.source.card.clone();
        // Mirror existing fresh() card fields without allocating identities or
        // changing payment, RNG, modifiers, or the transaction itself.
        expected.id = c.id.clone(); expected.face_down = false;
        expected.advancement_markers = 0; expected.named_character = None; expected.chosen_number = None; expected.chosen_subtype = None;
        expected.time_markers = 0; expected.charge_markers = 0; expected.permanent_combat_markers = 0;
        expected.permanent_defense_markers = 0; expected.lock_markers = 0;
        // fresh() allocates the transaction after its original source, and
        // push_stack() allocates the immediately following frame UID. Checking
        // only the interval would still admit a retired UID after the source.
        self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&c.id))
            .zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|((source, transaction), frame)| source < transaction && transaction.checked_add(1) == Some(frame))
            && *c == expected
    }
    pub(crate) fn validate_pack_a_state(&self) -> RuleResult<()> {
        // Check recursive ownership before source-specific early returns, in
        // every persisted container, including forged foreign source frames.
        for item in &self.stack { if let Some(f) = &item.frame { self.validate_pack_a_frame(f)?; }
            else if item.card.as_ref().is_some_and(|c| CARDS.contains(&c.definition.as_str())) { return Err("隐秘卡响应对象缺少原程序帧".into()); } }
        for e in &self.effects { match e {
            Effect::Frame { frame } => self.validate_pack_a_frame(frame)?,
            Effect::Declare { declaration } => self.validate_pack_a_declaration(declaration)?,
            _ => (),
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Frame { frame,.. } => self.validate_pack_a_frame(frame)?,
            ChoiceResolution::Declare { declaration,.. } => self.validate_pack_a_declaration(declaration)?,
            _ => (),
        } }
        let mut frames = BTreeSet::new(); let mut events = BTreeSet::new();
        let mut remember = |source:&SourceSnapshot,ops:&[Op]| -> RuleResult<()> {
            if let Some(Op::MysticWin { contest_id:Some(id) }) = ops.first() {
                if !events.insert((source.card.id.clone(),id.clone())) { return Err("同一观察者不能重复消费同次隐秘获胜".into()); }
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame {
            if !CARDS.contains(&f.source.card.definition.as_str()) || self.validated_cross_card_frame(f)? { continue; }
            self.validate_pack_a_frame(f)?; remember(&f.source,&f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            let transaction = matches!(f.ability_key.as_str(),"deploy"|"reveal"|EARTH);
            if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0
                || !matches!(f.guard,GuardState::Unchecked) || item.card.is_some() != transaction || item.reveal != (f.ability_key == "reveal")
                || item.target != f.targets.first().map(|t| t.id.clone())
                || item.deploy_region != if matches!(f.ability_key.as_str(),"deploy"|"reveal") { f.chosen_region } else { None }
                || item.card.as_ref().is_some_and(|c| !self.pack_a_transaction_matches(f,c)) {
                return Err("隐秘卡响应外壳与原执行帧不符".into());
            }
        } }
        for e in &self.effects { match e {
            Effect::Declare { declaration:d } if CARDS.contains(&d.source.card.definition.as_str()) && !self.validated_cross_card_declaration(d)? => {
                self.validate_pack_a_declaration(d)?; remember(&d.source,&d.ability.ops)?;
            },
            Effect::Frame { frame:f } if CARDS.contains(&f.source.card.definition.as_str()) && !self.validated_cross_card_frame(f)? => {
                self.validate_pack_a_frame(f)?;
                if !self.rumor_frame_continuation_valid(f) || !frames.insert(&f.frame_id) { return Err("隐秘卡执行续体只能来自真实暂停的传闻，不能绕过响应".into()); }
            },
            _ => (),
        } }
        let Some(p) = &self.pending else { return Ok(()); };
        match &p.resolution {
            ChoiceResolution::Declare { declaration:d,stage } if CARDS.contains(&d.source.card.definition.as_str()) && !self.validated_cross_card_declaration(d)? => {
                self.validate_pack_a_declaration(d)?; remember(&d.source,&d.ability.ops)?;
                if d.ability.key == SHADOW_REVEAL { return Err("阴影之子的现身触发不可改为可选".into()); }
                let (options,expected_stage) = if let Some(t) = d.ability.targets.first() { (self.binding_options(d.actor,&d.source,t),DeclareChoice::Target) }
                    else { (vec![ChoiceOption { id:"accept".into(),label:"发动触发能力".into(),card:None }],DeclareChoice::Accept) };
                let expected = Choice { id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",catalog::card(&d.source.card.definition).name,d.ability.label),
                    description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options,preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true) };
                if p.seat != d.actor || !self.mystic_instance_valid(&p.choice.id) || !same(stage,&expected_stage) || !same(&p.choice,&expected) { return Err("隐秘触发声明待选被改写".into()); }
            },
            ChoiceResolution::Frame { frame:f,choice } if CARDS.contains(&f.source.card.definition.as_str()) && !self.validated_cross_card_frame(f)? => {
                self.validate_pack_a_frame(f)?;
                if !frames.insert(&f.frame_id) { return Err("隐秘卡待选帧重复".into()); }
                match choice {
                    FrameChoice::MysticCommit { .. }|FrameChoice::MysticReward { .. } => (),
                    FrameChoice::Discard { seat,redraw:false } if f.ability_key == SHADOW_WIN && f.cursor == 2 && f.targets[0].id == player_id(*seat) => {
                        let options = self.players[*seat].hand.iter().map(|c| self.option(c,*seat,None,None)).collect::<Vec<_>>();
                        let expected = Choice { id:p.choice.id.clone(),kind:"discard".into(),title:"弃 1 张手牌".into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(*seat),options,preview_cards:vec![],min:Some(1),max:Some(1),amount:None,allow_decline:Some(false) };
                        if p.seat != *seat || !self.mystic_instance_valid(&p.choice.id) || !same(&p.choice,&expected) { return Err("阴影之子弃牌必须保留原目标玩家私有选择".into()); }
                    },
                    _ => return Err("隐秘卡不能移植其他待选程序".into()),
                }
            },
            _ => (),
        }
        Ok(())
    }
}
