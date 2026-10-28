//! Pinned closed Shoggoth command: region sweep or two sequential core invocations, then repress one.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ063";
pub(crate) const KEY: &str = "bq063-command";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut a = ability(KEY, "神话威严：选择模式", Timing::Standard, vec![], vec![], vec![], None);
    a.play_only = true;
    a.modes = vec![
        Mode { key: "hide".into(), label: "核心或触手所在地区角色翻暗，然后遏制1".into(), targets: vec![], ops: vec![Op::BQ063Sweep { hide: true }, Op::BQ063Repress] },
        Mode { key: "return".into(), label: "核心或触手所在地区暗藏者回手，然后遏制1".into(), targets: vec![], ops: vec![Op::BQ063Sweep { hide: false }, Op::BQ063Repress] },
        Mode { key: "core".into(), label: "发动两次核心能力，然后遏制1".into(), targets: vec![], ops: vec![Op::BQ063InvokeCore, Op::BQ063InvokeCore, Op::BQ063Repress] },
    ];
    with_abilities(vec![a])
}
fn selected() -> Vec<AbilitySpec> {
    let a = definition().abilities.remove(0);
    let mut out = vec![a.clone()];
    for m in &a.modes { let mut b = a.clone(); b.ops = m.ops.clone(); b.modes.clear(); out.push(b); }
    out
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("bq063-") || a.ops.iter().chain(a.modes.iter().flat_map(|m| &m.ops)).any(op_related)
}
fn op_related(o: &Op) -> bool { matches!(o, Op::BQ063Sweep { .. } | Op::BQ063InvokeCore | Op::BQ063Repress) }
fn frame_related(f: &ResolutionFrame) -> bool { f.source.card.definition == ID || f.ability_key.starts_with("bq063-") || f.steps.iter().any(|s| op_related(&s.op)) }
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || !selected().iter().any(|b| same(a,b)) { return Err("神话威严须保留完整三个模式与末尾遏制".into()); } Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == ID || d.abilities.iter().any(related)) && (id != ID || !same(d,&definition())) { return Err("神话威严定义不可删改或移植".into()); } Ok(())
}
impl Game {
    pub(crate) fn bq063_action(&self, actor: usize, c: &Card, a: &Action) -> RuleResult<()> {
        if c.definition != ID { return Ok(()); }
        if *a == (Action { card_id: a.card_id.clone(), ..Action::new("asset") }) { return Ok(()); }
        if !self.has_shoggoth(actor) { return Err("神话威严为修格斯秘社限定".into()); }
        if !matches!(a.option.as_deref(),Some("hide"|"return"|"core")) || a.ability_id.as_deref().is_some_and(|k|k!=KEY)
            || *a != (Action { card_id:a.card_id.clone(), ability_id:a.ability_id.clone(), option:a.option.clone(), ..Action::new("play") }) {
            return Err("神话威严须选择一个原模式，不带目标或额外载荷".into());
        } Ok(())
    }
    pub(crate) fn validate_bq063_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !frame_related(f) { return Ok(()); }
        self.validate_resource_frame(f)?;
        let s=&f.source; let c=&s.card;
        let program=selected().into_iter().filter(|a|a.modes.is_empty()).any(|a|same(&a.ops,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>()));
        if c.definition!=ID || c.face_down || c.controller!=f.actor || c.owner>=self.players.len() || f.actor>=self.players.len()
            || !self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(c,f)|c<f)
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some() || s.attachment_host_instance.is_some()
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.ability_key!=KEY || !program || !f.targets.is_empty() || f.chosen_region.is_some() || f.steps.iter().any(|s|s.context!=f.actor)
            || f.cursor>f.steps.len() || f.cursor==0&&!matches!(f.guard,GuardState::Unchecked) || f.cursor>0&&!matches!(f.guard,GuardState::Accepted)
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=3 && ids.iter().collect::<BTreeSet<_>>().len()==ids.len() && ids.iter().all(|id|id!=&c.id&&self.mystic_instance_number(id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,f)|a<f))) {
            return Err("神话威严原来源、支付、模式或续体损坏".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn bq063_sweep(&mut self, f:&ResolutionFrame, hide:bool) -> RuleResult<()> {
        self.validate_bq063_frame(f)?;
        if f.cursor!=1 || !matches!(f.steps[0].op,Op::BQ063Sweep{hide:h} if h==hide) { return Err("神话威严清场须来自原第一步".into()); }
        // Snapshot eligible regions before the sweep hides its own core. Read the actual board after responses.
        let regions=(0..self.regions.len()).filter(|&r|self.region_live(r)&&self.region_characters(r).any(|c|
            !c.face_down&&c.controller==f.actor&&(self.current_subtypes(c).iter().any(|s|s=="触手")
                || self.core_zones.iter().any(|z|z.owner==f.actor&&z.card.is_none()&&z.definition==c.definition&&c.owner==f.actor)))).collect::<Vec<_>>();
        let ids=self.in_play_cards().into_iter().filter(|(r,c)|r.is_some_and(|r|regions.contains(&r))
            && if hide { !c.face_down&&catalog::has_kind(catalog::card(&c.definition),"character") } else { c.face_down&&!crate::batch4_plans::is_plan(c) })
            .map(|(_,c)|c.id.clone()).collect::<Vec<_>>();
        for id in ids { if hide { self.hide_in_place(&id); } else { self.return_hand(&id); } }
        Ok(())
    }
    pub(crate) fn bq063_invoke(&mut self, f:ResolutionFrame) -> RuleResult<()> {
        self.validate_bq063_frame(&f)?;
        if !matches!(f.cursor,1|2)||!matches!(f.steps[f.cursor-1].op,Op::BQ063InvokeCore) { return Err("神话威严核心发动顺序无效".into()); }
        let actor=f.actor;
        // Keep the original paid frame in the forecast continuation, never a detached cursor beside it.
        if self.selected_core_source(actor).is_ok() {
            let (core,amount)=self.selected_bq046_core_program(actor)?;
            let options=self.players[actor].deck.iter().take(amount).map(|c|self.option(c,actor,None,None)).collect::<Vec<_>>();
            if !options.is_empty() {
                let n=options.len();let invocation=f.cursor;
                self.choice(actor,"investigation",format!("预测 {n}"),options,n,n,Some(n as u32),ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ063Forecast{seat:actor,invocation,core:Box::new(core)}});
                return Ok(());
            }
        }
        self.effects.push_front(Effect::Frame{frame:Box::new(f)});
        Ok(())
    }
    fn bq063_opponents(&self,actor:usize)->Vec<ChoiceOption> {
        self.players.iter().filter(|p|!p.eliminated&&self.team(p.seat)!=self.team(actor)).map(|p|ChoiceOption{id:player_id(p.seat),label:p.name.clone(),card:None}).collect()
    }
    pub(crate) fn bq063_repress(&mut self,f:ResolutionFrame)->RuleResult<()> {
        self.validate_bq063_frame(&f)?;
        if f.cursor!=f.steps.len() {return Err("神话威严遏制须在原模式结束后".into());}
        let options=self.bq063_opponents(f.actor);
        if !options.is_empty() {self.choice(f.actor,"bq063-opponent","神话威严：选择遏制的敌方玩家".into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ063Opponent});}
        Ok(())
    }
    pub(crate) fn bq063_choose_opponent(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_bq063_frame(f)?;
        if f.cursor!=f.steps.len()||chooser!=f.actor||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1||!self.bq063_opponents(f.actor).iter().any(|o|o.id==selected[0]) {return Err("神话威严遏制对象无效".into());}
        let seat=selected[0].strip_prefix('p').and_then(|s|s.parse().ok()).ok_or("敌方玩家无效")?;
        let regions=self.batch20_repress_regions(seat);
        if !regions.is_empty() {self.choice(seat,"bq063-repress","神话威严：移除1个本方势力".into(),self.batch20_repress_options(&regions,1),1,1,Some(1),ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::BQ063Repress{seat,regions}});}
        Ok(())
    }
    pub(crate) fn bq063_finish(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String],seat:usize,regions:&[(usize,String,u32)])->RuleResult<()> {
        self.validate_bq063_frame(f)?;
        if f.cursor!=f.steps.len()||chooser!=seat||seat>=self.players.len()||self.team(seat)==self.team(f.actor)||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1||!same(&regions.to_vec(),&self.batch20_repress_regions(seat))||!self.batch20_repress_options(regions,1).iter().any(|o|o.id==selected[0]) {return Err("神话威严遏制确认无效".into());}
        self.wm040_repress_complete(seat,regions,selected)
    }
    pub(crate) fn validate_bq063_play(&self,i:&StackItem)->RuleResult<()> {
        if i.card.as_ref().is_some_and(|c|c.definition==ID)||i.frame.as_ref().is_some_and(frame_related) {
            let f=i.frame.as_ref().ok_or("神话威严实际事务缺帧")?;self.validate_bq063_frame(f)?;
            if f.source.card.definition!=ID||f.cursor!=0||i.id!=f.frame_id||i.controller!=f.actor||i.reveal||i.deploy_region.is_some()||i.target.is_some()||i.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("神话威严实际事务与原帧不符".into());}
        } Ok(())
    }
    pub(crate) fn validate_bq063_state(&self)->RuleResult<()> {
        if self.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::Forecast{..}))
            && self.effects.iter().any(|e|matches!(e,Effect::Frame{frame:f} if f.source.card.definition==ID&&matches!(f.cursor,1|2)&&matches!(f.steps.get(f.cursor-1).map(|s|&s.op),Some(Op::BQ063InvokeCore)))) {
            return Err("神话威严预测不能拆为通用待选和独立命令游标".into());
        }
        let mut ids=BTreeSet::new();
        let mut check=|f:&ResolutionFrame|->RuleResult<()> {self.validate_bq063_frame(f)?;if frame_related(f)&&!ids.insert(f.frame_id.clone()){return Err("神话威严同一帧不可并存重复续体".into());}Ok(())};
        for i in &self.stack {
            if i.card.as_ref().is_some_and(|c|c.definition==ID)||i.frame.as_ref().is_some_and(frame_related) {
                let f=i.frame.as_ref().ok_or("神话威严实际事务缺帧")?;check(f)?;
                if f.source.card.definition!=ID||f.cursor!=0||i.id!=f.frame_id||i.controller!=f.actor||i.reveal||i.deploy_region.is_some()||i.target.is_some()||i.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)) {return Err("神话威严实际事务与原帧不符".into());}
            }
        }
        for e in &self.effects {match e {Effect::Frame{frame}=>check(frame)?,Effect::Declare{declaration:d} if d.source.card.definition==ID||related(&d.ability)=>return Err("神话威严没有触发声明".into()),_=>{}}}
        if let Some(p)=&self.pending {match &p.resolution {
            ChoiceResolution::Frame{frame:f,choice} if frame_related(f)||matches!(choice,FrameChoice::BQ063Forecast{..}|FrameChoice::BQ063Opponent|FrameChoice::BQ063Repress{..})=>{
                check(f)?;
                if let FrameChoice::BQ063Forecast{seat,invocation,core}=choice {
                    let (actual,count)=self.selected_bq046_core_program(*seat)?;
                    let options=self.players[*seat].deck.iter().take(count).map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();let n=options.len();
                    if f.source.card.definition!=ID||f.ability_key!=KEY||*seat!=f.actor||!matches!(*invocation,1|2)||f.cursor!=*invocation||!matches!(f.steps[f.cursor-1].op,Op::BQ063InvokeCore)||!same(core.as_ref(),&actual)
                        ||n==0||p.seat!=*seat||p.choice.player_id!=player_id(*seat)||!self.mystic_instance_valid(&p.choice.id)||p.choice.kind!="investigation"||p.choice.title!=format!("预测 {n}")||p.choice.amount!=Some(n as u32)||p.choice.min!=Some(n)||p.choice.max!=Some(n)||p.choice.allow_decline!=Some(false)||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options){return Err("神话威严预测须绑定原命令、发动序号、核心和当前预测牌".into());}
                    return Ok(());
                }
                let(seat,kind,title,amount,options)=match choice {
                    FrameChoice::BQ063Opponent=>(f.actor,"bq063-opponent","神话威严：选择遏制的敌方玩家",None,self.bq063_opponents(f.actor)),
                    FrameChoice::BQ063Repress{seat,regions}=>{
                        if *seat>=self.players.len()||self.team(*seat)==self.team(f.actor)||regions.is_empty()||!same(regions,&self.batch20_repress_regions(*seat)){return Err("神话威严原遏制地区不符".into());}
                        (*seat,"bq063-repress","神话威严：移除1个本方势力",Some(1),self.batch20_repress_options(regions,1))
                    },_=>return Err("神话威严不可移植其他待选种类".into())};
                if f.ability_key!=KEY||f.source.card.definition!=ID||f.cursor!=f.steps.len()||p.seat!=seat||p.choice.player_id!=player_id(seat)||!self.mystic_instance_valid(&p.choice.id)||p.choice.kind!=kind||p.choice.title!=title||p.choice.amount!=amount||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options){return Err("神话威严待选合同损坏".into());}
            },
            ChoiceResolution::Declare{declaration:d,..} if d.source.card.definition==ID||related(&d.ability)=>return Err("神话威严没有触发待选".into()),_=>{}
        }}
        Ok(())
    }
}
