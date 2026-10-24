//! WM060 observes personal, other face-up Criminal entries. Its draw/discard
//! chain resumes in the normal frame; Repress removes board influence markers.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const REVEAL: &str = "batch4-wm060-reveal";
pub(crate) const ENTRY: &str = "batch4-wm060-criminal";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriminalEntry { pub event_id:String, pub turn:u32, pub entered:SourceSnapshot }
pub(crate) fn definition()->Definition {
    let mut d=with_abilities(vec![
        ability(REVEAL,"现身：按其他本方罪犯放置本方势力",Timing::Fast,vec![],vec![],vec![Op::WM060RevealInfluence],Some(Event::Reveal)),
        ability(ENTRY,"其他本方罪犯进场：抓1，弃1，遏制2",Timing::Fast,vec![],vec![],vec![
            Op::WM060CriminalObservation{observation:None},
            Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top},
            Op::Discard{player:PlayerRef::Actor,count:Some(1),redraw:false,optional:false},Op::WM060RepressTwo,
        ],Some(Event::WM060CriminalEntered)),
    ]);d.modifiers.push(StaticModifier::WM060ConditionalPublic);d
}
pub(crate) fn canonical(a:&AbilitySpec)->Option<AbilitySpec> {
    let mut expected=definition().abilities.into_iter().find(|b|b.key==a.key)?;
    if a.key==ENTRY {if let Some(Op::WM060CriminalObservation{observation})=a.ops.first(){expected.ops[0]=Op::WM060CriminalObservation{observation:observation.clone()};}}
    Some(expected)
}
pub(crate) fn validate_action(a:&Action)->RuleResult<()> {
    let expected=match a.kind.as_str(){
        "deploy"|"conceal"=>Action{card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind)},
        "reveal"=>Action{card_id:a.card_id.clone(),..Action::new("reveal")},
        _=>return Err("多情达莉娅只有普通派遣、秘密派遣和现身操作".into()),
    };if *a!=expected{return Err("多情达莉娅操作不能附带其他载荷".into());}Ok(())
}
impl Game {
    pub(crate) fn wm060_observe_entry(&mut self,id:&str) {
        let Some((r,c))=self.board(id).filter(|(_,c)|!c.face_down&&catalog::card(&c.definition).kind=="character"&&self.current_subtypes(c).iter().any(|s|s=="罪犯"))else{return;};
        let sources=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,region)|region.cards.iter().map(move|c|(r,c)))
            .filter(|(_,s)|s.definition=="WM060"&&!s.face_down&&s.controller==c.controller&&s.id!=c.id&&!self.players[s.controller].eliminated)
            .map(|(r,s)|self.source_snapshot(s,Some(r))).collect::<Vec<_>>();
        if sources.is_empty(){return;}
        let mut entered=self.source_snapshot(c,Some(r));entered.source_region_instance=Some(self.regions[r].card.id.clone());
        let observation=CriminalEntry{event_id:self.id(),turn:self.turn,entered};
        for source in sources {let mut a=definition().abilities.remove(1);a.ops[0]=Op::WM060CriminalObservation{observation:Some(Box::new(observation.clone()))};
            self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});
        }
    }
    fn wm060_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()> {
        let c=&s.card;
        if c.definition!="WM060"||c.id.is_empty()||actor>=self.players.len()||c.controller!=actor||c.owner>=self.players.len()
            ||s.attachment_host_instance.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()
            ||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_ref().is_none_or(|id|id.is_empty()||*id==c.id)
            ||s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,reg)|reg.card.id==*id&&r!=other))
            ||s.play_source.is_none()&&self.batch3_physical_cards().any(|live|live.id==c.id)&&self.board(&c.id).is_none()
        {return Err("多情达莉娅须保留原玩家、角色、地区和来源身份".into());}self.mobility_immutable_identity(c)
    }
    fn wm060_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
        let Some(Op::WM060CriminalObservation{observation:Some(o)})=ops.first()else{return Err("达莉娅缺少实际其他罪犯进场观察".into());};
        let e=&o.entered;let c=&e.card;
        let print=catalog::catalog().cards.iter().find(|d|d.id==c.definition&&d.kind=="character"&&catalog::runtime_implemented(d)).ok_or("罪犯观察不是当前角色")?;
        if c.id.is_empty()||c.id==s.card.id||c.face_down||c.controller!=actor||c.owner>=self.players.len()||!print.subtypes.iter().any(|t|t=="罪犯")
            ||o.turn!=self.turn||o.event_id.strip_prefix('i').and_then(|v|v.parse::<u64>().ok()).is_none_or(|n|n==0||n>self.sequence)
            ||o.event_id==c.id||o.event_id==s.card.id||e.play_source.is_some()||e.attachment_host_instance.is_some()||e.observed_death.is_some()||e.observed_exposure.is_some()||e.observed_mobility_grant.is_some()
            ||e.region.is_none_or(|r|r>=self.regions.len())||e.source_region_instance.as_ref().is_none_or(|id|id.is_empty()||*id==c.id)
            ||e.region.zip(e.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,reg)|reg.card.id==*id&&r!=other))
            ||self.batch3_physical_cards().any(|live|live.id==c.id)&&self.board(&c.id).is_none()
        {return Err("达莉娅只观察个人其他明置罪犯的实际进场".into());}self.mobility_immutable_identity(c)
    }
    fn wm060_expected(&self,s:&SourceSnapshot,key:&str,ops:&[Op])->RuleResult<AbilitySpec> {
        if key=="renown"||key=="jc089-combat-glory" {let uid=s.source_region_instance.as_deref().ok_or("达莉娅奖励缺少原地区")?;
            return Ok(if key=="renown"{crate::renown::renown_ability(uid)}else{crate::rules::jc089_combat_glory_ability(uid)});
        }
        let mut a=definition().abilities.into_iter().find(|a|a.key==key).ok_or("达莉娅未知程序")?;
        if key==ENTRY {a.ops[0]=ops.first().cloned().ok_or("达莉娅观察程序缺失")?;}Ok(a)
    }
    pub(crate) fn validate_wm060_declaration(&self,d:&Declaration)->RuleResult<()> {
        self.wm060_source(d.actor,&d.source)?;
        let a=self.wm060_expected(&d.source,&d.ability.key,&d.ability.ops)?;
        if d.source.card.face_down||d.source.play_source.is_some()||!same(&a,&d.ability){return Err("达莉娅触发必须保留完整事件程序".into());}
        if a.key==ENTRY {self.wm060_observation(d.actor,&d.source,&a.ops)?;}Ok(())
    }
    pub(crate) fn validate_wm060_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        self.wm060_source(f.actor,&f.source)?;
        if f.frame_id.is_empty()||f.cursor>f.steps.len()||(f.cursor==0&&!matches!(f.guard,GuardState::Unchecked))||(f.cursor>0&&!matches!(f.guard,GuardState::Accepted))||!f.targets.is_empty(){return Err("达莉娅帧、游标、守卫或目标无效".into());}
        if matches!(f.ability_key.as_str(),"deploy"|"reveal") {
            if !f.steps.is_empty()||f.chosen_region!=f.source.region
                ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=5&&ids.iter().all(|id|!id.is_empty()&&*id!=f.source.card.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
                ||(f.ability_key=="deploy"&&(f.source.card.face_down||!f.source.valid_character_play()))
                ||(f.ability_key=="reveal"&&(!f.source.card.face_down||f.source.play_source.is_some())){return Err("达莉娅正常派遣或现身的支付、原地区无效".into());}
        } else {
            let ops=f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>();let a=self.wm060_expected(&f.source,&f.ability_key,&ops)?;
            if f.steps.iter().any(|s|s.context!=f.actor)||!same(&ops,&a.ops)||!f.already_paid.is_empty()||f.chosen_region.is_some(){return Err("达莉娅触发的完整程序或环境被改写".into());}
            self.validate_wm060_declaration(&Declaration{actor:f.actor,source:f.source.clone(),ability:a})?;
        }Ok(())
    }
    pub(crate) fn wm060_reveal_influence(&mut self,f:&ResolutionFrame) {
        if let Some(r)=self.original_region(&f.source){let n=self.region_characters(r).filter(|c|!c.face_down&&c.id!=f.source.card.id&&c.controller==f.actor&&catalog::card(&c.definition).kind=="character"&&self.current_subtypes(c).iter().any(|t|t=="罪犯")).count() as u32;
            let team=self.team(f.actor);self.regions[r].influence[team]=self.regions[r].influence[team].saturating_add(n);
        }
    }
    pub(crate) fn wm060_repress_start(&mut self,f:ResolutionFrame)->RuleResult<()> {
        self.validate_wm060_frame(&f)?;if self.players[f.actor].eliminated{return Ok(());}
        let regions=self.batch20_repress_regions(f.actor);let n=2usize.min(regions.iter().map(|(_,_,n)|*n as usize).sum());
        if n>0 {self.choice(f.actor,"influence",format!("遏制{n}：选择并移除本方势力标志"),self.batch20_repress_options(&regions,n),n,n,Some(n as u32),ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM060Repress{regions}});}Ok(())
    }
    pub(crate) fn wm060_repress_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,regions:&[(usize,String,u32)],selected:&[String])->RuleResult<()> {
        self.validate_wm060_frame(f)?;let n=2usize.min(regions.iter().map(|(_,_,n)|*n as usize).sum());let options=self.batch20_repress_options(regions,n);
        self.wm060_choice_payload(f,chooser,a)?;
        if f.ability_key!=ENTRY||f.cursor!=4||!matches!(f.guard,GuardState::Accepted)||!same(&regions.to_vec(),&self.batch20_repress_regions(f.actor))||n==0
            ||selected.len()!=n||selected.iter().collect::<BTreeSet<_>>().len()!=n||selected.iter().any(|id|!options.iter().any(|o|o.id==*id)){return Err("达莉娅遏制须选择原地区上的指定数量不同标志".into());}
        self.wm040_repress_complete(f.actor,regions,selected)
    }
    pub(crate) fn wm060_choice_payload(&self,f:&ResolutionFrame,chooser:usize,a:&Action)->RuleResult<()> {
        if f.source.card.definition!="WM060"{return Ok(());}
        self.validate_wm060_frame(f)?;
        if chooser!=f.actor||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("达莉娅待选只能由原玩家提交原选择".into());}Ok(())
    }
    pub(crate) fn validate_wm060_state(&self)->RuleResult<()> {
        let mut frames=BTreeSet::new();let mut entries=BTreeSet::new();
        let mut remember=|s:&SourceSnapshot,ops:&[Op]|->RuleResult<()>{if let Some(Op::WM060CriminalObservation{observation:Some(o)})=ops.first(){if !entries.insert((s.card.id.clone(),o.event_id.clone())){return Err("同次罪犯进场不能重复触发同一达莉娅".into());}}Ok(())};
        for item in &self.stack {if let Some(f)=&item.frame {if f.source.card.definition=="WM060"&&!self.validated_cross_card_frame(f)? {
            self.validate_wm060_frame(f)?;remember(&f.source,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;
            let play=matches!(f.ability_key.as_str(),"deploy"|"reveal");
            if !frames.insert(f.frame_id.clone())||item.id!=f.frame_id||item.controller!=f.actor||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||item.card.is_some()!=play||item.reveal!=(f.ability_key=="reveal")||item.deploy_region!=(if play{f.source.region}else{None})||item.target.is_some()
                ||item.card.as_ref().is_some_and(|c|c.definition!="WM060"||c.face_down||c.owner!=f.source.card.owner||c.controller!=f.actor){return Err("达莉娅响应外壳或实例重复".into());}
        }}}
        for e in &self.effects {match e {
            Effect::Declare{declaration:d}if d.source.card.definition=="WM060"&&!self.validated_cross_card_declaration(d)?=>{self.validate_wm060_declaration(d)?;remember(&d.source,&d.ability.ops)?;},
            Effect::Frame{frame:f}if f.source.card.definition=="WM060"&&!self.validated_cross_card_frame(f)?=>return Err("达莉娅不能直接保存跳过响应的续体".into()),_=>{},
        }}
        if let Some(p)=&self.pending {match &p.resolution {
            ChoiceResolution::Declare{declaration:d,stage}if d.source.card.definition=="WM060"&&!self.validated_cross_card_declaration(d)?=>{
                self.validate_wm060_declaration(d)?;remember(&d.source,&d.ability.ops)?;
                let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",catalog::card("WM060").name,d.ability.label),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options:vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}],preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
                if !matches!(stage,DeclareChoice::Accept)||p.seat!=d.actor||p.choice.id.is_empty()||!same(&p.choice,&expected){return Err("达莉娅声明待选被改写".into());}
            },
            ChoiceResolution::Frame{frame:f,choice}if f.source.card.definition=="WM060"&&!self.validated_cross_card_frame(f)?=>{
                self.validate_wm060_frame(f)?;remember(&f.source,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;
                let (kind,title,options,n,amount)=match choice {
                    FrameChoice::Discard{seat,redraw}if *seat==f.actor&&!*redraw&&f.cursor==3=> ("discard","弃 1 张手牌".into(),self.players[f.actor].hand.iter().map(|c|self.option(c,f.actor,None,None)).collect(),1,None),
                    FrameChoice::WM060Repress{regions}if f.cursor==4&&same(&regions.to_vec(),&self.batch20_repress_regions(f.actor))=>{let n=2usize.min(regions.iter().map(|(_,_,n)|*n as usize).sum());("influence",format!("遏制{n}：选择并移除本方势力标志"),self.batch20_repress_options(regions,n),n,Some(n as u32))},
                    _=>return Err("达莉娅只准入原弃牌或遏制阶段".into()),
                };
                let expected=Choice{id:p.choice.id.clone(),kind:kind.into(),title,description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(f.actor),options,preview_cards:vec![],min:Some(n),max:Some(n),amount,allow_decline:Some(false)};
                if f.ability_key!=ENTRY||!matches!(f.guard,GuardState::Accepted)||p.seat!=f.actor||p.choice.id.is_empty()||n==0||expected.options.is_empty()||!same(&p.choice,&expected){return Err("达莉娅弃牌/遏制的玩家、阶段或原选项被改写".into());}
            },
            ChoiceResolution::Frame{choice:FrameChoice::WM060Repress{..},..}=>return Err("达莉娅遏制不能移植来源".into()),_=>{},
        }}Ok(())
    }
}
