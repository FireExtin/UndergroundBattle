//! Closed BQ061. Project ruling: recover exactly one eligible card, if any.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ061";
pub(crate) const KEY: &str = "bq061-recover-one";
const TITLE: &str = "钻研故纸堆：选择一张计划、任务或命令牌回手";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut a = ability(KEY, TITLE, Timing::Standard, vec![], vec![], vec![Op::BQ061Recover], None);
    a.play_only = true;
    with_abilities(vec![a])
}
pub(crate) fn related(a: &AbilitySpec) -> bool { a.key.starts_with("bq061-") || a.ops.iter().chain(a.modes.iter().flat_map(|m| &m.ops)).any(|o| matches!(o, Op::BQ061Recover)) }
fn frame_related(f: &ResolutionFrame) -> bool { f.source.card.definition == ID || f.ability_key.starts_with("bq061-") || f.steps.iter().any(|s| matches!(s.op, Op::BQ061Recover)) }
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || !same(a, &definition().abilities[0]) { return Err("钻研故纸堆须保留原单张墓地回收程序".into()); } Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == ID || d.abilities.iter().any(related)) && (id != ID || !same(d, &definition())) { return Err("钻研故纸堆完整定义不可改写或移植".into()); } Ok(())
}
pub(crate) fn validate_action(c: &Card, a: &Action) -> RuleResult<()> {
    if c.definition != ID { return Ok(()); }
    let expected = if a.kind == "asset" { Action { card_id:a.card_id.clone(), ..Action::new("asset") } }
        else { Action { card_id:a.card_id.clone(), ability_id:a.ability_id.clone(), ..Action::new("play") } };
    if *a != expected || a.ability_id.as_deref().is_some_and(|k| k != KEY) { return Err("钻研故纸堆不接受预选目标、模式或额外载荷".into()); } Ok(())
}
impl Game {
    pub(crate) fn validate_bq061_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !frame_related(f) { return Ok(()); }
        self.validate_resource_frame(f)?;
        let s=&f.source; let c=&s.card;
        if c.definition!=ID || f.actor>=self.players.len() || c.controller!=f.actor || c.owner>=self.players.len() || c.face_down
            || !self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(c,f)|c<f)
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some() || s.attachment_host_instance.is_some()
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.ability_key!=KEY || !f.targets.is_empty() || f.chosen_region.is_some()
            || !same(&f.steps,&vec![Step{context:f.actor,op:Op::BQ061Recover}]) || f.cursor>1
            || f.cursor==0&&!matches!(f.guard,GuardState::Unchecked) || f.cursor==1&&!matches!(f.guard,GuardState::Accepted)
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)==1 && ids.iter().collect::<BTreeSet<_>>().len()==ids.len() && ids.iter().all(|id|id!=&c.id&&self.mystic_instance_number(id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,f)|a<f))) {
            return Err("钻研故纸堆原来源、支付或续体损坏".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn bq061_options(&self, f:&ResolutionFrame)->Vec<ChoiceOption> {
        self.bq095_reference_cards(&f.source.card).iter().filter(|c|catalog::catalog().cards.iter().find(|d|d.id==c.definition)
            .is_some_and(|d|d.subtypes.iter().any(|s|matches!(s.as_str(),"计划"|"任务"|"命令"))))
            .map(|c|self.option(c,f.actor,None,None)).collect()
    }
    pub(crate) fn bq061_start(&mut self,f:ResolutionFrame)->RuleResult<()> {
        self.validate_bq061_frame(&f)?;
        if f.cursor!=1 {return Err("钻研故纸堆回收阶段无效".into());}
        if self.players[f.actor].eliminated {return Ok(());}
        let options=self.bq061_options(&f);
        if !options.is_empty() {self.choice(f.actor,"graveyard",TITLE.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ061Recover});}
        Ok(())
    }
    pub(crate) fn bq061_finish(&mut self,f:&ResolutionFrame,seat:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_bq061_frame(f)?;
        if f.cursor!=1 || seat!=f.actor || self.players[f.actor].eliminated || selected.len()!=1
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:Some(selected.to_vec()),..Action::new("choose")})
            || !self.bq061_options(f).iter().any(|o|o.id==selected[0]) {return Err("钻研故纸堆须选择自己墓地中的一张合法牌".into());}
        let c=self.bq095_take_reference(&f.source.card,&selected[0])?;let c=self.reset_zone_card(c);self.players[f.actor].hand.push(c);Ok(())
    }
    pub(crate) fn validate_bq061_play(&self,i:&StackItem)->RuleResult<()> {
        if i.card.as_ref().is_some_and(|c|c.definition==ID)||i.frame.as_ref().is_some_and(frame_related) {
            let f=i.frame.as_ref().ok_or("钻研故纸堆实际事务缺帧")?;self.validate_bq061_frame(f)?;
            if f.source.card.definition!=ID || f.cursor!=0 || i.id!=f.frame_id || i.controller!=f.actor || i.reveal || i.deploy_region.is_some() || i.target.is_some() || i.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)) {return Err("钻研故纸堆实际事务与原帧不符".into());}
        }Ok(())
    }
    pub(crate) fn validate_bq061_state(&self)->RuleResult<()> {
        let mut ids=BTreeSet::new();
        let mut check=|f:&ResolutionFrame|->RuleResult<()> {self.validate_bq061_frame(f)?;if frame_related(f)&&!ids.insert(f.frame_id.clone()){return Err("钻研故纸堆同帧不能重复续体".into());}Ok(())};
        for i in &self.stack {self.validate_bq061_play(i)?;if let Some(f)=&i.frame {check(f)?;}}
        for e in &self.effects {match e {Effect::Frame{frame}=>check(frame)?,Effect::Declare{declaration:d} if d.source.card.definition==ID||related(&d.ability)=>return Err("钻研故纸堆没有触发声明".into()),_=>{}}}
        if let Some(p)=&self.pending {match &p.resolution {
            ChoiceResolution::Frame{frame:f,choice} if frame_related(f)||matches!(choice,FrameChoice::BQ061Recover)=>{
                check(f)?;if f.source.card.definition!=ID {return Err("钻研故纸堆待选来源不符".into());}let options=self.bq061_options(f);
                if f.source.card.definition!=ID || !matches!(choice,FrameChoice::BQ061Recover) || f.cursor!=1 || self.players[f.actor].eliminated || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor)
                    || !self.mystic_instance_valid(&p.choice.id) || p.choice.kind!="graveyard" || p.choice.title!=TITLE || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || options.is_empty() || !same(&options,&p.choice.options) {return Err("钻研故纸堆强制单选或原墓地选项损坏".into());}
            },
            ChoiceResolution::Declare{declaration:d,..} if d.source.card.definition==ID||related(&d.ability)=>return Err("钻研故纸堆不能伪装触发待选".into()),_=>{}
        }}Ok(())
    }
}
