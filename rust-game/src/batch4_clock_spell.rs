//! WM037's normal spell and personal shown-card continuation. No new contest,
//! draw, payment or persistence kernel: all continue existing shared frames.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::{ContestOutcome,MysticPhase}, rules::*};
use std::collections::BTreeSet;
pub(crate) const PLAY:&str="clock-wm037-play";
pub(crate) const SHOWN:&str="clock-wm037-shown";
const MAY_KIND:&str="wm037-optional-contest";
const MAY_TITLE:&str="戴森的计谋：是否进行一次隐秘对抗？";
const TARGET_KIND:&str="wm037-shown-time-target";
const TARGET_TITLE:&str="戴森的计谋：计时者在目标角色或附属放置时间1";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool {serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition {
    let mut a=ability(PLAY,"行动：弃1，牌库底两张入手；场上时间至少4可隐秘对抗",Timing::Standard,vec![],vec![],
        vec![Op::Discard{player:PlayerRef::Actor,count:Some(1),redraw:false,optional:false},
            Op::MoveBottomToHand{player:PlayerRef::Actor,count:2},Op::MysticContest{actor_bonus:0}],None);
    a.play_only=true;with_abilities(vec![a])
}
fn owns(ops:&[Op])->bool {
    ops.iter().any(|op|match op {
        Op::WM037ShownTime{..}=>true,
        Op::ForEachLivingPlayer(body)|Op::ForEachLivingPlayerFromActor(body)=>owns(body),
        Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),
        _=>false,
    })
}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {
    if (id=="WM037"||a.key.starts_with("clock-wm037-")||owns(&a.ops)||a.modes.iter().any(|m|owns(&m.ops)))
        && (id!="WM037"||!same(a,&definition().abilities[0])) {
        return Err("戴森的计谋只能使用完整印刷行动，展示后置不能移植为静态能力".into());
    }
    Ok(())
}
pub(crate) fn validate_action(a:&Action)->RuleResult<()> {
    if *a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("play")})
        || a.ability_id.as_deref().is_some_and(|key|key!=PLAY) {return Err("戴森的计谋打出不指定目标或额外弃牌费用".into());}
    Ok(())
}
impl Game {
    fn wm037_has_four_time(&self)->bool {
        // Physical markers are counted once. A host's inherited projection must
        // not count the same attached counter again toward the global threshold.
        self.in_play_cards().into_iter().map(|(_,c)|u64::from(c.time_markers))
            .chain(self.regions.iter().filter(|r|!r.vacant).map(|r|u64::from(r.card.time_markers)))
            .chain(self.players.iter().filter(|p|!p.eliminated).filter_map(|p|p.society_zone.card.as_ref()).map(|c|u64::from(c.time_markers)))
            .sum::<u64>()>=4
    }
    pub(crate) fn wm037_offer_contest(&mut self,f:ResolutionFrame)->RuleResult<()> {
        self.validate_wm037_frame(&f)?;
        if f.cursor!=3 {return Err("戴森的计谋只能在弃牌及牌库底入手后询问对抗".into());}
        if !self.wm037_has_four_time()||self.players[f.actor].eliminated {return Ok(());}
        self.choice(f.actor,MAY_KIND,MAY_TITLE.into(),vec![ChoiceOption{id:"contest".into(),label:"进行一次隐秘对抗".into(),card:None}],0,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM037Contest});Ok(())
    }
    pub(crate) fn wm037_choose_contest(&mut self,f:ResolutionFrame,seat:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_wm037_frame(&f)?;
        if seat!=f.actor||f.cursor!=3||!self.wm037_has_four_time()||selected.len()>1
            || selected.first().is_some_and(|s|s!="contest")
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {
            return Err("戴森的计谋只能由原玩家决定此次可选对抗".into());
        }
        if selected.is_empty(){Ok(())}else{self.mystic_contest_start(f,0)}
    }
    pub(crate) fn wm037_shown(&mut self,o:&ContestOutcome,index:usize)->RuleResult<()> {
        let shown=o.shown.get(index).ok_or("戴森展示后置索引无效")?;
        if shown.card.definition!="WM037"||self.players[shown.seat].eliminated
            || !self.players[shown.seat].society_zone.card.as_ref().is_some_and(|c|c.definition=="MSJC13"&&!c.face_down) {return Ok(());}
        let f=ResolutionFrame{frame_id:self.id(),ability_key:SHOWN.into(),actor:shown.seat,source:self.source_snapshot(&shown.card,None),
            targets:vec![],already_paid:vec![],guard:GuardState::Accepted,cursor:1,
            steps:vec![Step{context:shown.seat,op:Op::WM037ShownTime{contest_id:o.contest_id.clone(),index}}],chosen_region:None};
        let Some(state)=&mut self.mystic_contest else{return Err("戴森展示后置缺少原对抗".into());};
        let MysticPhase::After{post_frame,..}=&mut state.phase else{return Err("戴森展示后置阶段无效".into());};
        *post_frame=Some(Box::new(f.clone()));self.validate_wm037_frame(&f)?;
        let options=self.binding_options(f.actor,&f.source,&crate::batch4_clock::time_target());
        if options.is_empty(){return Ok(());}
        self.choice(f.actor,TARGET_KIND,TARGET_TITLE.into(),options,1,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM037TimeTarget});Ok(())
    }
    pub(crate) fn wm037_choose_time(&mut self,f:&ResolutionFrame,seat:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_wm037_frame(f)?;
        if seat!=f.actor||f.ability_key!=SHOWN||selected.len()!=1
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {
            return Err("戴森展示后置须由原本人选择一个合法角色或附属".into());
        }
        let spec=crate::batch4_clock::time_target();
        if !self.valid_binding(f.actor,&f.source,&spec,&selected[0]) {return Err("戴森展示后置目标无效".into());}
        if !self.shield_stops(f.actor,&selected[0]) {
            let c=self.board_mut(&selected[0]).ok_or("原时间目标已离场")?;c.time_markers=c.time_markers.saturating_add(1);
        }
        Ok(())
    }
    pub(crate) fn validate_wm037_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let c=&f.source.card;
        if c.definition!="WM037"||c.face_down||f.actor>=self.players.len()||c.controller!=f.actor||c.owner>=self.players.len()
            || !self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==c.id
            || f.source.region.is_some()||f.source.source_region_instance.is_some()||f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()||f.chosen_region.is_some()||!f.targets.is_empty() {
            return Err("戴森的计谋须保留原事务实例及个人操控者".into());
        }
        self.mobility_immutable_identity(c)?;
        if f.ability_key==SHOWN {
            let [Step{context,op:Op::WM037ShownTime{contest_id,index}}]=f.steps.as_slice() else{return Err("戴森展示后置缺少原结果收据".into());};
            let state=self.mystic_states().find(|s|s.contest_id==*contest_id).ok_or("戴森展示后置缺少原层")?;
            let MysticPhase::After{outcome,next_shown,post_frame}=&state.phase else{return Err("戴森展示后置非展示阶段".into());};
            let shown=outcome.shown.get(*index).ok_or("戴森原展示索引无效")?;
            if *context!=shown.seat||shown.seat!=f.actor||shown.card.definition!="WM037"||*next_shown!=index+1
                || !same(&f.source,&self.source_snapshot(&shown.card,None))||!f.already_paid.is_empty()||f.cursor!=1||!matches!(f.guard,GuardState::Accepted)
                || post_frame.as_ref().is_none_or(|p|!same(p.as_ref(),f))
                || !self.players[shown.seat].society_zone.card.as_ref().is_some_and(|c|c.definition=="MSJC13"&&!c.face_down) {
                return Err("戴森后置须绑定本人计时者及同次展示结果".into());
            }
            return Ok(());
        }
        let a=definition().abilities.remove(0);
        if f.ability_key!=PLAY||f.cursor>a.ops.len()||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)
            || f.cursor>0&&!matches!(f.guard,GuardState::Accepted)
            || f.steps.len()!=a.ops.len()||f.steps.iter().zip(&a.ops).any(|(s,op)|s.context!=f.actor||!same(&s.op,op))
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()<=2
                && ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()) {
            return Err("戴森的计谋须保留费用、弃一/底二入手/可选对抗完整次序".into());
        }
        Ok(())
    }
    pub(crate) fn validate_wm037_state(&self)->RuleResult<()> {
        let related=|f:&ResolutionFrame|f.source.card.definition=="WM037"||f.ability_key.starts_with("clock-wm037-")
            || f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)));
        for item in &self.stack {if let Some(f)=&item.frame {if related(f){self.validate_wm037_frame(f)?;}}}
        for e in &self.effects {if let Effect::Frame{frame:f}=e {if related(f){self.validate_wm037_frame(f)?;}}}
        if let Some(p)=&self.pending {
            if let ChoiceResolution::Frame{frame:f,choice}=&p.resolution {
                if !related(f){return Ok(());}self.validate_wm037_frame(f)?;
                let (kind,title,options,min)=match choice {
                    FrameChoice::WM037Contest if f.cursor==3&&f.ability_key==PLAY&&self.wm037_has_four_time()=>
                        (MAY_KIND,MAY_TITLE,vec![ChoiceOption{id:"contest".into(),label:"进行一次隐秘对抗".into(),card:None}],0),
                    FrameChoice::WM037TimeTarget if f.ability_key==SHOWN=>
                        (TARGET_KIND,TARGET_TITLE,self.binding_options(f.actor,&f.source,&crate::batch4_clock::time_target()),1),
                    // Shared discard, contest commit and reward continuations
                    // retain their own existing validators and privacy policy.
                    FrameChoice::Discard{seat,redraw:false} if f.ability_key==PLAY&&f.cursor==1&&*seat==f.actor=>return Ok(()),
                    FrameChoice::MysticCommit{..}|FrameChoice::MysticReward{..} if f.ability_key==PLAY=>return Ok(()),
                    _=>return Err("戴森待选与原执行阶段不符".into()),
                };
                if p.seat!=f.actor||p.choice.player_id!=player_id(f.actor)||p.choice.kind!=kind||p.choice.title!=title
                    || p.choice.min!=Some(min)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(min==0)
                    || p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options) {
                    return Err("戴森待选只能属于原本人及当前合法选项".into());
                }
            }
        }
        Ok(())
    }
}
