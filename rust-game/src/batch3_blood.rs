//! Hegemony manual physical page 20 (printed 19): the drinking character's
//! current controller chooses blood from their graveyard or society seals.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VampireEntryObservation{pub event_id:String,pub turn:u32,pub entered:SourceSnapshot,pub subtypes:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BloodFeedCursor{pub recipients:Vec<String>,pub cursor:usize,pub completed:Vec<BloodFeedStep>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BloodFeedStep{pub recipient:String,pub attached:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BQ029ExtraPayment{pub play_instance:String,pub play_frame:String,pub assets:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BQ029EntryObservation{pub event_id:String,pub turn:u32,pub moon_at_entry:u32,pub extra:Option<BQ029ExtraPayment>}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn bq037_definition()->Definition{
    let mut a=ability("batch3-bq037-reveal","行动1：可令目标本方暗藏者的吸血鬼正面现身，每回合一次",Timing::Standard,vec![Cost::Assets(1)],vec![target(Zone::Board,EntityKind::Hidden,Relation::ControlledByActor,Range::Anywhere)],vec![Op::BQ037RevealVampire],None);a.activation_only=true;a.per_turn_limit=Some(1);
    with_abilities(vec![ability("batch3-bq037-feed","另一个本方吸血鬼进场：该角色和本来源分别饮血1",Timing::Fast,vec![],vec![],vec![Op::BQ037FeedPair{observation:None}],Some(Event::BQ037OtherVampireEntered)),a])
}
pub(crate) fn bq029_definition()->Definition{
    with_abilities(vec![ability("raid-1","袭击1",Timing::Fast,vec![],vec![target(Zone::Board,EntityKind::Character,Relation::Any,Range::SourceRegion)],vec![Op::DamageTarget{slot:0,amount:1}],Some(Event::Reveal)),ability("batch3-bq029-feed","进场：已付额外1或本方隐秘时，饮血1",Timing::Fast,vec![],vec![],vec![Op::BQ029SelfFeed{observation:None}],Some(Event::BQ029QualifiedEntry))])
}
impl Game{
    pub(crate) fn bq029_printed_payment(&self,f:&ResolutionFrame)->bool{
        let (printed,extra)=match f.already_paid.as_slice(){[PaidCost::Assets(ids)]=>(ids.as_slice(),None),[PaidCost::Assets(ids),PaidCost::BQ029ExtraAssets(extra)] if f.ability_key=="deploy" && extra.len()==1=>(ids.as_slice(),Some(extra.as_slice())),_=>return false};
        let all=printed.iter().chain(extra.into_iter().flatten()).map(String::as_str).collect::<Vec<_>>();printed.len()<=2 && all.iter().all(|id|!id.is_empty() && *id!=f.source.card.id.as_str()) && all.iter().collect::<BTreeSet<_>>().len()==all.len() && !self.players.iter().enumerate().any(|(seat,p)|seat!=f.actor && p.assets.iter().any(|a|all.contains(&a.id.as_str())))
    }
    pub(crate) fn validate_bq029_raid_pending(&self,p:&Pending,d:&Declaration,stage:&DeclareChoice)->RuleResult<()> {
        self.validate_batch3_declaration(d)?;let options=self.binding_options(d.actor,&d.source,&d.ability.targets[0]);
        if d.ability.key!="raid-1" || !matches!(stage,DeclareChoice::Target) || options.is_empty() || p.seat!=d.actor || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty() || p.choice.title!=format!("{}：{}",catalog::card("BQ029").name,d.ability.label) || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&options,&p.choice.options){return Err("赤鳞近卫袭击须保留原现身目标选择".into());}Ok(())
    }
    pub(crate) fn current_non_asset(&self,c:&Card,actor:usize)->bool{catalog::card(&c.definition).keywords.iter().any(|k|k=="非资产") || matches!(c.definition.as_str(),"BQ029"|"BQ031"|"BQ032") && self.players.get(actor).is_some_and(|p|p.society_zone.exposure>0)}
    pub(crate) fn bq029_pay_extra(&mut self,actor:usize)->RuleResult<PaidCost>{let ids=self.players[actor].assets.iter().filter(|a|!a.exhausted).take(1).map(|a|a.id.clone()).collect::<Vec<_>>();self.pay(actor,1)?;Ok(PaidCost::BQ029ExtraAssets(ids))}
    pub(crate) fn bq029_observe_entry(&mut self,id:&str,extra:Option<BQ029ExtraPayment>){
        let Some((r,c))=self.board(id).filter(|(_,c)|c.definition=="BQ029" && !c.face_down)else{return;};let actor=c.controller;let moon=self.players[actor].society_zone.secrecy;if extra.is_none() && moon==0{return;}
        let source=self.source_snapshot(c,Some(r));let observation=BQ029EntryObservation{event_id:self.id(),turn:self.turn,moon_at_entry:moon,extra};let mut a=bq029_definition().abilities.remove(1);a.ops=vec![Op::BQ029SelfFeed{observation:Some(observation)}];self.effects.push_back(Effect::Declare{declaration:Declaration{actor,source,ability:a}});
    }
    pub(crate) fn bq029_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let [Op::BQ029SelfFeed{observation:Some(o)}]=ops else{return Err("赤鳞近卫缺少真实条件进场观察".into());};
        let number=|id:&str|id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).filter(|n|*n>0 && *n<=self.sequence);
        if s.card.definition!="BQ029" || s.card.face_down || s.card.controller!=actor || o.turn!=self.turn || number(&o.event_id).is_none() || o.event_id==s.card.id || o.extra.is_none() && o.moon_at_entry==0 {return Err("赤鳞近卫进场条件或事件身份不符".into());}
        if let Some(p)=&o.extra{if p.assets.len()!=1 || number(&p.assets[0]).is_none() || number(&p.play_instance).is_none() || number(&p.play_frame).is_none() || number(&p.play_instance)>=number(&p.play_frame) || number(&p.play_frame)>=number(&s.card.id) || p.assets[0]==p.play_instance || p.assets[0]==s.card.id || self.batch3_physical_cards().any(|c|c.id==p.play_instance) || self.players.iter().enumerate().any(|(seat,player)|seat!=actor && player.assets.iter().any(|a|p.assets.contains(&a.id))){return Err("赤鳞近卫额外费用须保留原打出与独立支付1收据".into());}}
        Ok(())
    }
    pub(crate) fn bq029_feed_start(&mut self,f:ResolutionFrame)->RuleResult<()> {
        let [Step{op:Op::BQ029SelfFeed{observation:Some(o)},..}]=f.steps.as_slice()else{return Err("赤鳞近卫饮血程序不符".into());};
        if o.extra.is_none() && self.players[f.actor].society_zone.secrecy==0{return Ok(());}
        let recipients=self.blood_recipients(&f)?;self.blood_feed_start(f,BloodFeedCursor{recipients,cursor:0,completed:vec![]})
    }
    pub(crate) fn bq037_observe_entry(&mut self,id:&str){
        let Some((r,c))=self.board(id).filter(|(_,c)|!c.face_down && catalog::card(&c.definition).kind=="character")else{return;};
        let subtypes=self.current_subtypes(c);if !subtypes.iter().any(|s|s=="吸血鬼"){return;}
        let sources=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,region)|region.cards.iter().map(move|c|(r,c))).filter(|(_,s)|s.definition=="BQ037" && !s.face_down && s.controller==c.controller && s.id!=c.id && !self.players[s.controller].eliminated).map(|(r,s)|self.source_snapshot(s,Some(r))).collect::<Vec<_>>();if sources.is_empty(){return;}
        let entered=self.source_snapshot(c,Some(r));let observation=VampireEntryObservation{event_id:self.id(),turn:self.turn,entered,subtypes};
        for source in sources{let mut a=bq037_definition().abilities.remove(0);a.ops=vec![Op::BQ037FeedPair{observation:Some(observation.clone())}];self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});}
    }
    pub(crate) fn bq037_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let [Op::BQ037FeedPair{observation:Some(o)}]=ops else{return Err("穿刺公缺少实际吸血鬼进场观察".into());};let e=&o.entered;let c=&e.card;
        let Some(printed)=catalog::catalog().cards.iter().find(|d|d.id==c.definition && d.kind=="character")else{return Err("观察不是登记角色".into());};
        if s.card.definition!="BQ037" || c.id.is_empty() || c.id==s.card.id || c.face_down || c.controller!=actor || c.owner>=self.players.len() || e.region.is_none_or(|r|r>=self.regions.len()) || e.play_source.is_some() || e.observed_death.is_some() || e.observed_exposure.is_some() || e.observed_mobility_grant.is_some()
            || !o.subtypes.iter().any(|t|t=="吸血鬼") || !(printed.subtypes.iter().any(|t|t=="吸血鬼" || t=="人类") || c.definition=="JC030") || o.subtypes.iter().collect::<BTreeSet<_>>().len()!=o.subtypes.len() || o.subtypes.iter().any(|t|!printed.subtypes.contains(t) && !matches!(t.as_str(),"吸血鬼"|"奴仆"|"邪教徒"|"宿主"|"不死生物"|"骷髅"))
            || o.turn!=self.turn || o.event_id==c.id || o.event_id==s.card.id || o.event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n==0 || n>self.sequence){return Err("穿刺公观察的原操控者、当前类别或进场身份不符".into());}self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_blood_trigger_pending(&self,p:&Pending,d:&Declaration,stage:&DeclareChoice)->RuleResult<()> {
        self.validate_batch3_declaration(d)?;
        if !matches!((d.source.card.definition.as_str(),d.ability.key.as_str()),("BQ037","batch3-bq037-feed")|("BQ029","batch3-bq029-feed")) || !matches!(stage,DeclareChoice::Accept) || p.seat!=d.actor || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty() || p.choice.title!=format!("{}：{}",catalog::card(&d.source.card.definition).name,d.ability.label) || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}]){return Err("穿刺公饮血须保留原可选进场声明".into());}Ok(())
    }
    fn blood_recipients(&self,f:&ResolutionFrame)->RuleResult<Vec<String>>{
        match f.steps.as_slice(){[Step{op:Op::BQ037FeedPair{observation:Some(o)},..}]=>Ok(vec![o.entered.card.id.clone(),f.source.card.id.clone()]),[Step{op:Op::BQ029SelfFeed{..},..}]=>Ok(vec![f.source.card.id.clone()]),_=>Err("非准入饮血程序".into())}
    }
    pub(crate) fn blood_drinker(&self,id:&str)->Option<&Card>{self.board(id).map(|(_,c)|c).filter(|c|!c.face_down && catalog::card(&c.definition).kind=="character" && !self.players[c.controller].eliminated && self.current_subtypes(c).iter().any(|s|s=="吸血鬼"))}
    fn validate_blood_cursor(&self,f:&ResolutionFrame,state:&BloodFeedCursor)->RuleResult<()> {
        if state.recipients!=self.blood_recipients(f)? || state.cursor>state.recipients.len() || state.completed.len()!=state.cursor {return Err("饮血续体必须保留每个已处理收受者的收据".into());}
        let event_id=match f.steps.as_slice(){[Step{op:Op::BQ037FeedPair{observation:Some(o)},..}]=>&o.event_id,[Step{op:Op::BQ029SelfFeed{observation:Some(o)},..}]=>&o.event_id,_=>return Err("非准入饮血事件".into())};let event=event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).ok_or("饮血事件身份无效")?;
        let mut used=BTreeSet::new();
        for (i,done) in state.completed.iter().enumerate(){if done.recipient!=state.recipients[i]{return Err("饮血处理收据顺序不符".into());}
            if let Some(id)=&done.attached {if !used.insert(id) || id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n<=event || n>self.sequence) || !self.attachments.iter().any(|a|a.card.id==*id && a.host_id==done.recipient && catalog::card(&a.card.definition).name=="鲜血" && self.attachment_host_valid(a)){return Err("饮血收据缺少该次实际新结附鲜血".into());}}
            else if self.blood_drinker(&done.recipient).is_some_and(|c|!self.blood_options(c.controller).is_empty()){return Err("有可用鲜血的原收受者不能被续体跳过".into());}
        }Ok(())
    }
    pub(crate) fn blood_options(&self,seat:usize)->Vec<ChoiceOption>{
        let mut out=self.players[seat].graveyard.iter().filter(|c|c.owner==seat && catalog::card(&c.definition).name=="鲜血" && catalog::card(&c.definition).kind=="attachment").map(|c|self.option(c,seat,None,Some(&format!("{}（墓地）",catalog::card(&c.definition).name)))).collect::<Vec<_>>();
        out.extend(self.sealed_cards.iter().filter(|s|s.host_id==format!("society:p{seat}") && catalog::card(&s.card.definition).name=="鲜血" && catalog::card(&s.card.definition).kind=="attachment").map(|s|ChoiceOption{id:s.card.id.clone(),label:"鲜血（本方秘社封印）".into(),card:Some(self.sealed_card_view(&s.card,seat))}));out
    }
    pub(crate) fn blood_feed_start(&mut self,f:ResolutionFrame,mut state:BloodFeedCursor)->RuleResult<()> {
        self.validate_batch3_frame(&f)?;
        self.validate_blood_cursor(&f,&state)?;
        while state.cursor<state.recipients.len(){if let Some(c)=self.blood_drinker(&state.recipients[state.cursor]){let seat=c.controller;let title=format!("{}：饮血1",catalog::card(&c.definition).name);let options=self.blood_options(seat);if !options.is_empty(){self.choice(seat,"batch3-blood",title,options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BloodFeed{state}});return Ok(());}}state.completed.push(BloodFeedStep{recipient:state.recipients[state.cursor].clone(),attached:None});state.cursor+=1;}Ok(())
    }
    pub(crate) fn bq037_feed_start(&mut self,f:ResolutionFrame)->RuleResult<()> {let recipients=self.blood_recipients(&f)?;self.blood_feed_start(f,BloodFeedCursor{recipients,cursor:0,completed:vec![]})}
    pub(crate) fn validate_blood_pending(&self,p:&Pending,f:&ResolutionFrame,state:&BloodFeedCursor)->RuleResult<()> {
        self.validate_batch3_frame(f)?;self.validate_blood_cursor(f,state)?;let id=state.recipients.get(state.cursor).ok_or("饮血游标越界")?;let c=self.blood_drinker(id).ok_or("原饮血角色已失效")?;let options=self.blood_options(c.controller);
        if state.recipients!=self.blood_recipients(f)? || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || options.is_empty() || p.seat!=c.controller || p.choice.player_id!=player_id(c.controller) || p.choice.id.is_empty() || p.choice.kind!="batch3-blood" || p.choice.title!=format!("{}：饮血1",catalog::card(&c.definition).name) || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&options){return Err("饮血待选须由当前操控者使用本人墓地或秘社封印".into());}Ok(())
    }
    pub(crate) fn blood_feed_complete(&mut self,f:&ResolutionFrame,mut state:BloodFeedCursor,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_batch3_frame(f)?;self.validate_blood_cursor(f,&state)?;let host=state.recipients.get(state.cursor).ok_or("饮血续体越界")?.clone();let c=self.blood_drinker(&host).ok_or("饮血原角色已失效")?;
        if f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || state.recipients!=self.blood_recipients(f)? || chooser!=c.controller || selected.len()!=1 || !self.blood_options(chooser).iter().any(|o|o.id==selected[0]) || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("饮血确认来源、操控者或鲜血选择不符".into());}
        let (definition,id)=self.attach_selected_blood(chooser,&host,&selected[0])?;
        self.enter_triggers(chooser,&definition,&id,false);state.completed.push(BloodFeedStep{recipient:host,attached:Some(id)});state.cursor+=1;self.blood_feed_start(f.clone(),state)
    }
    pub(crate) fn attach_selected_blood(&mut self,chooser:usize,host:&str,selected:&str)->RuleResult<(String,String)> {
        if self.blood_drinker(host).is_none_or(|c|c.controller!=chooser) || !self.blood_options(chooser).iter().any(|o|o.id==selected) {return Err("实际饮血来源或收受者已失效".into());}
        let c=if let Some(i)=self.players[chooser].graveyard.iter().position(|c|c.id==selected){self.players[chooser].graveyard.remove(i)}else{let i=self.sealed_cards.iter().position(|s|s.card.id==selected && s.host_id==format!("society:p{chooser}")).ok_or("原秘社封印鲜血失效")?;self.sealed_cards.remove(i).card};
        let mut c=self.reset_zone_card(c);c.controller=chooser;let id=c.id.clone();let definition=c.definition.clone();self.attachments.push(Attachment{card:c,host_id:host.into()});Ok((definition,id))
    }
    fn bq037_front_vampire(&self,f:&ResolutionFrame)->bool{f.targets.first().and_then(|t|self.board(&t.id)).is_some_and(|(_,c)|c.face_down && c.controller==f.actor && catalog::card(&c.definition).kind=="character" && catalog::card(&c.definition).subtypes.iter().any(|s|s=="吸血鬼"))}
    fn bq037_reveal_options(&self,f:&ResolutionFrame)->Vec<ChoiceOption>{f.targets.first().and_then(|t|self.board(&t.id)).map(|(r,c)|vec![self.option(c,f.actor,Some(r),Some("令该吸血鬼现身"))]).unwrap_or_default()}
    pub(crate) fn bq037_reveal_start(&mut self,f:ResolutionFrame)->RuleResult<()> {if self.bq037_front_vampire(&f){let options=self.bq037_reveal_options(&f);self.choice(f.actor,"batch3-bq037-reveal","穿刺公：可以令目标吸血鬼现身".into(),options,0,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ037Reveal});}Ok(())}
    pub(crate) fn validate_bq037_reveal_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_batch3_frame(f)?;if f.ability_key!="batch3-bq037-reveal" || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || !self.bq037_front_vampire(f) || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty() || p.choice.kind!="batch3-bq037-reveal" || p.choice.title!="穿刺公：可以令目标吸血鬼现身" || p.choice.min!=Some(0) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&self.bq037_reveal_options(f)){return Err("穿刺公现身待选与原暗藏者绑定不符".into());}Ok(())
    }
    pub(crate) fn bq037_reveal_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
        self.validate_batch3_frame(f)?;if chooser!=f.actor || f.ability_key!="batch3-bq037-reveal" || f.cursor!=1 || selected.len()>1 || !self.bq037_front_vampire(f) || selected.first().is_some_and(|id|Some(id)!=f.targets.first().map(|t|&t.id)) || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("穿刺公现身确认含无关目标或费用".into());}if let Some(id)=selected.first(){self.world_reveal(chooser,id,false)?;}Ok(())
    }
}
