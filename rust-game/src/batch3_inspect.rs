//! Investigation File has zero, one or two explicit targets. The normal whole
//! frame guard runs once before private inspection and the later one-card draw.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
const KEY: &str = "batch3-jc062-investigate";
const TITLE: &str = "调查档案：检视所选暗藏者";
const KIND: &str = "jc062-inspection";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut t = target(Zone::Board, EntityKind::Hidden, Relation::Any, Range::Anywhere); t.min=0; t.max=2;
    let mut a = ability(KEY, "快速行动：检视至多两个暗藏者，然后抓一张牌", Timing::Fast, vec![], vec![t],
        vec![Op::JC062Inspect, Op::Draw { player: PlayerRef::Actor, count: 1, end: DeckEnd::Top }], None);
    a.play_only = true; crate::rules::with_abilities(vec![a])
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.selected.as_ref().is_some_and(|ids| ids.len()>2)
        || a.ability_id.as_deref().is_some_and(|id| id != KEY)
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), selected: a.selected.clone(), ..Action::new("play") }) {
        return Err("调查档案须在打出前明确指定零至两个不同的暗藏者".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jc062_action_candidates(&self, actor: usize, c: &Card) -> Vec<(Action,String)> {
        let source=self.source_snapshot(c,None); let d=definition(); let options=self.binding_options(actor,&source,&d.abilities[0].targets[0]);
        let mut sets=vec![vec![]];
        for (i,a) in options.iter().enumerate() {
            sets.push(vec![a.id.clone()]);
            for b in options.iter().skip(i+1) { sets.push(vec![a.id.clone(),b.id.clone()]); }
        }
        sets.into_iter().map(|ids| (Action { card_id:Some(c.id.clone()),ability_id:Some(KEY.into()),selected:Some(ids.clone()),..Action::new("play") },
            if ids.is_empty() { "调查档案：不指定目标，抓一张牌".into() } else { format!("调查档案：检视 {}",ids.iter().map(|id| options.iter().find(|o| o.id==*id).unwrap().label.clone()+" ["+id+"]").collect::<Vec<_>>().join("、")) })).collect()
    }
    pub(crate) fn jc062_bind(&self, actor:usize,source:&SourceSnapshot,a:&AbilitySpec,action:&Action)->RuleResult<Vec<BoundTarget>> {
        validate_action(action)?;
        let ids=action.selected.clone().unwrap_or_default();
        if ids.iter().collect::<BTreeSet<_>>().len()!=ids.len() { return Err("调查档案不能重复指定同一暗藏者".into()); }
        let spec=&a.targets[0];
        ids.into_iter().map(|id| {
            if !self.valid_binding(actor,source,spec,&id) { return Err("调查档案目标不是场上暗藏者".into()); }
            Ok(BoundTarget { public:self.public_target(actor,source,spec,&id),id,spec:spec.clone(),region_instance:None })
        }).collect()
    }
    pub(crate) fn validate_jc062_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let c=&f.source.card; let d=definition(); let a=&d.abilities[0];
        if c.definition!="JC062" || c.id.is_empty() || c.face_down || c.owner>=self.players.len() || f.actor>=self.players.len() || c.controller!=f.actor
            || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some() || f.source.observed_death.is_some() || f.source.observed_mobility_grant.is_some() || f.source.observed_exposure.is_some()
            || f.frame_id.is_empty() || f.ability_key!=KEY || f.chosen_region.is_some() || f.cursor>2
            || (f.cursor==0 && !matches!(f.guard,GuardState::Unchecked)) || (f.cursor>0 && !matches!(f.guard,GuardState::Accepted))
            || !same(&f.steps,&a.ops.iter().map(|op|Step{context:f.actor,op:op.clone()}).collect::<Vec<_>>()) || f.targets.len()>2
            || f.targets.iter().map(|t|&t.id).collect::<BTreeSet<_>>().len()!=f.targets.len()
            || f.targets.iter().any(|t| !same(&t.spec,&a.targets[0]) || t.id.is_empty() || t.public.instance_id!=t.id || t.public.kind!="hidden"
                || !t.public.valid || t.public.status!="valid" || t.public.invalid_reason.is_some() || t.region_instance.is_some()
                || t.public.owner.as_deref().is_none_or(|id|!self.players.iter().any(|p|player_id(p.seat)==id))
                || t.public.controller.as_deref().is_none_or(|id|!self.players.iter().any(|p|player_id(p.seat)==id))
                || t.public.region.is_none_or(|r|r>=self.regions.len()))
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()<=1 && ids.iter().all(|id|!id.is_empty() && *id!=c.id && f.targets.iter().all(|t|t.id!=*id)) && ids.iter().collect::<BTreeSet<_>>().len()==ids.len()) {
            return Err("调查档案的原来源、费用、零至二目标或执行阶段不符".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn jc062_inspect_start(&mut self,f:ResolutionFrame)->RuleResult<bool> {
        self.validate_jc062_frame(&f)?;
        if f.targets.is_empty() { return Ok(false); }
        let inspected=f.targets.iter().map(|t|self.board(&t.id).map(|(_,c)|c.clone()).ok_or_else(||"检视原暗藏者失效".into())).collect::<RuleResult<Vec<_>>>()?;
        for c in &inspected {self.observe_hidden_inspection(f.actor,&c.id);}
        self.choice(f.actor,KIND,TITLE.into(),vec![],0,0,None,ChoiceResolution::Frame {frame:Box::new(f),choice:FrameChoice::JC062Inspection {inspected}});
        self.pending.as_mut().unwrap().choice.allow_decline=Some(false);
        Ok(true)
    }
    pub(crate) fn validate_jc062_pending(&self,p:&Pending,f:&ResolutionFrame,inspected:&[Card])->RuleResult<()> {
        self.validate_jc062_frame(f)?;
        if f.cursor!=1 || f.targets.is_empty() || inspected.len()!=f.targets.len()
            || inspected.iter().zip(&f.targets).any(|(c,t)| c.id!=t.id || !c.face_down || c.owner>=self.players.len() || c.controller>=self.players.len()
                || !catalog::catalog().cards.iter().any(|d|d.id==c.definition && d.kind=="character")
                || self.board(&c.id).is_none_or(|(_,live)|live!=c))
            || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty()
            || p.choice.kind!=KIND || p.choice.title!=TITLE || !p.choice.options.is_empty() || !p.choice.preview_cards.is_empty()
            || p.choice.min!=Some(0) || p.choice.max!=Some(0) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() {
            return Err("调查档案的私密检视快照、选择者或确认阶段不符".into());
        }
        Ok(())
    }
    pub(crate) fn jc062_inspect_complete(&self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String],inspected:&[Card])->RuleResult<()> {
        self.validate_jc062_frame(f)?;
        if chooser!=f.actor || f.cursor!=1 || inspected.len()!=f.targets.len() || !selected.is_empty()
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {
            return Err("调查档案只能由原玩家确认私密检视".into());
        }
        Ok(())
    }
    pub(crate) fn jc062_private_choice(&self,p:&Pending,viewer:usize)->Choice {
        let mut choice=p.choice.clone();
        if let ChoiceResolution::Frame {frame,choice:FrameChoice::JC062Inspection{inspected}}=&p.resolution {
            if viewer==frame.actor { choice.preview_cards=inspected.iter().map(|c|{let mut front=c.clone();front.face_down=false;self.card_view(&front,viewer,None,None)}).collect(); }
        }
        choice
    }
}
