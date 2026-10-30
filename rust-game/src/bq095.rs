//! Closed BQ095 persistent-zone WIP. Card entry and reference consumers remain unwired.
//! A headquarters is a distinct zone, never an alias for the physical grave.
//! Entry dispatch, repeated creation and whole-engine consumers must be wired
//! before this module can be used by the live rules interpreter.
#![allow(dead_code)]
use crate::{engine::RuleResult, model::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ID: &str = "BQ095";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Headquarters {
    pub id: String,
    pub seat: usize,
    pub source_instance: String,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Destination {
    Graveyard(usize),
    Headquarters(String),
}

impl Headquarters {
    /// Caller must dispatch the actual resolved BQ095 entry. This primitive
    /// only implements the first-created zone;
    /// it makes no ruling about repeated entries creating same-named zones.
    pub(crate) fn first_entry(
        game: &mut Game,
        source: &Card,
        host: &str,
    ) -> RuleResult<Self> {
        if source.definition != ID
            || source.face_down
            || host != format!("society:p{}", source.controller)
            || source.id.is_empty()
            || source.controller >= game.players.len()
        {
            return Err("BQ095总部创建须有实际正面来源、本方秘社与独立区域身份".into());
        }
        let mut zone = Self {
            // Replaced by a fresh engine identity after all input validation.
            id: String::new(),
            seat: source.controller,
            source_instance: source.id.clone(),
            cards: vec![],
        };
        // Validate before moving anything: failure must leave the grave intact.
        zone.validate_cards(&game.players[source.controller].graveyard)?;
        zone.id = game.id();
        let mut cards=vec![];while !game.players[source.controller].graveyard.is_empty(){cards.push(game.take_grave_observed(source.controller,0));}
        zone.cards = cards.into_iter().map(|c| game.reset_zone_card(c)).collect();
        Ok(zone)
    }

    pub(crate) fn validate(&self) -> RuleResult<()> {
        if self.id.is_empty() || self.source_instance.is_empty() || self.id == self.source_instance {
            return Err("BQ095总部缺少独立区域或原来源身份".into());
        }
        self.validate_cards(&self.cards)
    }

    fn validate_cards(&self, cards: &[Card]) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        if cards.iter().any(|c| {
            c.owner != self.seat || c.id.is_empty() || c.id == self.id
                || c.id == self.source_instance || !ids.insert(&c.id)
        }) {
            return Err("BQ095总部迁移须保持原墓地牌的拥有者和唯一实体".into());
        }
        Ok(())
    }

    /// Only an owned card's printed *own graveyard* reference is rewritten.
    /// Controller changes cannot transfer ownership of the lasting modifier.
    pub(crate) fn own_printed_reference(&self, source: &Card) -> Destination {
        if source.owner == self.seat {
            Destination::Headquarters(self.id.clone())
        } else {
            Destination::Graveyard(source.controller)
        }
    }

    /// Generic/opponent/all-grave references remain physical grave references.
    pub(crate) fn ordinary_grave_reference(seat: usize) -> Destination {
        Destination::Graveyard(seat)
    }

    /// Destination replacement applies independently of the effect's owner.
    /// The caller retains death/discard events, but must not emit a physical
    /// grave-arrival event when the resolved destination is headquarters.
    pub(crate) fn incoming_destination(&self, grave_seat: usize) -> Destination {
        if grave_seat == self.seat {
            Destination::Headquarters(self.id.clone())
        } else {
            Destination::Graveyard(grave_seat)
        }
    }

    /// Receive a card after the existing caller has applied its zone reset.
    /// This never copies a card into a second container or changes its identity.
    pub(crate) fn receive(&mut self, card: Card) -> RuleResult<()> {
        self.validate()?;
        self.validate_cards(std::slice::from_ref(&card))?;
        if self.cards.iter().any(|old| old.id == card.id) {
            return Err("BQ095总部不能重复接收同一实体".into());
        }
        self.cards.push(card);
        Ok(())
    }

    /// The caller still checks the printed target's kind/subtype/cost and any
    /// bound-frame guard. This removes only the resolved zone's actual card.
    pub(crate) fn take_for_own_printed_reference(
        &mut self,
        source: &Card,
        id: &str,
    ) -> RuleResult<Card> {
        self.validate()?;
        if self.own_printed_reference(source) != Destination::Headquarters(self.id.clone()) {
            return Err("非本方拥有的牌不能借墓地引用访问集团总部".into());
        }
        let index = self.cards.iter().position(|c| c.id == id)
            .ok_or("BQ095总部原目标已失效")?;
        Ok(self.cards.remove(index))
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadquartersView {
    pub id: String,
    pub player_id: String,
    pub name: String,
    pub cards: Vec<crate::model::CardView>,
}

impl Game {
    /// Internal resolved-entry primitive, not a public action. Until the full
    /// printed-reference integration lands, no card dispatch invokes it.
    /// Project ruling: repeated entry reuses this society's one lasting zone.
    pub(crate) fn bq095_create_or_reuse(&mut self, source: &Card, host: &str) -> RuleResult<String> {
        if source.definition != ID || source.face_down || source.controller >= self.players.len()
            || host != format!("society:p{}",source.controller)
            || !self.mystic_instance_valid(&source.id) {
            return Err("总部进场来源或本方秘社无效".into());
        }
        self.validate_bq095_state()?;
        if let Some(zone)=self.bq095_headquarters.iter().find(|z|z.seat==source.controller) {
            // Destination replacement will keep the physical grave empty.
            // Never destroy/merge unexplained physical contents on re-entry.
            if !self.players[source.controller].graveyard.is_empty() {
                return Err("已有总部仍存在物理墓地牌，不能重进场掩盖损坏".into());
            }
            return Ok(zone.id.clone());
        }
        if self.players[source.controller].graveyard.iter().any(|c|
            self.batch3_physical_cards().filter(|live|live.id==c.id).count()!=1) {
            return Err("总部迁移前实体已重复，不能用新实例掩盖原损坏".into());
        }
        let mut next=self.clone();
        let zone=Headquarters::first_entry(&mut next,source,host)?;
        let id=zone.id.clone();next.bq095_headquarters.push(zone);
        next.validate_bq095_state()?;
        *self=next;Ok(id)
    }

    /// Resolve owned printed "own graveyard" independently of generic/enemy
    /// graveyard access. The supplied source is the original frozen card.
    pub(crate) fn bq095_own_reference(&self, source:&Card) -> Destination {
        self.bq095_headquarters.iter().find(|z|z.seat==source.owner && (crate::catalog::catalog().cards.iter().any(|d|d.id==source.definition&&d.kind!="region")||crate::catalog::catalog().societies.iter().any(|d|d.card.id==source.definition)))
            .map_or(Destination::Graveyard(source.controller),|z|z.own_printed_reference(source))
    }
    pub(crate) fn bq095_destination(&self,seat:usize)->Destination {
        self.bq095_headquarters.iter().find(|z|z.seat==seat)
            .map_or(Destination::Graveyard(seat),|z|Destination::Headquarters(z.id.clone()))
    }
    pub(crate) fn bq095_reference_seat(&self,source:&Card)->usize {
        match self.bq095_own_reference(source) {Destination::Graveyard(seat)=>seat,Destination::Headquarters(id)=>self.bq095_headquarters.iter().find(|z|z.id==id).unwrap().seat}
    }
    pub(crate) fn bq095_reference_cards(&self,source:&Card)->&[Card] {
        match self.bq095_own_reference(source) {
            Destination::Headquarters(id)=>&self.bq095_headquarters.iter().find(|z|z.id==id).unwrap().cards,
            Destination::Graveyard(seat)=>&self.players[seat].graveyard,
        }
    }
    pub(crate) fn bq095_commit_destination(&mut self,card:Card) {
        if let Some(z)=self.bq095_headquarters.iter_mut().find(|z|z.seat==card.owner) {
            z.cards.push(card);
        } else {self.players[card.owner].graveyard.push(card);}
    }
    pub(crate) fn bq095_views(&self,viewer:usize)->Vec<HeadquartersView> {
        self.bq095_headquarters.iter().map(|z|HeadquartersView {
            id:z.id.clone(),player_id:crate::model::player_id(z.seat),name:"集团总部".into(),
            cards:z.cards.iter().map(|c|self.card_view(c,viewer,None,None)).collect(),
        }).collect()
    }
    pub(crate) fn validate_bq095_state(&self)->RuleResult<()> {
        let mut seats=BTreeSet::new();let mut zones=BTreeSet::new();
        for z in &self.bq095_headquarters {
            z.validate()?;
            if z.seat>=self.players.len() || !self.players[z.seat].graveyard.is_empty() || !seats.insert(z.seat) || !zones.insert(&z.id)
                || !self.mystic_instance_number(&z.source_instance).zip(self.mystic_instance_number(&z.id)).is_some_and(|(s,i)|s<i)
                || !self.mystic_instance_valid(&z.id)
                || self.batch3_physical_cards().any(|c|c.id==z.id) {
                return Err("总部区域身份、创建来源或唯一归属损坏".into());
            }
            for c in &z.cards {
                if !self.mystic_instance_valid(&c.id) || c.controller!=z.seat || c.face_down || c.exhausted
                    || !crate::catalog::catalog().cards.iter().any(|d|d.id==c.definition)
                    || c.named_character.is_some() || c.chosen_number.is_some() || c.chosen_subtype.is_some() || c.advancement_markers!=0 || c.blood_origin.is_some()
                    || c.generated.is_some() || c.damage!=0 || c.wounds!=0 || c.shield!=0
                    || c.lock_markers!=0 || c.time_markers!=0 || c.charge_markers!=0
                    || c.permanent_combat_markers!=0 || c.permanent_investigation_markers!=0 || c.permanent_defense_markers!=0
                    || self.batch3_physical_cards().filter(|v|v.id==c.id).count()!=1 {
                    return Err("总部实体重复、生成物或跨区状态未清理".into());
                }
                self.mobility_immutable_identity(c)?;
            }
        }
        self.validate_bq095_carriers()?;
        Ok(())
    }
}

use crate::rules::*;
fn same<T:Serialize>(a:&T,b:&T)->bool {serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition {
    let host=target(Zone::Society,EntityKind::Any,Relation::ControlledByActor,Range::Anywhere);
    let mut attach=ability("attach","结附于本方秘社",Timing::Standard,vec![],vec![host.clone()],vec![],None);attach.play_only=true;
    let enter=ability("bq095-entry","创建集团总部",Timing::Fast,vec![],vec![],vec![Op::BQ095Entry],Some(Event::Enter));
    let prepare=ability("bq095-prepare","准备：抓两张牌然后弃一张",Timing::Fast,vec![],vec![],vec![Op::BQ095Prepare{event_id:None}],Some(Event::Prepare));
    let mut d=with_abilities(vec![attach,enter,prepare]);d.attachment=Some(AttachmentSpec{controls_host:false,host_subtype_change:SubtypeChange::None,host,host_icons:crate::model::Icons::default(),host_temporary_icons:None,host_barrier:false,host_defense_bonus:0,host_leaves:HostLeaveDestination::OwnerGraveyard});d
}
pub(crate) fn related(a:&AbilitySpec)->bool {a.key.starts_with("bq095-")||a.ops.iter().any(|op|matches!(op,Op::BQ095Entry|Op::BQ095Prepare{..}))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {
    if id!=ID{return Err("总部规则不可移植".into());}
    if a.key=="bq095-prepare" {return crate::jc035::validate_ability(id,a);}
    if !definition().abilities.iter().any(|v|same(v,a)){return Err("总部固定能力损坏".into());}Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()> {
    if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("总部完整卡文定义损坏".into());}Ok(())
}
impl Game {
    pub(crate) fn bq095_entry(&mut self,f:&crate::model::ResolutionFrame)->RuleResult<()> {
        self.validate_bq095_entry_frame(f)?;
        let s=&f.source;
        if s.card.definition!=ID||s.card.face_down||f.actor!=s.card.controller||s.play_source.is_some()
            ||f.ability_key!="bq095-entry"||f.cursor!=1||!matches!(f.guard,crate::model::GuardState::Accepted)
            ||!f.targets.is_empty()||!f.already_paid.is_empty()||!f.resource_payments.is_empty()
            ||!same(&f.steps,&vec![crate::model::Step{context:f.actor,op:Op::BQ095Entry}]) {
            return Err("总部进场须沿原触发帧".into());
        }
        if self.players[f.actor].eliminated{return Ok(());}
        let host=s.attachment_host_instance.as_deref().ok_or("总部进场缺原秘社宿主")?;
        self.bq095_create_or_reuse(&s.card,host)?;Ok(())
    }
}

impl Game {
    pub(crate) fn validate_bq095_play(&self,item:&StackItem)->RuleResult<()> {
        if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(|f|f.source.card.definition==ID&&f.ability_key=="attach"){return Ok(());}
        let f=item.frame.as_ref().ok_or("BQ095结附事务不能缺原帧")?;let s=&f.source;let c=&s.card;
        self.validate_resource_frame(f)?;
        let [t]=f.targets.as_slice()else{return Err("BQ095须有唯一原本方秘社目标".into());};
        if c.definition!=ID||f.actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=f.actor||c.face_down||s.region.is_some()||s.source_region_instance.is_some()||s.attachment_host_instance.is_some()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()
            ||f.ability_key!="attach"||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||f.chosen_region.is_some()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
            ||t.id!=format!("society:p{}",f.actor)||!same(&t.spec,&definition().abilities[0].targets[0])||t.public.instance_id!=t.id||t.public.kind!="society"||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()
            ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=6&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()&&ids.iter().all(|id|self.mystic_instance_valid(id)))
            ||item.id!=f.frame_id||item.controller!=f.actor||item.target.as_deref()!=Some(t.id.as_str())||item.reveal||item.deploy_region.is_some()||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("BQ095结附实际事务、原宿主或程序损坏".into());}
        self.mobility_immutable_identity(c)
    }
}

impl Game {
    fn bq095_entry_source(&self,s:&SourceSnapshot,actor:usize)->RuleResult<()> {
        if actor>=self.players.len()||s.card.owner>=self.players.len()||s.card.controller!=actor||s.card.definition!=ID||s.card.face_down
            ||!self.mystic_instance_valid(&s.card.id)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()
            ||s.attachment_host_instance.as_deref()!=Some(format!("society:p{actor}").as_str())
            ||s.observed_death.is_some()||s.death_echo.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some() {
            return Err("总部触发原来源或秘社宿主损坏".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    fn validate_bq095_entry_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        if f.ability_key!="bq095-entry"&&!f.steps.iter().any(|s|matches!(s.op,Op::BQ095Entry)){return Ok(());}
        self.bq095_entry_source(&f.source,f.actor)?;
        if f.ability_key!="bq095-entry"||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)
            ||!f.targets.is_empty()||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()
            ||!self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(s,f)|s<f)
            ||!same(&f.steps,&vec![Step{context:f.actor,op:Op::BQ095Entry}]) {return Err("总部原进场帧损坏".into());}Ok(())
    }
    fn bq095_entry_declaration(&self,d:&Declaration)->RuleResult<()> {
        if d.ability.key!="bq095-entry"&&!d.ability.ops.iter().any(|o|matches!(o,Op::BQ095Entry)){return Ok(());}
        validate_ability(&d.source.card.definition,&d.ability)?;self.bq095_entry_source(&d.source,d.actor)
    }
    fn validate_bq095_carriers(&self)->RuleResult<()> {
        for item in &self.stack {self.validate_bq095_play(item)?;if let Some(f)=&item.frame {self.validate_bq095_entry_frame(f)?;
            if f.ability_key=="bq095-entry"&&(item.id!=f.frame_id||item.controller!=f.actor||item.card.is_some()||item.reveal||item.target.is_some()||item.deploy_region.is_some()||f.cursor!=0){return Err("总部响应外壳损坏".into());}}}
        for e in &self.effects {match e {Effect::Declare{declaration:d}=>self.bq095_entry_declaration(d)?,Effect::Frame{frame:f}=>self.validate_bq095_entry_frame(f)?,_=>()}}
        if let Some(p)=&self.pending {match &p.resolution {ChoiceResolution::Declare{declaration:d,..}=>self.bq095_entry_declaration(d)?,ChoiceResolution::Frame{frame:f,..}=>{self.validate_bq095_entry_frame(f)?;if f.ability_key=="bq095-entry"{return Err("总部单步进场不能伪造执行待选".into());}},_=>()}}
        Ok(())
    }
}

impl Game {
    pub(crate) fn bq095_take_reference(&mut self,source:&Card,id:&str)->RuleResult<Card> {
        match self.bq095_own_reference(source) {
            Destination::Graveyard(seat)=>{let i=self.players[seat].graveyard.iter().position(|c|c.id==id).ok_or("原本方墓地实体已离开")?;Ok(self.take_grave_observed(seat,i))},
            Destination::Headquarters(zone)=>{
                let z=self.bq095_headquarters.iter_mut().find(|z|z.id==zone).ok_or("原总部已不存在")?;
                let i=z.cards.iter().position(|c|c.id==id).ok_or("原总部实体已离开")?;let c=z.cards.remove(i);let seat=z.seat;
                self.jz53_observe_departure(&c,seat,Some(zone));Ok(c)
            }
        }
    }
}
