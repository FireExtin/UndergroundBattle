//! The base manual trigger rule lets each controller order simultaneous entry
//! triggers. Payloads remain in the ordinary effects queue for all established
//! validators; the temporary order choice stores references, not copied Ops.
use crate::{catalog,engine::RuleResult,model::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryOrderGroup{pub entry_instance:String,pub turn:u32,pub actor:usize,pub remaining:Vec<String>}
fn key(d:&Declaration)->String{
    if let [crate::rules::Op::WM058Tax{observation:Some(o)}]=d.ability.ops.as_slice(){return format!("{}:{}:{}:{}",d.source.card.id,o.event_id,o.target.card.id,d.ability.key);}
    if let [crate::rules::Op::CQ07SpellTargeted{observation:Some(o)}]=d.ability.ops.as_slice(){return format!("{}:{}:{}:{}",d.source.card.id,o.event_id,o.target.card.id,d.ability.key);}
    // One observer may contribute to distinct entries before the queue drains.
    // Qualify the existing reference with its actual observation, not just the
    // observer UID and printed ability shared by all of those entries.
    let event=match d.ability.ops.as_slice(){
        [crate::rules::Op::BQ031AttachmentDiscard{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ038RaidFeed{observation:Some(o)}]=>Some(o.raid.frame_id.as_str()),
        [crate::rules::Op::WM044Inspect{contest_id:Some(id)}] | [crate::rules::Op::MysticWin{contest_id:Some(id)},..] | [crate::rules::Op::BQ031WinFeed{contest_id:Some(id)}]=>Some(id.as_str()),
        [crate::rules::Op::BQ038ContestBonus{observation:Some(o)}]=>Some(o.contest_id.as_str()),
        [crate::rules::Op::XG18RenownPlay{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::WM041PublicPlay{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::XG17AdvancePlan{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::XG13EntryTime{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::IndustryMill{observation:Some(o)}] | [crate::rules::Op::IndustryInvestigation{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::CQ24TakeAttachment{observation:Some(o),..}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::WM060CriminalObservation{observation:Some(o)},..]=>Some(o.event_id.as_str()),
        [crate::rules::Op::JZ38RepeatDeath{observation:Some(o)}]=>Some(o.proof.event_id.as_str()),
        [crate::rules::Op::WM035HostDeathDraw{observation:Some(o)}] | [crate::rules::Op::BlackDeathRecovery{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::JZ34DeathInfluence{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ079CorpseIcons{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ041BloodDemand{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ030AttachmentDraw{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ037FeedPair{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::XQ26Wanted{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ029SelfFeed{observation:Some(o)}]=>Some(o.event_id.as_str()),
        [crate::rules::Op::BQ026Attach{observation:Some(o)}]=>Some(o.event_id.as_str()),
        _=>None,
    }.or_else(||d.source.observed_death.as_ref().map(|o|o.card.id.as_str()));
    if let Some(event)=event{format!("{}:{event}:{}",d.source.card.id,d.ability.key)}else{format!("{}:{}",d.source.card.id,d.ability.key)}
}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
impl Game{
    pub(crate) fn batch3_register_entry_order(&mut self,entry:&str,previous:usize){
        let mut tail=self.effects.split_off(previous).into_iter().collect::<Vec<_>>();
        let all_decl=tail.iter().all(|e|matches!(e,Effect::Declare{..}));
        let introduced=tail.iter().any(|e|matches!(e,Effect::Declare{declaration:d} if matches!(d.source.card.definition.as_str(),"WM044"|"WM022"|"WM032"|"BQ031"|"BQ032"|"BQ036"|"BQ038"|"XG18"|"WM058"|"XG17"|"BQ029"|"BQ037"|"XQ26"|"WM060"|"BQ041"|"JZ38"|"XG13"|"CQ24"|"BQ093")));
        if all_decl && introduced && tail.len()>1{
            tail.sort_by_key(|e|if let Effect::Declare{declaration:d}=e{(self.team(d.actor)!=self.first_team,d.actor)}else{unreachable!()});
            for actor in 0..self.players.len(){let remaining=tail.iter().filter_map(|e|if let Effect::Declare{declaration:d}=e{(d.actor==actor).then(||key(d))}else{None}).collect::<Vec<_>>();
                if remaining.len()>1{self.entry_order_groups.push(EntryOrderGroup{entry_instance:entry.into(),turn:self.turn,actor,remaining});}}
        }self.effects.extend(tail);
    }
    fn entry_order_options(&self,g:&EntryOrderGroup)->RuleResult<Vec<ChoiceOption>>{
        g.remaining.iter().map(|id|{let ds=self.effects.iter().filter_map(|e|if let Effect::Declare{declaration:d}=e{(key(d)==*id).then_some(d)}else{None}).collect::<Vec<_>>();
            if ds.len()!=1 || ds[0].actor!=g.actor{return Err("进场触发排序引用丢失或重复".into());}let d=ds[0];let printed=catalog::catalog().cards.iter().find(|c|c.id==d.source.card.definition).ok_or("进场排序来源不是登记牌")?;Ok(ChoiceOption{id:id.clone(),label:format!("{}：{}",printed.name,d.ability.label),card:None})}).collect()
    }
    pub(crate) fn batch3_entry_order_declare(&mut self,d:&Declaration)->RuleResult<bool>{
        let k=key(d);let Some(i)=self.entry_order_groups.iter().position(|g|g.actor==d.actor && g.remaining.contains(&k))else{return Ok(false);};
        if self.entry_order_groups[i].remaining.len()==1{self.entry_order_groups.remove(i);return Ok(false);}
        self.effects.push_front(Effect::Declare{declaration:d.clone()});let g=self.entry_order_groups[i].clone();let options=self.entry_order_options(&g)?;
        self.choice(d.actor,"batch3-entry-order","选择下一个进入堆叠的触发能力".into(),options,1,1,None,ChoiceResolution::Batch3EntryOrder{entry_instance:g.entry_instance});Ok(true)
    }
    pub(crate) fn batch3_choose_entry_order(&mut self,entry:&str,actor:usize,a:&Action,selected:&[String])->RuleResult<()> {
        let i=self.entry_order_groups.iter().position(|g|g.actor==actor && g.entry_instance==entry).ok_or("原进场触发排序已失效")?;
        if selected.len()!=1 || !self.entry_order_groups[i].remaining.contains(&selected[0]) || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("进场触发排序只接受一个原能力引用".into());}
        let pos=self.effects.iter().position(|e|matches!(e,Effect::Declare{declaration:d} if key(d)==selected[0] && d.actor==actor)).ok_or("选中的进场声明已失效")?;
        let Effect::Declare{declaration}=self.effects.remove(pos).unwrap()else{unreachable!()};self.entry_order_groups[i].remaining.retain(|id|id!=&selected[0]);if self.entry_order_groups[i].remaining.is_empty(){self.entry_order_groups.remove(i);}
        self.declare_trigger_ordered(declaration)
    }
    pub(crate) fn validate_entry_order_groups(&self)->RuleResult<()> {
        let mut groups=BTreeSet::new();let mut members=BTreeSet::new();
        for g in &self.entry_order_groups{if g.actor>=self.players.len() || self.players[g.actor].eliminated || g.turn!=self.turn || g.entry_instance.strip_prefix('i').and_then(|s|s.parse::<u64>().ok()).is_none_or(|n|n==0 || n>self.sequence) || !groups.insert((g.entry_instance.clone(),g.actor)) || g.remaining.is_empty() || g.remaining.iter().any(|id|id.is_empty() || !members.insert(id.clone())){return Err("进场触发排序的回合、玩家、原实例或能力引用不符".into());}self.entry_order_options(g)?;}
        if let Some(p)=&self.pending{if let ChoiceResolution::Batch3EntryOrder{entry_instance}=&p.resolution{
            let g=self.entry_order_groups.iter().find(|g|g.actor==p.seat && g.entry_instance==*entry_instance).ok_or("进场排序待选缺少原观察组")?;
            if g.remaining.len()<2 || p.choice.id.is_empty() || p.choice.player_id!=player_id(g.actor) || p.choice.kind!="batch3-entry-order" || p.choice.title!="选择下一个进入堆叠的触发能力" || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&self.entry_order_options(g)?){return Err("进场排序待选的操控者、原队列或选项不符".into());}
        }}Ok(())
    }
}
