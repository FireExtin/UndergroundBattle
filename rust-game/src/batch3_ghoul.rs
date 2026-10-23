//! BQ079 observes actual arrival in the owner's graveyard, not character death.
//! A removal wave freezes all observers before any simultaneous source leaves.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};

const KEY:&str="batch3-bq079-corpse";
const UPKEEP:&str="亡嚎食尸鬼：将一张本方墓地牌置于牌库底";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpseGraveObservation { pub event_id:String,pub turn:u32,pub arrived:Card }
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition {
    with_abilities(vec![
        ability("batch3-bq079-prepare",UPKEEP,Timing::Fast,vec![],vec![],vec![Op::Batch3Prepare{event_id:None}],Some(Event::Prepare)),
        ability(KEY,"无名尸体进入本方墓地：自身获得永久战斗1和防御+1标志",Timing::Fast,vec![],vec![],vec![Op::BQ079CorpseIcons{observation:None}],Some(Event::BQ079CorpseGrave)),
    ])
}
impl Game {
    pub(crate) fn bq079_sources(&self)->Vec<SourceSnapshot>{
        self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,region)|region.cards.iter()
            .filter(|c|c.definition=="BQ079" && !c.face_down && !self.players[c.controller].eliminated)
            .map(move |c|self.source_snapshot(c,Some(r)))).collect()
    }
    /// The caller has performed its original zone reset. This is the single
    /// physical graveyard commit; callers keep their own death/discard events.
    pub(crate) fn commit_owner_grave(&mut self,c:Card,frozen:Option<&[SourceSnapshot]>){
        let owner=c.owner;
        let observes=catalog::card(&c.definition).name=="无名尸体" && !self.players[owner].eliminated;
        let sources=if observes{frozen.map(<[SourceSnapshot]>::to_vec).unwrap_or_else(||self.bq079_sources())}else{vec![]};
        let sources=sources.into_iter().filter(|s|s.card.controller==owner).collect::<Vec<_>>();
        if sources.is_empty(){self.players[owner].graveyard.push(c);return;}
        let observation=CorpseGraveObservation{event_id:self.id(),turn:self.turn,arrived:c.clone()};
        self.players[owner].graveyard.push(c);
        for source in sources{
            let mut a=definition().abilities.remove(1);
            a.ops=vec![Op::BQ079CorpseIcons{observation:Some(observation.clone())}];
            self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});
        }
    }
    pub(crate) fn batch3_physical_cards(&self)->impl Iterator<Item=&Card>{
        self.regions.iter().flat_map(|r|std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p|p.hand.iter().chain(&p.deck).chain(&p.assets).chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a|&a.card)).chain(self.sealed_cards.iter().map(|s|&s.card))
            .chain(self.stack.iter().filter_map(|s|s.card.as_ref())).chain(self.effects.iter().filter_map(|e|if let Effect::Bury{card}=e{Some(card)}else{None}))
    }
    pub(crate) fn bq079_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let [Op::BQ079CorpseIcons{observation:Some(o)}]=ops else{return Err("食尸鬼缺少唯一入墓观察".into());};
        let c=&o.arrived;
        if s.card.definition!="BQ079" || s.card.face_down || s.card.controller!=actor || actor>=self.players.len()
            || s.play_source.is_some() || s.region.is_none() || s.source_region_instance.is_none()
            || c.id.is_empty() || c.owner!=actor || c.controller!=actor || c.face_down || c.exhausted || c.damage!=0 || c.wounds!=0 || c.shield!=0 || c.lock_markers!=0 || c.time_markers!=0 || c.permanent_combat_markers!=0 || c.permanent_defense_markers!=0 || c.named_character.is_some()
            || !catalog::catalog().cards.iter().any(|d|d.id==c.definition && d.name=="无名尸体")
            || o.turn!=self.turn || o.event_id==c.id || o.event_id==s.card.id
            || o.event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n==0 || n>self.sequence)
            || self.batch3_physical_cards().any(|live|live.id==c.id && (!same(live,c) || !self.players[actor].graveyard.iter().any(|grave|grave==live)))
            || self.batch3_physical_cards().any(|live|live.id==s.card.id && !self.board(&s.card.id).is_some_and(|(_,board)|board.id==live.id && board.definition==s.card.definition && board.owner==s.card.owner)) {
            return Err("食尸鬼入墓观察的牌名、拥有者、原来源或区域不符".into());
        }
        self.mobility_immutable_identity(c)?;
        Ok(())
    }
    pub(crate) fn bq079_reward(&mut self,source:&SourceSnapshot){
        if let Some(c)=self.board_mut(&source.card.id).filter(|c|c.definition=="BQ079" && !c.face_down){
            c.permanent_combat_markers=c.permanent_combat_markers.saturating_add(1);
            c.permanent_defense_markers=c.permanent_defense_markers.saturating_add(1);
        }
    }
    pub(crate) fn validate_bq079_markers(&self)->RuleResult<()> {
        // XQ20 may move the physical defense markers off the original Ghoul;
        // fresh/private-zone entities still cannot retain them.
        if self.batch3_physical_cards().any(|c|c.permanent_defense_markers>0 && !self.marker_move_carrier(&c.id)){return Err("永久防御标志只能留在有效在场角色或附属".into());}
        Ok(())
    }
    pub(crate) fn bq079_upkeep_start(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        if self.players[f.actor].graveyard.is_empty(){self.remove_dead(&f.source.card.id,RemovalCause::Sacrifice);}
        else{let options=self.players[f.actor].graveyard.iter().map(|c|self.option(c,f.actor,None,None)).collect();self.choice(f.actor,"batch3-bq079-bottom",UPKEEP.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::BQ079Upkeep});}Ok(())
    }
    pub(crate) fn validate_bq079_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_jc035_frame(f)?;
        let options=self.players[f.actor].graveyard.iter().map(|c|self.option(c,f.actor,None,None)).collect::<Vec<_>>();
        if f.source.card.definition!="BQ079" || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || options.is_empty() || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty()
            || p.choice.kind!="batch3-bq079-bottom" || p.choice.title!=UPKEEP || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&options,&p.choice.options){return Err("食尸鬼强制准备待选与原来源或墓地不符".into());}Ok(())
    }
    pub(crate) fn bq079_upkeep_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if chooser!=f.actor || f.source.card.definition!="BQ079" || f.cursor!=1 || selected.len()!=1 || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("食尸鬼准备确认包含非准入载荷".into());}
        let i=self.players[f.actor].graveyard.iter().position(|c|c.id==selected[0]).ok_or("食尸鬼须选择本方墓地一张牌")?;
        let c=self.players[f.actor].graveyard.remove(i);let c=self.reset_zone_card(c);self.players[f.actor].deck.push(c);Ok(())
    }
    pub(crate) fn validate_bq079_trigger_pending(&self,p:&Pending,d:&Declaration,stage:&DeclareChoice)->RuleResult<()> {
        self.validate_batch3_declaration(d)?;
        if d.ability.key!=KEY || !matches!(stage,DeclareChoice::Accept) || p.seat!=d.actor || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty()
            || p.choice.title!=format!("{}：{}",catalog::card("BQ079").name,d.ability.label) || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options,&vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}]) {return Err("食尸鬼入墓触发须保留原可选声明".into());}Ok(())
    }
}
