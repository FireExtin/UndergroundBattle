//! LC16: one normal preparation receipt, private subtype search, shuffle, then
//! influence on the region chosen before responses. No second search engine.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const KEY: &str = "batch4-lc16-prepare-search";
pub(crate) const KIND: &str = "lc16-prepare-search";
const TITLE: &str = "江月：检索零或一张法术牌或阴牌置墓，然后洗牌";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let a = ability(KEY, "准备：检索法术或阴牌置墓并洗牌，原目标地区放本方势力1", Timing::Fast,
        vec![], vec![target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere)],
        vec![Op::LC16PrepareSearch { event_id: None }], Some(Event::Prepare));
    let mut d = with_abilities(vec![a]); d.traits.wound = 2; d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.remove(0);
    if a.key != KEY { return None; }
    if let [Op::LC16PrepareSearch { event_id }] = a.ops.as_slice() { expected.ops = vec![Op::LC16PrepareSearch { event_id: event_id.clone() }]; }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind) },
        "reveal" => Action { card_id:a.card_id.clone(),..Action::new("reveal") },
        _ => return Err("江月只有正常派遣、秘密派遣及现身，准备能力须从实际队列发动".into()),
    };
    if *a != expected { return Err("江月派遣不能夹带检索、目标或额外费用载荷".into()); } Ok(())
}
impl Game {
    fn lc16_trigger_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()> {
        let c=&s.card;
        if c.definition!="LC16" || c.id.is_empty() || c.face_down || actor>=self.players.len() || c.owner>=self.players.len() || c.controller!=actor
            || s.play_source.is_some() || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.is_none_or(|r|r>=self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id|id.is_empty() || *id==c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,live)|live.card.id==*id && other!=r))
            || self.batch3_physical_cards().any(|live|live.id==c.id) && self.in_play(&c.id).is_none() {
            return Err("江月触发须保留原角色、操控者及地区实体".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_lc16_declaration(&self,d:&Declaration)->RuleResult<()> {
        self.lc16_trigger_source(d.actor,&d.source)?;
        if d.ability.event==Some(Event::Prepare) { return crate::jc035::validate_ability("LC16",&d.ability); }
        if crate::batch4::reward(&d.source,&d.ability.key).is_none_or(|expected|!same(&expected,&d.ability)) {
            return Err("江月只能保留原准备程序或既有完整获授奖励".into());
        }
        Ok(())
    }
    pub(crate) fn lc16_bound_region(&self, f:&ResolutionFrame) -> bool {
        let [t] = f.targets.as_slice() else { return false; };
        let Some(r) = t.id.strip_prefix("region:").and_then(|n|n.parse::<usize>().ok()).filter(|r|*r<self.regions.len()) else {return false;};
        t.id == format!("region:{r}") && same(&t.spec,&definition().abilities[0].targets[0])
            && f.chosen_region == Some(r) && t.public.instance_id == t.id && t.public.kind == "region" && t.public.region == Some(r)
            && t.public.owner.is_none() && t.public.controller.is_none() && t.public.valid && t.public.status == "valid" && t.public.invalid_reason.is_none()
            && t.region_instance.as_ref().is_some_and(|id|!id.is_empty() && *id != f.source.card.id
                && !self.regions.iter().enumerate().any(|(other,live)|live.card.id == *id && other!=r))
    }
    pub(crate) fn validate_lc16_frame(&self, f:&ResolutionFrame) -> RuleResult<()> {
        if crate::jc035::preparation_frame(f) { return self.validate_jc035_frame(f); }
        if let Some(a)=crate::batch4::reward(&f.source,&f.ability_key) {
            self.validate_lc16_declaration(&Declaration{actor:f.actor,source:f.source.clone(),ability:a.clone()})?;
            if f.frame_id.is_empty() || f.frame_id==f.source.card.id || f.cursor>a.ops.len()
                || f.cursor==0 && !matches!(f.guard,GuardState::Unchecked) || f.cursor>0 && !matches!(f.guard,GuardState::Accepted)
                || f.chosen_region.is_some() || !f.already_paid.is_empty() || !f.targets.is_empty()
                || f.steps.len()!=a.ops.len() || f.steps.iter().zip(&a.ops).any(|(s,op)|s.context!=f.actor || !same(&s.op,op)) {
                return Err("江月获授奖励须保留原完整操作及地区".into());
            }
            return Ok(());
        }
        let c=&f.source.card;
        if c.definition!="LC16" || c.id.is_empty() || c.owner>=self.players.len() || f.actor>=self.players.len() || c.controller!=f.actor
            || f.source.death_echo.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
            || f.source.attachment_host_instance.is_some() || f.source.region.is_none_or(|r|r>=self.regions.len())
            || f.source.source_region_instance.as_ref().is_none_or(|id|id.is_empty()) || f.frame_id.is_empty() || f.frame_id==c.id
            || !matches!(f.ability_key.as_str(),"deploy"|"reveal") || f.cursor!=0 || !matches!(f.guard,GuardState::Unchecked)
            || !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region!=f.source.region
            || f.ability_key=="deploy" && (c.face_down || !f.source.valid_character_play())
            || f.ability_key=="reveal" && (!c.face_down || f.source.play_source.is_some())
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()<=6 && ids.iter().all(|id|!id.is_empty() && *id!=c.id) && ids.iter().collect::<BTreeSet<_>>().len()==ids.len()) {
            return Err("江月须保留正常角色打出、原地区、操控者及印刷费用".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn lc16_candidates(&self, actor:usize) -> Vec<ChoiceOption> {
        self.players[actor].deck.iter().filter(|c|catalog::catalog().cards.iter().find(|d|d.id==c.definition)
            .is_some_and(|d|d.subtypes.iter().any(|s|s=="法术"||s=="阴")))
            .map(|c|self.option(c,actor,None,None)).collect()
    }
    pub(crate) fn validate_lc16_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if f.source.card.definition!="LC16" || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted)
            || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty()
            || p.choice.kind!=KIND || p.choice.title!=TITLE || p.choice.min!=Some(0) || p.choice.max!=Some(1)
            || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options,&self.lc16_candidates(f.actor)) {
            return Err("江月检索只属于原准备玩家及其当前牌库".into());
        }
        Ok(())
    }
    pub(crate) fn lc16_search_start(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_jc035_frame(f)?;
        let options=self.lc16_candidates(f.actor);
        // Even zero hits uses an actor-private prompt: other seats do not learn
        // library composition from whether a search prompt was skipped.
        self.choice(f.actor,KIND,TITLE.into(),options,0,1,None,
            ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::LC16Search});
        self.pending.as_mut().unwrap().choice.allow_decline=Some(false); Ok(())
    }
    pub(crate) fn lc16_search_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if chooser!=f.actor || f.source.card.definition!="LC16" || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted)
            || selected.len()>1 || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {
            return Err("江月只能继续原准备检索一次".into());
        }
        if let Some(id)=selected.first() {
            if !self.lc16_candidates(f.actor).iter().any(|o|o.id==*id) {return Err("江月须选择自己牌库中的法术牌或阴牌".into());}
            let i=self.players[f.actor].deck.iter().position(|c|c.id==*id).unwrap();
            let card=self.players[f.actor].deck.remove(i);let card=self.reset_zone_card(card);
            self.note(format!("{} 将检索的 {} 置于墓地",self.players[f.actor].name,catalog::card(&card.definition).name));
            self.commit_owner_grave(card.clone(),None);
            self.observe_black_deck_to_graveyard(f.actor,&[card]);
        }
        self.shuffle_player(f.actor);
        if let Some(t)=f.targets.first().filter(|t|self.valid_bound_target(f.actor,&f.source,t)) {
            let r=t.id.strip_prefix("region:").and_then(|s|s.parse::<usize>().ok()).ok_or("江月原地区无效")?;
            self.place_influence(f.actor,r,1);
        }
        // One fixed operation has finished. Do not insert a second frame or
        // reopen the already-resolved preparation receipt after the private choice.
        Ok(())
    }
    pub(crate) fn validate_lc16_state(&self)->RuleResult<()> {
        if let Some(p)=&self.pending {
            if matches!(&p.resolution,ChoiceResolution::Frame{choice:FrameChoice::LC16Search,..}) || p.choice.kind==KIND {
                let ChoiceResolution::Frame{frame,choice:FrameChoice::LC16Search}=&p.resolution else {return Err("江月检索缺少固定续体".into());};
                self.validate_lc16_pending(p,frame)?;
            }
        }
        Ok(())
    }
}
