//! WM041 distinguishes a paid face-up character play from reveal/direct entry.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
const KEY:&str="batch3-wm041-public-play";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPlayObservation{pub event_id:String,pub turn:u32,pub played:SourceSnapshot,pub exposure_at_play:u32}
pub(crate) fn definition()->Definition{
    let mut d=with_abilities(vec![ability(KEY,"强制：打出公开角色，本方秘社放置暴露1，然后抓1",Timing::Fast,vec![],vec![],vec![Op::WM041PublicPlay{observation:None}],Some(Event::WM041PublicCharacterPlayed))]);
    d.modifiers.push(StaticModifier::WM041RenownControl);d
}
impl Game{
    fn wm041_cards(&self)->impl Iterator<Item=(usize,&Card)>{self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,region)|region.cards.iter().map(move |c|(r,c))).filter(|(_,c)|c.definition=="WM041" && !c.face_down && !self.players[c.controller].eliminated)}
    fn wm041_sources(&self)->Vec<SourceSnapshot>{self.wm041_cards().map(|(r,c)|self.source_snapshot(c,Some(r))).collect()}
    pub(crate) fn wm041_renown_denied(&self,c:&Card)->bool{self.wm041_cards().any(|(_,s)|self.team(s.controller)!=self.team(c.controller))}
    pub(crate) fn wm041_renown_granted(&self,c:&Card)->bool{
        (c.definition=="WM041" && self.players[c.controller].society_zone.exposure>0)
            || (self.board(&c.id).is_some_and(|(_,live)|live==c) && self.current_public(c,c.controller) && self.wm041_cards().any(|(_,s)|s.controller==c.controller))
    }
    pub(crate) fn wm041_observe_last_play(&mut self){
        let sources=self.wm041_sources();if sources.is_empty(){return;}
        let Some(f)=self.stack.last().and_then(|s|s.frame.as_ref())else{return;};
        if f.ability_key!="deploy" || !self.current_public(&f.source.card,f.actor){return;}
        let observation=PublicPlayObservation{event_id:f.frame_id.clone(),turn:self.turn,played:f.source.clone(),exposure_at_play:self.players[f.actor].society_zone.exposure};
        for source in sources.into_iter().filter(|s|s.card.controller==f.actor){
            let mut a=definition().abilities.remove(0);a.ops=vec![Op::WM041PublicPlay{observation:Some(observation.clone())}];
            self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});
        }
    }
    pub(crate) fn wm041_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let [Op::WM041PublicPlay{observation:Some(o)}]=ops else{return Err("阿尔法先生缺少实际公开打出观察".into());};let p=&o.played;let c=&p.card;
        if s.card.definition!="WM041" || s.card.controller!=actor || s.card.face_down || s.play_source.is_some() || actor>=self.players.len() || c.id.is_empty() || c.id==s.card.id || c.face_down || c.controller!=actor || c.owner>=self.players.len()
            || p.region.is_none_or(|r|r>=self.regions.len()) || p.attachment_host_instance.is_some() || p.observed_death.is_some() || p.observed_mobility_grant.is_some() || p.observed_exposure.is_some()
            || !catalog::catalog().cards.iter().any(|d|d.id==c.definition && d.kind=="character") || !p.valid_character_play() || !(crate::rules::definition(&c.definition).traits.public || (c.definition=="WM040" && o.exposure_at_play>0))
            || o.turn!=self.turn || o.event_id==c.id || o.event_id==s.card.id || o.event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n==0 || n>self.sequence)
            || self.stack.iter().any(|item|item.id==o.event_id && item.frame.as_ref().is_none_or(|f|f.ability_key!="deploy" || f.actor!=actor || serde_json::to_value(&f.source).unwrap()!=serde_json::to_value(p).unwrap())){
            return Err("阿尔法先生观察须保留原玩家、公开条件和实际打出来源".into());}
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn batch3_forced_declare(&mut self,d:&Declaration)->RuleResult<bool>{
        if d.ability.event!=Some(Event::WM041PublicCharacterPlayed){return Ok(false);}
        self.validate_batch3_declaration(d)?;
        if !self.players[d.actor].eliminated{let f=self.make_frame(d.actor,d.source.clone(),&d.ability,vec![],vec![],None);self.dispatch_frame(f,d.ability.label.clone(),None,ResponsePolicy::Respondable);}Ok(true)
    }
    pub(crate) fn batch3_reject_optional(&self,d:&Declaration)->RuleResult<()>{if d.ability.event==Some(Event::WM041PublicCharacterPlayed){return Err("阿尔法先生公开打出奖励是强制触发".into());}Ok(())}
}
