//! A named character is observed on actual face-up entry. The wanted effect has
//! no target: it retains both original UIDs through the existing response stack.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
const NAME:&str="batch3-xq26-name";const WANTED:&str="batch3-xq26-wanted";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedEntryObservation{pub event_id:String,pub turn:u32,pub entered:SourceSnapshot}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition{
    let mut d=attachment_definition(target(Zone::Society,EntityKind::Any,Relation::ControlledByActor,Range::Anywhere));d.abilities[0].key="batch3-xq26-attach".into();
    d.abilities.push(ability(NAME,"进场：说出一个角色牌名称",Timing::Fast,vec![],vec![],vec![Op::XQ26Name],Some(Event::Enter)));
    d.abilities.push(ability(WANTED,"强制：牺牲本通缉令，然后消灭该名称的进场角色",Timing::Fast,vec![],vec![],vec![Op::XQ26Wanted{observation:None}],Some(Event::XQ26NamedCharacterEntered)));d
}
impl Game{
    fn xq26_names(&self)->Vec<ChoiceOption>{catalog::catalog().cards.iter().filter(|d|d.kind=="character").map(|d|d.name.clone()).collect::<BTreeSet<_>>().into_iter().map(|name|ChoiceOption{id:name.clone(),label:name,card:None}).collect()}
    pub(crate) fn xq26_observe_entry(&mut self,id:&str){
        let Some((r,c))=self.board(id).filter(|(_,c)|!c.face_down && catalog::card(&c.definition).kind=="character")else{return;};
        let name=&catalog::card(&c.definition).name;
        let sources=self.attachments.iter().filter(|a|a.card.definition=="XQ26" && !a.card.face_down && a.card.named_character.as_ref()==Some(name) && self.attachment_host_valid(a)).map(|a|self.source_snapshot(&a.card,None)).collect::<Vec<_>>();
        if sources.is_empty(){return;}
        let entered=self.source_snapshot(c,Some(r));let observation=NamedEntryObservation{event_id:self.id(),turn:self.turn,entered};
        for source in sources{let mut a=definition().abilities.remove(2);a.ops=vec![Op::XQ26Wanted{observation:Some(observation.clone())}];self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});}
    }
    pub(crate) fn xq26_validate_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()> {
        if s.card.definition!="XQ26" || s.card.id.is_empty() || s.card.face_down || actor>=self.players.len() || s.card.controller!=actor || s.card.owner>=self.players.len() || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.as_deref()!=Some(format!("society:p{actor}").as_str()) || s.play_source.is_some() || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.observed_exposure.is_some()
            || self.batch3_physical_cards().any(|c|c.id==s.card.id && !self.attachments.iter().any(|a|a.card.id==c.id && a.card.definition=="XQ26" && a.card.owner==s.card.owner)) {return Err("通缉令须保留原秘社宿主和公开来源".into());}self.mobility_immutable_identity(&s.card)
    }
    pub(crate) fn xq26_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        self.xq26_validate_source(actor,s)?;
        let [Op::XQ26Wanted{observation:Some(o)}]=ops else{return Err("通缉令缺少唯一实际进场观察".into());};let e=&o.entered;let c=&e.card;
        if c.face_down || c.id.is_empty() || c.id==s.card.id || c.owner>=self.players.len() || c.controller>=self.players.len() || !catalog::catalog().cards.iter().any(|d|d.id==c.definition && d.kind=="character" && s.card.named_character.as_deref()==Some(d.name.as_str()))
            || e.region.is_none_or(|r|r>=self.regions.len()) || e.play_source.is_some() || e.observed_death.is_some() || e.observed_mobility_grant.is_some() || e.observed_exposure.is_some()
            || o.turn!=self.turn || o.event_id==c.id || o.event_id==s.card.id || o.event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n==0 || n>self.sequence) {return Err("通缉令观察的原名称、角色或进场身份不符".into());}self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_xq26_declaration(&self,d:&Declaration)->RuleResult<()> {
        self.xq26_validate_source(d.actor,&d.source)?;let mut expected=definition().abilities.into_iter().find(|a|a.key==d.ability.key).ok_or("非准入通缉令声明")?;
        if d.ability.key==WANTED{self.xq26_observation(d.actor,&d.source,&d.ability.ops)?;expected.ops=d.ability.ops.clone();}
        else if d.ability.key!=NAME || d.source.card.named_character.is_some(){return Err("通缉令进场命名上下文不符".into());}
        if !same(&expected,&d.ability){return Err("通缉令完整声明被改写".into());}Ok(())
    }
    pub(crate) fn xq26_name_start(&mut self,f:ResolutionFrame)->RuleResult<()> {
        if !self.attachments.iter().any(|a|a.card.id==f.source.card.id && a.card.named_character.is_none() && !a.card.face_down){return Ok(());}
        let options=self.xq26_names();self.choice(f.actor,"batch3-xq26-name","通缉令：说出一个当前牌池角色牌名称".into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::XQ26Name});Ok(())
    }
    pub(crate) fn validate_xq26_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_batch3_grave_frame(f)?;
        if f.ability_key!=NAME || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || !self.attachments.iter().any(|a|a.card.id==f.source.card.id && a.card.named_character.is_none() && !a.card.face_down) || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty() || p.choice.kind!="batch3-xq26-name" || p.choice.title!="通缉令：说出一个当前牌池角色牌名称" || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&self.xq26_names()){return Err("通缉令命名待选与原来源或当前角色名集合不符".into());}Ok(())
    }
    pub(crate) fn xq26_name_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_batch3_grave_frame(f)?;
        if chooser!=f.actor || f.ability_key!=NAME || f.cursor!=1 || selected.len()!=1 || !self.xq26_names().iter().any(|o|o.id==selected[0]) || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {return Err("通缉令只能命名一个准入角色牌名称".into());}
        if let Some(c)=self.attachments.iter_mut().find(|a|a.card.id==f.source.card.id).map(|a|&mut a.card){c.named_character=Some(selected[0].clone());}Ok(())
    }
    pub(crate) fn xq26_wanted_resolve(&mut self,f:&ResolutionFrame,o:&NamedEntryObservation){self.remove_dead(&f.source.card.id,RemovalCause::Sacrifice);self.remove_dead(&o.entered.card.id,RemovalCause::Destroy);}
    pub(crate) fn validate_named_characters(&self)->RuleResult<()> {
        for c in self.batch3_physical_cards().filter(|c|c.named_character.is_some()) {if c.definition!="XQ26" || c.face_down || !self.attachments.iter().any(|a|a.card==*c && self.attachment_host_valid(a)) || !self.xq26_names().iter().any(|o|Some(&o.id)==c.named_character.as_ref()){return Err("通缉令名称只能留在原场内附属实例".into());}}Ok(())
    }
    pub(crate) fn validate_xq26_trigger_pending(&self,p:&Pending,d:&Declaration,stage:&DeclareChoice)->RuleResult<()> {
        self.validate_xq26_declaration(d)?;
        if d.ability.key!=NAME || !matches!(stage,DeclareChoice::Accept) || p.seat!=d.actor || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty() || p.choice.title!=format!("{}：{}",catalog::card("XQ26").name,d.ability.label) || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}]){return Err("通缉令进场声明须保留可选命名阶段".into());}Ok(())
    }
}
