//! JZ26 observes actual inspection/destruction, never a view read. Continuous
//! JC057 inspection is a free, anytime information command with the existing
//! permission; the ordinary command transaction and room receipts still apply.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const KEY:&str="batch3-jz26-trap";
const LABEL:&str="被敌方检视或消灭：可随机消灭原地区最低印刷费用的敌方角色";
#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum HiddenEvent { Inspection, Destruction }
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HiddenObservation {pub event_id:String,pub turn:u32,pub by:usize,pub event:HiddenEvent,pub blue_assets:Vec<Card>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectionReceipt {pub event_id:String,pub source:String,pub hidden:String,pub inspector:usize,pub region_instance:String}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn source_label(s:&SourceSnapshot)->&str {
    if s.card.definition=="JZ26" && s.card.face_down {"暗藏者"}else{crate::batch4_plans::label(&s.card)}
}
pub(crate) fn definition()->Definition {
    let mut d=with_abilities(vec![ability(KEY,LABEL,Timing::Fast,vec![],vec![],vec![Op::JZ26Trap{observation:None}],Some(Event::JZ26HiddenObserved))]);
    d.traits.stealth=true;d.traits.cannot_be_equipped=true;d
}
impl Game {
    fn blue_assets(&self,actor:usize)->Vec<Card>{self.players[actor].assets.iter().filter(|c|catalog::card(&c.definition).color=="蓝").take(2).cloned().collect()}
    fn jz26_capture(&self,by:usize,id:&str,event:HiddenEvent)->Option<(SourceSnapshot,HiddenObservation)> {
        let(r,c)=self.board(id)?;let actor=c.controller;
        if c.definition!="JZ26" || !c.face_down || by>=self.players.len() || self.players[actor].eliminated || !self.is_enemy(by,c){return None;}
        let blue_assets=self.blue_assets(actor);if blue_assets.len()!=2{return None;}
        Some((self.source_snapshot(c,Some(r)),HiddenObservation{event_id:String::new(),turn:self.turn,by,event,blue_assets}))
    }
    fn jz26_emit(&mut self,source:SourceSnapshot,mut o:HiddenObservation){
        o.event_id=self.id();let mut a=definition().abilities.remove(0);a.ops=vec![Op::JZ26Trap{observation:Some(o)}];
        self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});
    }
    pub(crate) fn observe_hidden_inspection(&mut self,by:usize,id:&str){
        if let Some((s,o))=self.jz26_capture(by,id,HiddenEvent::Inspection){self.jz26_emit(s,o);}
    }
    pub(crate) fn destroy_from_frame(&mut self,f:&ResolutionFrame,id:&str){
        // An automatic world frame's placeholder actor is not a player source.
        if catalog::card(&f.source.card.definition).kind=="region" {self.remove_dead(id,RemovalCause::Destroy);return;}
        self.destroy_as(f.actor,id);
    }
    pub(crate) fn destroy_as(&mut self,by:usize,id:&str){
        let observation=self.jz26_capture(by,id,HiddenEvent::Destruction);
        self.remove_dead(id,RemovalCause::Destroy);
        if self.in_play(id).is_none(){if let Some((s,o))=observation{self.jz26_emit(s,o);}}
    }
    pub(crate) fn destroy_wave_as(&mut self,by:usize,deaths:Vec<(String,RemovalCause,SourceSnapshot)>){
        let observations=deaths.iter().filter(|(_,cause,_)|matches!(cause,RemovalCause::Destroy))
            .filter_map(|(id,_,_)|self.jz26_capture(by,id,HiddenEvent::Destruction)).collect::<Vec<_>>();
        self.remove_death_batch(deaths);
        for(s,o)in observations{if self.in_play(&s.card.id).is_none(){self.jz26_emit(s,o);}}
    }
    fn jz26_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let [Op::JZ26Trap{observation:Some(o)}]=ops else{return Err("袖中毒物缺少实际检视或消灭观察".into());};let c=&s.card;
        let number=|id:&str|id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).filter(|n|*n>0 && *n<=self.sequence);
        if c.definition!="JZ26" || !c.face_down || c.owner>=self.players.len() || actor>=self.players.len() || c.controller!=actor || number(&c.id).is_none()
            || s.region.is_none_or(|r|r>=self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id|id.is_empty() || *id==c.id)
            || s.play_source.is_some() || s.attachment_host_instance.is_some() || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.observed_exposure.is_some()
            || o.turn!=self.turn || o.by>=self.players.len() || self.team(o.by)==self.team(actor) || number(&o.event_id).is_none() || number(&o.event_id)<=number(&c.id)
            || o.blue_assets.len()!=2 || o.blue_assets[0].id==o.blue_assets[1].id
            || self.batch3_physical_cards().any(|live|live.id==c.id && self.board(&c.id).is_none())
            || self.board(&c.id).is_some_and(|(_,live)|!live.face_down)
        {return Err("袖中毒物的原暗藏者、敌方玩家或事件身份无效".into());}
        self.mobility_immutable_identity(c)?;
        for a in &o.blue_assets {
            if a.face_down || a.controller!=actor || a.owner>=self.players.len() || number(&a.id).is_none() || number(&a.id)>=number(&o.event_id)
                || !catalog::catalog().cards.iter().any(|d|d.id==a.definition && d.color=="蓝")
                || self.batch3_physical_cards().any(|live|live.id==a.id && !self.players[actor].assets.iter().any(|live|live.id==a.id))
            {return Err("袖中毒物须保留个人资产区两个独立蓝图标，不以横置或盟友替代".into());}
            self.mobility_immutable_identity(a)?;
        }
        Ok(())
    }
    pub(crate) fn validate_jz26_declaration(&self,d:&Declaration)->RuleResult<()> {
        self.jz26_observation(d.actor,&d.source,&d.ability.ops)?;
        let mut a=definition().abilities.remove(0);a.ops=d.ability.ops.clone();if !same(&a,&d.ability){return Err("袖中毒物触发不能改写目标、事件或费用".into());}Ok(())
    }
    pub(crate) fn validate_jz26_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let ops=f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>();self.jz26_observation(f.actor,&f.source,&ops)?;
        if f.frame_id.is_empty() || f.ability_key!=KEY || f.steps.len()!=1 || f.steps[0].context!=f.actor || f.cursor>1
            || (f.cursor==0 && !matches!(f.guard,GuardState::Unchecked)) || (f.cursor==1 && !matches!(f.guard,GuardState::Accepted))
            || !f.targets.is_empty() || f.chosen_region.is_some() || !f.already_paid.is_empty(){return Err("袖中毒物必须保留无目标的完整响应帧".into());}Ok(())
    }
    pub(crate) fn validate_jz26_pending(&self,p:&Pending,d:&Declaration,stage:&DeclareChoice)->RuleResult<()> {
        self.validate_jz26_declaration(d)?;
        let options=vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}];
        if !matches!(stage,DeclareChoice::Accept) || p.seat!=d.actor || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty()
            || p.choice.title!=format!("暗藏者：{LABEL}") || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1)
            || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&options)
        {return Err("袖中毒物须保留不泄露正面的个人可选触发".into());}Ok(())
    }
    pub(crate) fn jz26_resolve(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_jz26_frame(f)?;if self.blue_assets(f.actor).len()!=2{return Ok(());}
        let Some(r)=self.original_region(&f.source)else{return Ok(());};
        let candidates=self.region_characters(r).filter(|c|!c.face_down && self.is_enemy(f.actor,c)).collect::<Vec<_>>();
        let Some(min)=candidates.iter().map(|c|catalog::card(&c.definition).cost).min()else{return Ok(());};
        let ids=candidates.into_iter().filter(|c|catalog::card(&c.definition).cost==min).map(|c|c.id.clone()).collect::<Vec<_>>();
        let id=ids[self.random_below(ids.len())].clone();self.destroy_as(f.actor,&id);Ok(())
    }
    pub(crate) fn jc057_can_inspect(&self,actor:usize,source:&str,hidden:&str)->bool {
        if self.status!="playing" || self.players.get(actor).is_none_or(|p|p.eliminated){return false;}
        self.board(source).zip(self.board(hidden)).is_some_and(|((r,s),(to,c))|r==to && self.region_live(r)
            && s.definition=="JC057" && s.controller==actor && !s.face_down && !s.exhausted && c.face_down)
    }
    pub(crate) fn jc057_inspection_known(&self,actor:usize,source:&str,hidden:&str,r:usize)->bool {
        self.jc057_can_inspect(actor,source,hidden) && self.inspection_receipts.iter().any(|p|p.inspector==actor && p.source==source && p.hidden==hidden && p.region_instance==self.regions[r].card.id)
    }
    pub(crate) fn inspect_hidden_command(&mut self,actor:usize,a:&Action)->RuleResult<()> {
        let source=a.card_id.as_deref().ok_or("检视缺少监控员来源")?;let hidden=a.target_id.as_deref().ok_or("检视缺少原暗藏者")?;
        if *a!=(Action{card_id:a.card_id.clone(),target_id:a.target_id.clone(),..Action::new("inspect-hidden")}) || !self.jc057_can_inspect(actor,source,hidden){return Err("检视仅限未横置监控员当前地区的原暗藏者，不消耗行动权".into());}
        let r=self.board(hidden).unwrap().0;
        let event_id=self.id();
        let receipt=InspectionReceipt{event_id,source:source.into(),hidden:hidden.into(),inspector:actor,region_instance:self.regions[r].card.id.clone()};
        if let Some(old)=self.inspection_receipts.iter_mut().find(|p|p.source==source && p.hidden==hidden && p.inspector==actor){*old=receipt;}else{self.inspection_receipts.push(receipt);}
        // A deliberate repeated inspection is another real inspection; reads of
        // the remembered view and command-id retries are not new inspections.
        self.observe_hidden_inspection(actor,hidden);Ok(())
    }
    pub(crate) fn jc057_inspection_actions(&self,actor:usize)->Vec<(Action,String)> {
        if self.status!="playing" || self.players.get(actor).is_none_or(|p|p.eliminated){return vec![];}
        self.regions.iter().filter(|r|!r.vacant).flat_map(|r|r.cards.iter().filter(|c|c.definition=="JC057" && c.controller==actor && !c.face_down && !c.exhausted)
            .flat_map(move |s|r.cards.iter().filter(|c|c.face_down).map(move |c|(Action{card_id:Some(s.id.clone()),target_id:Some(c.id.clone()),..Action::new("inspect-hidden")},format!("监控员：检视暗藏者 [{}]",c.id))))).collect()
    }
    pub(crate) fn prune_inspection_receipts(&mut self){
        let retained=self.inspection_receipts.iter().filter(|p|self.board(&p.source).zip(self.board(&p.hidden)).is_some_and(|((r,s),(to,c))|r==to && !s.face_down && s.definition=="JC057" && s.controller==p.inspector && c.face_down && self.regions[r].card.id==p.region_instance)).cloned().collect();self.inspection_receipts=retained;
    }
    pub(crate) fn validate_inspection_receipts(&self)->RuleResult<()> {
        let mut seen=BTreeSet::new();let number=|id:&str|id.strip_prefix('i').and_then(|s|s.parse::<u64>().ok()).is_some_and(|n|n>0 && n<=self.sequence);
        for p in &self.inspection_receipts {if p.inspector>=self.players.len() || !number(&p.event_id) || !number(&p.source) || !number(&p.hidden) || !number(&p.region_instance) || p.source==p.hidden || !seen.insert((p.source.as_str(),p.hidden.as_str(),p.inspector)) {return Err("私密检视收据的席位、原实例或重复记录无效".into());}}
        for p in &self.inspection_receipts {
            let ordinal=|id:&str|id.strip_prefix('i').and_then(|s|s.parse::<u64>().ok());
            if [p.source.as_str(),p.hidden.as_str(),p.region_instance.as_str()].iter().any(|id|ordinal(id)>=ordinal(&p.event_id))
                || self.batch3_physical_cards().any(|c|c.id==p.source && (c.definition!="JC057" || c.face_down))
                || self.batch3_physical_cards().any(|c|c.id==p.hidden && (!c.face_down || self.board(&c.id).is_none()))
            {return Err("私密检视收据须保留原监控员、暗藏者与后发事件".into());}
        }
        Ok(())
    }
}
