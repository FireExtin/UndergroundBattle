//! JZ36 is a finite two-mode program with a resumable, unpaid cost draft.
//! X is the committed sacrifice-set size, never a client-supplied effect value.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-jz36-blood-sacrifice";
const CHOICE: &str = "extra-sacrifice";
const TITLE: &str = "选择额外牺牲的角色（数量决定X）";
fn same<T: Serialize>(a:&T,b:&T)->bool { serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap() }
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BloodDraft {
    pub actor:usize,
    pub turn:u32,
    pub card:Card,
    pub action:Action,
    pub targets:Vec<BoundTarget>,
}
pub(crate) fn definition()->Definition {
    let mut carrier=target(Zone::Board,EntityKind::Any,Relation::Any,Range::Anywhere);
    carrier.predicate=Some(TargetPredicate::JZ36CharacterOrAttachment);
    let mut a=ability(KEY,"牺牲X个角色，放置X时间并检查目标地区毁灭",Timing::Standard,
        vec![Cost::JZ36SacrificeBatch],vec![],vec![],None);
    a.play_only=true;
    a.modes=vec![Mode{key:"carrier".into(),label:"目标角色或附属".into(),targets:vec![carrier],ops:vec![Op::JZ36TimeAndDestroyTargetRegion]},
        Mode{key:"region".into(),label:"目标地区".into(),targets:vec![target(Zone::Region,EntityKind::Any,Relation::Any,Range::Anywhere)],ops:vec![Op::JZ36TimeAndDestroyTargetRegion]}];
    with_abilities(vec![a])
}
pub(crate) fn canonical(a:&AbilitySpec)->Option<AbilitySpec> {
    let base=definition().abilities.remove(0);
    if same(a,&base) { return Some(base); }
    for m in &base.modes {
        let mut flat=base.clone();flat.targets=m.targets.clone();flat.ops=m.ops.clone();flat.modes.clear();
        if same(a,&flat) { return Some(flat); }
    }
    None
}
pub(crate) fn validate_action(a:&Action)->RuleResult<()> {
    if a.kind!="play" || a.ability_id.as_deref().is_some_and(|k|k!=KEY) { return Err("血祭只有印刷标准打出程序".into()); }
    let mut expected=Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),option:a.option.clone(),cost_selected:a.cost_selected.clone(),..Action::new("play")};
    match a.option.as_deref() {Some("region")=>expected.region=a.region,Some("carrier")=>expected.target_id=a.target_id.clone(),_=>return Err("血祭须先选择角色/附属或地区模式".into())}
    if *a!=expected { return Err("血祭不能追加数量、其他目标或选择载荷".into()); }
    Ok(())
}
pub(crate) fn validate_choice_action(p:&Pending,a:&Action)->RuleResult<()> {
    if matches!(p.resolution,ChoiceResolution::JZ36Payment{..})
        && *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {
        return Err("额外牺牲只接受当前选择，不接受重选目标或费用载荷".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jz36_target(&self,source:&SourceSnapshot,c:&Card)->bool {
        source.card.definition=="JZ36" && ((!c.face_down && catalog::card(&c.definition).kind=="character")
            || self.attachments.iter().any(|a|a.card.id==c.id && self.attachment_host_valid(a)
                && (!c.face_down || crate::batch4_plans::secret(c)) && catalog::card(&c.definition).kind=="attachment"))
    }
    fn jz36_cost_options(&self,actor:usize)->Vec<ChoiceOption> {
        self.in_play_cards().into_iter().filter(|(_,c)|c.controller==actor && !c.face_down && catalog::card(&c.definition).kind=="character")
            .map(|(r,c)|self.option(c,actor,r,Some("character"))).collect()
    }
    fn validate_jz36_draft(&self,d:&BloodDraft)->RuleResult<()> {
        validate_action(&d.action)?;
        if d.actor>=self.players.len() || d.card.definition!="JZ36" || d.card.controller!=d.actor || d.card.owner>=self.players.len()
            || d.card.face_down || d.turn!=self.turn || self.status!="playing" || self.players[d.actor].eliminated
            || self.window!=Some(Window::Action(self.team(d.actor))) || self.priority_team!=self.team(d.actor) || !self.stack.is_empty()
            || d.action.card_id.as_deref()!=Some(d.card.id.as_str()) || d.action.cost_selected.is_some()
            || self.players[d.actor].hand.iter().find(|c|c.id==d.card.id).is_none_or(|c|c!=&d.card)
            || self.resources(d.actor)<self.effective_cost(d.actor,&d.card) || !self.loyalty(d.actor,"JZ36") {
            return Err("血祭未付费声明须保留原手牌、玩家、回合和行动步骤".into());
        }
        self.mobility_immutable_identity(&d.card)?;
        let source=self.source_snapshot(&d.card,None);
        let a=self.ability_for_action("JZ36",&d.action)?;
        if !same(&self.bind_action(d.actor,&source,&a,&d.action)?,&d.targets) { return Err("血祭未付费声明的原目标已改变".into()); }
        Ok(())
    }
    pub(crate) fn jz36_begin_cost(&mut self,actor:usize,c:&Card,a:&Action,targets:Vec<BoundTarget>)->RuleResult<()> {
        let draft=BloodDraft{actor,turn:self.turn,card:c.clone(),action:a.clone(),targets};self.validate_jz36_draft(&draft)?;
        let options=self.jz36_cost_options(actor);let max=options.len();
        self.choice(actor,CHOICE,TITLE.into(),options,0,max,None,ChoiceResolution::JZ36Payment{draft:Box::new(draft)});
        Ok(())
    }
    pub(crate) fn jz36_finish_cost(&mut self,draft:BloodDraft,selected:Vec<String>)->RuleResult<()> {
        self.validate_jz36_draft(&draft)?;
        let mut a=draft.action;a.cost_selected=Some(selected);
        // choose() has cleared only this pending choice. Normal play now checks
        // and pays once; the outer command owns versioning and persistence.
        self.apply_inner(draft.actor,a)
    }
    pub(crate) fn jz36_pay(&mut self,actor:usize,a:&Action)->RuleResult<PaidCost> {
        let ids=a.cost_selected.as_ref().ok_or("请选择额外牺牲数量与角色")?;
        if ids.iter().collect::<BTreeSet<_>>().len()!=ids.len() { return Err("血祭不能重复支付同一实体".into()); }
        let mut victims=vec![];
        for id in ids {
            let (r,c)=self.in_play(id).filter(|(_,c)|c.controller==actor && !c.face_down && catalog::card(&c.definition).kind=="character")
                .ok_or("血祭费用须为本人当前操控的明置角色")?;
            let mut proof=self.source_snapshot(c,r);proof.source_region_instance=r.map(|r|self.regions[r].card.id.clone());victims.push(proof);
        }
        let deaths=victims.iter().map(|s|(s.card.id.clone(),RemovalCause::Sacrifice,self.source_snapshot(&s.card,s.region))).collect();
        self.destroy_wave_as(actor,deaths);
        Ok(PaidCost::JZ36Sacrificed(victims))
    }
    fn jz36_paid<'a>(&self,f:&'a ResolutionFrame)->RuleResult<&'a Vec<SourceSnapshot>> {
        let [PaidCost::Assets(assets),PaidCost::JZ36Sacrificed(victims)]=f.already_paid.as_slice() else {return Err("血祭须保留印刷费和完整批量牺牲收据".into());};
        if assets.len()>1 || assets.iter().any(|id|id.is_empty() || *id==f.source.card.id) || assets.iter().collect::<BTreeSet<_>>().len()!=assets.len()
            || victims.len()>u32::MAX as usize { return Err("血祭费用数量或资产收据无效".into()); }
        let mut ids=BTreeSet::new();
        for s in victims {
            let c=&s.card;let valid=catalog::catalog().cards.iter().any(|d|d.id==c.definition && d.kind=="character" && catalog::runtime_implemented(d));
            if !valid || c.id.is_empty() || !ids.insert(&c.id) || c.id==f.source.card.id || c.id==f.frame_id || c.face_down
                || c.controller!=f.actor || c.owner>=self.players.len() || s.play_source.is_some() || s.death_echo.is_some()
                || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
                || s.region.is_none_or(|r|r>=self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id|id.is_empty())
                || s.attachment_host_instance.is_some() && c.definition!="BQ026"
                || s.attachment_host_instance.as_ref().is_some_and(|id|id.is_empty() || *id==c.id)
                || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,live)|live.card.id==*id && other!=r))
                || self.batch3_physical_cards().any(|live|live.id==c.id) { return Err("血祭批量牺牲收据身份或原地区无效".into()); }
            self.mobility_immutable_identity(c)?;
        }
        Ok(victims)
    }
    pub(crate) fn validate_jz36_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let s=&f.source;let c=&s.card;
        if c.definition!="JZ36" || c.id.is_empty() || c.face_down || c.owner>=self.players.len() || f.actor>=self.players.len() || c.controller!=f.actor
            || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some() || s.play_source.is_some()
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.ability_key!=KEY || f.frame_id.is_empty() || f.frame_id==c.id || f.cursor!=0 || !matches!(f.guard,GuardState::Unchecked)
            || !matches!(f.steps.as_slice(),[Step{context,op:Op::JZ36TimeAndDestroyTargetRegion}] if *context==f.actor)
            || f.targets.len()!=1 || self.batch3_physical_cards().any(|live|live.id==c.id) { return Err("血祭须保留原打出来源和未执行的有限程序".into()); }
        self.mobility_immutable_identity(c)?;self.jz36_paid(f)?;
        let t=&f.targets[0];let base=definition();let a=&base.abilities[0];
        let valid_player=|id:&str|id.strip_prefix('p').and_then(|n|n.parse::<usize>().ok()).is_some_and(|n|n<self.players.len() && player_id(n)==id);
        if !a.modes.iter().any(|m|same(&m.targets[0],&t.spec)) || t.id.is_empty() || t.public.instance_id!=t.id || !t.public.valid
            || t.public.status!="valid" || t.public.invalid_reason.is_some() {return Err("血祭目标模式或声明摘要无效".into());}
        if t.spec.zone==Zone::Region {
            let r=t.id.strip_prefix("region:").and_then(|n|n.parse::<usize>().ok()).filter(|r|*r<self.regions.len()).ok_or("血祭地区目标无效")?;
            if t.id!=format!("region:{r}") || f.chosen_region!=Some(r) || t.public.kind!="region" || t.public.region!=Some(r)
                || t.public.owner.is_some() || t.public.controller.is_some() || t.region_instance.as_ref().is_none_or(|id|id.is_empty()) {
                return Err("血祭须保留已声明地区的实体UID".into());
            }
        } else if f.chosen_region.is_some() || t.region_instance.is_some() || !matches!(t.public.kind.as_str(),"character" | "attachment")
            || t.public.owner.as_deref().is_none_or(|id|!valid_player(id)) || t.public.controller.as_deref().is_none_or(|id|!valid_player(id))
            || t.public.region.is_some_and(|r|r>=self.regions.len()) || t.public.kind=="character" && t.public.region.is_none() { return Err("血祭角色/附属目标摘要无效".into()); }
        Ok(())
    }
    pub(crate) fn jz36_resolve(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        let n=self.jz36_paid(f)?.len() as u32;let t=&f.targets[0];
        let region=if t.spec.zone==Zone::Region {
            let r=t.public.region.ok_or("血祭缺少地区")?;
            self.regions[r].card.time_markers=self.regions[r].card.time_markers.saturating_add(n);Some(r)
        } else {
            let r=self.in_play(&t.id).and_then(|(r,_)|r);
            if let Some(c)=self.board_mut(&t.id){c.time_markers=c.time_markers.saturating_add(n);}r
        };
        if let Some(r)=region.filter(|r|self.region_live(*r)) {if self.physical_region_time_markers(r)>=4 {self.regions[r].destroyed=true;}}
        Ok(())
    }
    pub(crate) fn validate_jz36_state(&self)->RuleResult<()> {
        if let Some(p)=&self.pending {if let ChoiceResolution::JZ36Payment{draft}=&p.resolution {
            self.validate_jz36_draft(draft)?;let options=self.jz36_cost_options(draft.actor);
            if p.seat!=draft.actor || p.choice.id.is_empty() || p.choice.id==draft.card.id || p.choice.kind!=CHOICE || p.choice.title!=TITLE
                || p.choice.player_id!=player_id(draft.actor) || p.choice.min!=Some(0) || p.choice.max!=Some(options.len())
                || p.choice.amount.is_some() || p.choice.allow_decline!=Some(true) || !p.choice.preview_cards.is_empty()
                || p.choice.description!="选择符合数量限制的选项，然后确认。" || !same(&p.choice.options,&options) {
                return Err("血祭未付费选择的数量、玩家、选项或续体被改写".into());
            }
        }else if matches!(&p.resolution,ChoiceResolution::Frame{frame,..} if frame.source.card.definition=="JZ36") {
            return Err("血祭结算后没有额外选择续体".into());
        }}
        let mut frames=BTreeSet::new();
        for item in &self.stack {if let Some(f)=&item.frame {if f.source.card.definition=="JZ36" && !self.validated_cross_card_frame(f)? {
            self.validate_jz36_frame(f)?;
            if !frames.insert(&f.frame_id) || item.id!=f.frame_id || item.controller!=f.actor || item.target!=Some(f.targets[0].id.clone())
                || item.reveal || item.deploy_region.is_some() || item.card.as_ref().is_none_or(|c|c.definition!="JZ36" || c.face_down || c.controller!=f.actor || c.owner!=f.source.card.owner) {
                return Err("血祭堆叠外壳、实体和程序不一致".into());
            }
        }}}
        for e in &self.effects {if matches!(e,Effect::Frame{frame} if frame.source.card.definition=="JZ36") {return Err("血祭原子程序不能停留在未绑定队列".into());}}
        Ok(())
    }
}
