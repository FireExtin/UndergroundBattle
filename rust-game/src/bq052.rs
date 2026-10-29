//! Closed original BQ052. Generation uses the existing preparation observation;
//! the three sacrifices are an atomic cost, never a post-response choice.
use crate::{engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ052";
pub(crate) const PREPARE: &str = "bq052-prepare-tentacle";
pub(crate) const PLACE: &str = "bq052-sacrifice-three-place-core";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool { serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap() }
pub(crate) fn definition()->Definition {
    let host=target(Zone::Society,EntityKind::Any,Relation::ControlledByActor,Range::Anywhere);
    let mut attach=ability("attach","结附本方秘社",Timing::Standard,vec![],vec![host.clone()],vec![],None);attach.play_only=true;
    let prepare=ability(PREPARE,"准备：随机地区生成一个触手",Timing::Fast,vec![],vec![],vec![Op::BQ052Prepare{event_id:None}],Some(Event::Prepare));
    let mut place=ability(PLACE,"牺牲三个本方触手：核心区核心随机进场",Timing::Standard,vec![Cost::BQ052SacrificeThree],vec![],vec![Op::BQ052PlaceCore],None);place.activation_only=true;
    let mut d=with_abilities(vec![attach,prepare,place]);
    d.attachment=Some(AttachmentSpec{controls_host:false,host_subtype_change:SubtypeChange::None,host,host_icons:Icons::default(),host_temporary_icons:None,host_barrier:false,host_defense_bonus:0,host_leaves:HostLeaveDestination::OwnerGraveyard});
    d.modifiers.push(StaticModifier::BQ052TentacleInfluence);d
}
pub(crate) fn related(a:&AbilitySpec)->bool {a.key.starts_with("bq052-")||a.costs.iter().any(|c|matches!(c,Cost::BQ052SacrificeThree))||a.ops.iter().any(|o|matches!(o,Op::BQ052Prepare{..}|Op::BQ052PlaceCore))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {
    if id!=ID{return Err("BQ052能力不能移植".into());}
    if a.key==PREPARE{return crate::jc035::validate_ability(id,a);}
    if !definition().abilities.iter().any(|v|same(a,v)){return Err("BQ052须保留原费用、宿主和完整程序".into());}Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()> {
    if (id==ID||d.abilities.iter().any(related)||d.modifiers.iter().any(|m|matches!(m,StaticModifier::BQ052TentacleInfluence)))&&(id!=ID||!same(d,&definition())){return Err("BQ052完整定义不能删改或移植".into());}Ok(())
}
impl Game {
    pub(crate) fn bq052_influence(&self,c:&Card)->u32 {
        if !crate::tentacle::character(c)||!self.board(&c.id).is_some_and(|(_,live)|live==c){return 0;}
        let actor=c.controller;
        if self.players.get(actor).is_none_or(|p|p.eliminated){return 0;}
        let active=self.players[actor].society_zone.secrecy>0||self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).any(|core|!core.face_down&&core.owner==actor&&core.controller==actor&&self.selected_core_card(core));
        if !active{return 0;}
        self.attachments.iter().filter(|a|a.card.definition==ID&&!a.card.face_down&&a.card.controller==actor&&a.host_id==format!("society:p{actor}")&&self.attachment_host_valid(a)).count() as u32
    }
    pub(crate) fn bq052_generate(&mut self,actor:usize)->RuleResult<()> {
        if self.players.get(actor).is_none_or(|p|p.eliminated){return Ok(());}
        let regions=(0..self.regions.len()).filter(|r|self.region_live(*r)).collect::<Vec<_>>();
        if regions.is_empty(){return Ok(());}
        let region=regions[self.random_below(regions.len())];
        let c=self.make_card(crate::tentacle::ID,actor);let id=c.id.clone();
        self.regions[region].cards.push(c);
        self.enter_triggers(actor,crate::tentacle::ID,&id,false);Ok(())
    }
    pub(crate) fn bq052_trios(&self,actor:usize)->Vec<Vec<String>> {
        let ids=self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).filter(|c|c.controller==actor&&crate::tentacle::character(c)).map(|c|c.id.clone()).collect::<Vec<_>>();
        let mut out=vec![];for i in 0..ids.len(){for j in i+1..ids.len(){for k in j+1..ids.len(){out.push(vec![ids[i].clone(),ids[j].clone(),ids[k].clone()]);}}}out
    }
    pub(crate) fn bq052_pay(&mut self,actor:usize,source:&SourceSnapshot,spec:&AbilitySpec,a:&Action)->RuleResult<PaidCost> {
        validate_ability(&source.card.definition,spec)?;
        if spec.key!=PLACE||!self.attachments.iter().any(|v|v.card.id==source.card.id&&v.card.controller==actor&&v.host_id==format!("society:p{actor}")&&self.attachment_host_valid(v))
            ||*a!=(Action{card_id:a.card_id.clone(),ability_id:Some(PLACE.into()),cost_selected:a.cost_selected.clone(),..Action::new("activate")}){return Err("BQ052牺牲必须来自原本方秘社附属行动".into());}
        let ids=a.cost_selected.as_ref().filter(|ids|ids.len()==3&&ids.iter().collect::<BTreeSet<_>>().len()==3).ok_or("须牺牲三个不同的本方触手")?;
        let mut victims=vec![];
        for id in ids {let (r,c)=self.board(id).filter(|(_,c)|c.controller==actor&&crate::tentacle::character(c)).ok_or("费用须是三个实际本方明置触手")?;let mut proof=self.source_snapshot(c,Some(r));proof.source_region_instance=Some(self.regions[r].card.id.clone());victims.push(proof);}
        let wave=victims.iter().map(|s|(s.card.id.clone(),RemovalCause::Sacrifice,s.clone())).collect();
        self.destroy_wave_as(actor,wave);Ok(PaidCost::BQ052Sacrificed(victims))
    }
    fn bq052_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let relevant=f.ability_key==PLACE||f.steps.iter().any(|s|matches!(s.op,Op::BQ052PlaceCore))||f.already_paid.iter().any(|p|matches!(p,PaidCost::BQ052Sacrificed(_)));
        if !relevant{return Ok(());}
        let s=&f.source;let c=&s.card;
        if c.definition!=ID||f.actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=f.actor||c.face_down||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||s.attachment_host_instance.as_deref()!=Some(format!("society:p{}",f.actor).as_str())
            ||!self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(c,f)|c<f)||f.ability_key!=PLACE||!f.targets.is_empty()||f.chosen_region.is_some()||!f.resource_payments.is_empty()||!same(&f.steps,&vec![Step{context:f.actor,op:Op::BQ052PlaceCore}])||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted){return Err("BQ052原牺牲帧来源、程序或宿主损坏".into());}
        self.mobility_immutable_identity(c)?;
        let [PaidCost::BQ052Sacrificed(victims)]=f.already_paid.as_slice()else{return Err("BQ052缺少原三牺牲回执".into());};
        if victims.len()!=3||victims.iter().map(|s|&s.card.id).collect::<BTreeSet<_>>().len()!=3{return Err("BQ052牺牲数量或实例重复".into());}
        for s in victims {let c=&s.card;if c.owner>=self.players.len()||c.controller!=f.actor||!crate::tentacle::character(c)||!self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(c,f)|c<f)||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.attachment_host_instance.is_some()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("BQ052三触手回执身份损坏".into());}self.mobility_immutable_identity(c)?;}
        Ok(())
    }
    pub(crate) fn bq052_place_core(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        self.bq052_frame(f)?;if f.cursor!=1{return Err("BQ052须执行原单步结算".into());}
        if self.players[f.actor].eliminated{return Ok(());}
        let Some(index)=self.core_zones.iter().position(|z|z.owner==f.actor&&z.card.is_some())else{return Ok(());};
        let regions=(0..self.regions.len()).filter(|r|self.region_live(*r)).collect::<Vec<_>>();if regions.is_empty(){return Ok(());}
        let region=regions[self.random_below(regions.len())];let c=self.core_zones[index].card.take().unwrap();let c=self.reset_zone_card(c);let id=c.id.clone();let definition=c.definition.clone();
        self.regions[region].cards.push(c);
        // The printed effect already chose its random region; chaos must not reroll it.
        self.enter_triggers_with_extra(f.actor,&definition,&id,false,None,None,true);Ok(())
    }
    pub(crate) fn validate_bq052_state(&self)->RuleResult<()> {
        let mut ids=BTreeSet::new();
        for item in &self.stack {
            self.validate_bq052_play(item)?;
            if let Some(f)=&item.frame {
                if f.source.card.definition==ID&&!matches!(f.ability_key.as_str(),"attach"|PREPARE|PLACE){return Err("BQ052不能携带其他来源能力".into());}
                self.bq052_frame(f)?;
                if f.ability_key==PLACE&&(item.card.is_some()||item.id!=f.frame_id||item.controller!=f.actor||item.target.is_some()||item.reveal||item.deploy_region.is_some()||f.cursor!=0||!ids.insert(&f.frame_id)){return Err("BQ052牺牲帧堆叠外壳或唯一载体损坏".into());}
            }
        }
        for e in &self.effects {if let Effect::Frame{frame}=e{self.bq052_frame(frame)?;if frame.ability_key==PLACE{return Err("BQ052单步不能保留游离续体".into());}}}
        if self.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Frame{frame,..}if frame.ability_key==PLACE)){return Err("BQ052三触手是支付，不能伪造结算待选".into());}Ok(())
    }
    pub(crate) fn validate_bq052_play(&self,item:&StackItem)->RuleResult<()> {
        if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(|f|f.source.card.definition==ID&&f.ability_key=="attach"){return Ok(());}
        let f=item.frame.as_ref().ok_or("BQ052结附事务不能缺原帧")?;let s=&f.source;let c=&s.card;
        self.validate_resource_frame(f)?;
        let [t]=f.targets.as_slice()else{return Err("BQ052须有唯一原本方秘社目标".into());};
        if c.definition!=ID||f.actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=f.actor||c.face_down||s.region.is_some()||s.source_region_instance.is_some()||s.attachment_host_instance.is_some()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()
            ||f.ability_key!="attach"||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||f.chosen_region.is_some()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
            ||t.id!=format!("society:p{}",f.actor)||!same(&t.spec,&definition().abilities[0].targets[0])||t.public.instance_id!=t.id||t.public.kind!="society"||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()
            ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=2&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()&&ids.iter().all(|id|self.mystic_instance_valid(id)))
            ||item.id!=f.frame_id||item.controller!=f.actor||item.target.as_deref()!=Some(t.id.as_str())||item.reveal||item.deploy_region.is_some()||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("BQ052结附实际事务、原宿主或程序损坏".into());}
        self.mobility_immutable_identity(c)
    }
}
