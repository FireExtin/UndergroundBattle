//! Black import pair. Existing grave moves, rumor transfers, contest frames,
//! repression and private inspection remain the execution mechanisms.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::{ContestOutcome, MysticPhase}, rules::*};
use std::collections::BTreeSet;
pub(crate) const CARDS: &[&str] = &["BQ081", "WM057"];
pub(crate) const ENTRY: &str = "import20-bq081-enter";
pub(crate) const SHOWN: &str = "import20-bq081-shown";
pub(crate) const ACT: &str = "import20-wm057-contest";
pub(crate) const WIN: &str = "import20-wm057-win";
const GRAVE: &str = "bq081-grave-character";
const GRAVE_TITLE: &str = "凋零食腐怪：选择本人墓地角色置于牌库底";
const INSPECT: &str = "wm057-inspect-bound";
const INSPECT_TITLE: &str = "信息掮客：是否检视已绑定的原目标？";
const OPPONENT: &str = "wm057-repress-opponent";
const OPPONENT_TITLE: &str = "信息掮客：指定遏制的敌方玩家";
const CONFIRM: &str = "wm057-inspection-confirm";
const CONFIRM_TITLE: &str = "信息掮客：确认私密检视";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition(id: &str) -> Option<Definition> {
    Some(with_abilities(match id {
        "BQ081" => vec![ability(ENTRY,"进场：本人墓地角色置库底；如此做则消灭目标附属",Timing::Fast,
            vec![],vec![target(Zone::Board,EntityKind::Attachment,Relation::Any,Range::Anywhere)],
            vec![Op::BQ081GraveBottomDestroy],Some(Event::Enter))],
        "WM057" => {
            let mut a=ability(ACT,"行动2并横置：隐秘对抗",Timing::Standard,vec![Cost::Assets(2),Cost::ExhaustSource],
                vec![],vec![Op::MysticContest{actor_bonus:0}],None);a.activation_only=true;
            vec![a,win_program(None,None).unwrap()]
        },
        _=>return None,
    }))
}
pub(crate) fn owns(ops:&[Op])->bool { ops.iter().any(|op|match op {
    Op::BQ081GraveBottomDestroy|Op::WM057Win{..}|Op::WM057Repress|Op::WM057Inspect=>true,
    Op::ForEachLivingPlayer(body)|Op::ForEachLivingPlayerFromActor(body)=>owns(body),
    Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),
    _=>false,
}) }
pub(crate) fn related(a:&AbilitySpec)->bool {matches!(a.key.as_str(),ENTRY|ACT|WIN|SHOWN)||owns(&a.ops)||a.modes.iter().any(|m|owns(&m.ops))}
pub(crate) fn frame_related(f:&ResolutionFrame)->bool {CARDS.contains(&f.source.card.definition.as_str())
    ||matches!(f.ability_key.as_str(),ENTRY|ACT|WIN|SHOWN)||f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)))||f.already_paid.iter().any(|p|matches!(p,PaidCost::WM057Borrowed(_)))}
fn win_program(contest_id:Option<String>,mode:Option<&str>)->Option<AbilitySpec>{
    let proof=Op::WM057Win{contest_id};
    let mut a=ability(WIN,"赢得隐秘对抗：遏制1，然后可检视暗藏者或秘密计划",Timing::Fast,vec![],vec![],vec![proof.clone()],Some(Event::MysticContestWon));
    a.modes=vec![
        Mode{key:"repress".into(),label:"仅遏制".into(),targets:vec![],ops:vec![proof.clone(),Op::WM057Repress]},
        Mode{key:"inspect".into(),label:"遏制并选择可检视的目标".into(),targets:vec![inspection_target()],ops:vec![proof,Op::WM057Repress,Op::WM057Inspect]},
    ];
    if let Some(key)=mode{let m=a.modes.iter().find(|m|m.key==key)?.clone();a.label=format!("{}：{}",a.label,m.label);a.targets=m.targets;a.ops=m.ops;a.modes.clear();}
    Some(a)
}
fn canonical(id:&str,a:&AbilitySpec)->Option<AbilitySpec>{
    if id=="WM057"&&a.key==WIN{
        let Some(Op::WM057Win{contest_id})=a.ops.first()else{return None;};
        if !a.modes.is_empty(){return win_program(contest_id.clone(),None);}
        return ["repress","inspect"].iter().find_map(|key|win_program(contest_id.clone(),Some(key)).filter(|p|same(&p.ops,&a.ops)));
    }
    definition(id)?.abilities.into_iter().find(|b|b.key==a.key)
}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{
    if canonical(id,a).is_none_or(|b|!same(a,&b)){return Err("黑色新卡须保留完整原能力、费用及目标程序".into());}Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{
    if CARDS.contains(&id)&&!same(d,&definition(id).unwrap()){return Err("黑色新卡完整定义不可增删、改时序或目标".into());}Ok(())
}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{
    let expected=match a.kind.as_str(){
        "deploy"|"conceal"=>Action{card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind)},
        "reveal"=>Action{card_id:a.card_id.clone(),..Action::new("reveal")},
        "activate" if id=="WM057"&&a.ability_id.as_deref().is_none_or(|k|k==ACT)=>
            Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("activate")},
        _=>return Err("本卡没有该行动或手动触发入口".into()),
    };
    if *a!=expected{return Err("本卡不可夹带目标、选择或额外费用载荷".into());}Ok(())
}
pub(crate) fn validate_choice_action(p:&Pending,a:&Action)->RuleResult<()>{
    let related=match &p.resolution{
        ChoiceResolution::Declare{declaration:d,..}=>CARDS.contains(&d.source.card.definition.as_str())||related(&d.ability),
        ChoiceResolution::Frame{frame:f,..}=>frame_related(f),_=>false,
    };
    if related&&!matches!(p.resolution,ChoiceResolution::Frame{choice:FrameChoice::MysticCommit{..}|FrameChoice::MysticReward{..},..})
        &&*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){
        return Err("黑色新卡续体只接受当前选择".into());
    }Ok(())
}
pub(crate) fn inspection_target()->TargetSlotSpec{
    let mut t=target(Zone::Board,EntityKind::Any,Relation::Any,Range::Anywhere);
    t.predicate=Some(TargetPredicate::WM057HiddenOrSecretPlan);t
}
impl Game{
    pub(crate) fn bq081_grave_options(&self,actor:usize)->Vec<ChoiceOption>{
        self.players[actor].graveyard.iter().filter(|c|c.owner==actor&&!c.face_down&&catalog::card(&c.definition).kind=="character")
            .map(|c|self.option(c,actor,None,Some("graveyard"))).collect()
    }
    pub(crate) fn bq081_entry_start(&mut self,f:ResolutionFrame)->RuleResult<()>{
        self.validate_import20_broker_frame(&f)?;
        let options=self.bq081_grave_options(f.actor);if options.is_empty(){return Ok(());}
        self.choice(f.actor,GRAVE,GRAVE_TITLE.into(),options,1,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ081GraveCharacter});Ok(())
    }
    pub(crate) fn bq081_entry_complete(&mut self,f:&ResolutionFrame,actor:usize,selected:&[String])->RuleResult<()>{
        self.validate_import20_broker_frame(f)?;
        if actor!=f.actor||f.cursor!=1||selected.len()!=1
            ||!self.bq081_grave_options(actor).iter().any(|o|o.id==selected[0]){return Err("凋零食腐怪须选择本人墓地的一张角色".into());}
        let index=self.players[actor].graveyard.iter().position(|c|c.id==selected[0]).ok_or("原墓地角色已失效")?;
        let c=self.take_grave_observed(actor,index);let name=catalog::card(&c.definition).name.clone();let c=self.reset_zone_card(c);
        self.players[actor].deck.push(c);self.note(format!("{} 将墓地的 {} 置于牌库底",self.players[actor].name,name));
        self.remove_dead(&f.targets[0].id,RemovalCause::Destroy);Ok(())
    }
    pub(crate) fn wm057_observe_win(&mut self,o:&ContestOutcome){
        let Some(team)=o.winner_team else{return;};
        let sources=self.in_play_cards().into_iter().filter(|(r,c)|r.is_some()&&c.definition=="WM057"&&!c.face_down
            &&!self.players[c.controller].eliminated&&self.team(c.controller)==team&&o.participants.contains(&c.controller))
            .map(|(r,c)|self.source_snapshot(c,r)).collect::<Vec<_>>();
        for source in sources{let a=win_program(Some(o.contest_id.clone()),None).unwrap();
            self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});}
    }
    fn wm057_opponents(&self,actor:usize)->Vec<ChoiceOption>{
        self.players.iter().filter(|p|!p.eliminated&&self.team(p.seat)!=self.team(actor))
            .map(|p|ChoiceOption{id:player_id(p.seat),label:p.name.clone(),card:None}).collect()
    }
    fn wm057_repress_regions(&self,seat:usize)->Vec<(usize,String)>{
        self.regions.iter().enumerate().filter(|(_,r)|!r.vacant&&r.influence[self.team(seat)]>0).map(|(r,v)|(r,v.card.id.clone())).collect()
    }
    fn wm057_repress_options(&self,seat:usize,regions:&[(usize,String)])->Vec<ChoiceOption>{
        regions.iter().map(|(r,_)|ChoiceOption{id:format!("region:{r}"),label:format!("地区{}：{}（本方势力{}）",r+1,catalog::card(&self.regions[*r].card.definition).name,self.regions[*r].influence[self.team(seat)]),card:None}).collect()
    }
    pub(crate) fn wm057_repress_start(&mut self,f:ResolutionFrame)->RuleResult<()>{
        self.validate_import20_broker_frame(&f)?;let options=self.wm057_opponents(f.actor);if options.is_empty(){return Ok(());}
        self.choice(f.actor,OPPONENT,OPPONENT_TITLE.into(),options,1,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM057Opponent});Ok(())
    }
    pub(crate) fn wm057_repress_choose(&mut self,f:ResolutionFrame,actor:usize,selected:&[String])->RuleResult<()>{
        self.validate_import20_broker_frame(&f)?;
        if actor!=f.actor||f.cursor!=2||selected.len()!=1||!self.wm057_opponents(actor).iter().any(|o|o.id==selected[0]){return Err("信息掮客须指定一名敌方玩家遏制".into());}
        let seat=self.players.iter().find(|p|player_id(p.seat)==selected[0]).ok_or("遏制玩家不存在")?.seat;
        let regions=self.wm057_repress_regions(seat);if regions.is_empty(){return self.resolve_frame(f);}
        let options=self.wm057_repress_options(seat,&regions);
        self.choice(seat,"target","遏制1：选择移除本方势力的地区".into(),options,1,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::RepressOne{seat,regions}});Ok(())
    }
    fn wm057_peek_options(&self,f:&ResolutionFrame)->Vec<ChoiceOption>{
        if f.targets.first().is_some_and(|t|self.valid_bound_target(f.actor,&f.source,t)){
            vec![ChoiceOption{id:"inspect".into(),label:"检视已绑定的原目标".into(),card:None}]
        }else{vec![]}
    }
    pub(crate) fn wm057_inspect_start(&mut self,f:ResolutionFrame)->RuleResult<()>{
        self.validate_import20_broker_frame(&f)?;let options=self.wm057_peek_options(&f);if options.is_empty(){return Ok(());}
        self.choice(f.actor,INSPECT,INSPECT_TITLE.into(),options,0,1,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM057InspectMay});Ok(())
    }
    pub(crate) fn wm057_inspect_choose(&mut self,f:ResolutionFrame,actor:usize,selected:&[String])->RuleResult<()>{
        self.validate_import20_broker_frame(&f)?;
        if actor!=f.actor||f.cursor!=3||selected.len()>1||selected.first().is_some_and(|s|s!="inspect"){return Err("信息掮客仅本人可决定是否检视原目标".into());}
        if selected.is_empty(){return Ok(());}
        let t=f.targets.first().ok_or("检视缺少原目标")?;
        if !self.valid_bound_target(actor,&f.source,t){return Ok(());}
        let inspected=self.in_play(&t.id).map(|(_,c)|c.clone()).ok_or("原检视对象已离场")?;
        if !crate::batch4_plans::is_plan(&inspected){self.observe_hidden_inspection(actor,&t.id);}
        self.choice(actor,CONFIRM,CONFIRM_TITLE.into(),vec![],0,0,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM057Inspection{inspected}});
        self.pending.as_mut().unwrap().choice.allow_decline=Some(false);Ok(())
    }
    pub(crate) fn wm057_inspect_complete(&self,f:&ResolutionFrame,actor:usize,selected:&[String])->RuleResult<()>{
        self.validate_import20_broker_frame(f)?;
        if actor!=f.actor||f.cursor!=3||!selected.is_empty(){return Err("信息掮客只接受本人确认检视".into());}Ok(())
    }
    fn import20_broker_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{
        let c=&s.card;
        if !CARDS.contains(&c.definition.as_str())||actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=actor
            ||!self.mystic_instance_valid(&c.id)||s.region.is_none_or(|r|r>=self.regions.len())
            ||s.source_region_instance.as_ref().is_none_or(|id|!self.mystic_instance_valid(id)||*id==c.id)
            ||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()
            ||s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,live)|live.card.id==*id&&other!=r)){
            return Err("黑色新卡须保留原角色、操控者和原地区".into());
        }self.mobility_immutable_identity(c)
    }
    fn import20_broker_expected(&self,s:&SourceSnapshot,key:&str,ops:&[Op])->RuleResult<AbilitySpec>{
        if matches!(key,"renown"|"jc089-combat-glory"){
            let region=s.source_region_instance.as_deref().ok_or("奖励缺少原地区")?;
            return Ok(if key=="renown"{crate::renown::renown_ability(region)}else{crate::rules::jc089_combat_glory_ability(region)});
        }
        let mut a=definition(&s.card.definition).and_then(|d|d.abilities.into_iter().find(|a|a.key==key)).ok_or("黑色新卡未知程序")?;
        if key==WIN{
            let Some(Op::WM057Win{contest_id:Some(id)})=ops.first()else{return Err("信息掮客缺少真实隐秘获胜结果".into());};
            let o=self.mystic_history.iter().find(|o|o.contest_id==*id).ok_or("信息掮客获胜结果不存在")?;
            if o.turn!=self.turn||!o.participants.contains(&s.card.controller)||o.winner_team!=Some(self.team(s.card.controller)){return Err("信息掮客奖励属于原实际获胜玩家".into());}
            self.validate_mystic_outcome(o)?;
            let proof=Some(id.clone());
            a=if ops.len()==1{win_program(proof,None).unwrap()}else{
                ["repress","inspect"].iter().find_map(|key|win_program(proof.clone(),Some(key)).filter(|p|same(&p.ops,&ops.to_vec())))
                    .ok_or("信息掮客须保留已选的完整遏制/检视程序")?
            };
        }Ok(a)
    }
    pub(crate) fn validate_import20_broker_declaration(&self,d:&Declaration)->RuleResult<()>{
        if self.validated_cross_card_declaration(d)?{return Ok(());}
        self.import20_broker_source(d.actor,&d.source)?;let a=self.import20_broker_expected(&d.source,&d.ability.key,&d.ability.ops)?;
        if d.source.card.face_down||d.source.play_source.is_some()||a.event.is_none()||!same(&d.ability,&a){return Err("黑色新卡触发须保留原完整声明".into());}Ok(())
    }
    fn validate_bq081_post(&self,f:&ResolutionFrame)->RuleResult<()>{
        let Some(state)=self.mystic_states().find(|s|matches!(&s.phase,MysticPhase::After{post_frame:Some(p),..}if p.frame_id==f.frame_id))
            else{return Err("凋零展示后置缺少同次对抗".into());};
        let MysticPhase::After{outcome,next_shown,post_frame}=&state.phase else{unreachable!()};
        let shown=next_shown.checked_sub(1).and_then(|i|outcome.shown.get(i)).ok_or("凋零展示索引无效")?;
        if shown.card.definition!="BQ081"||shown.seat!=f.actor||!same(&f.source,&self.source_snapshot(&shown.card,None))
            ||f.cursor!=1||!matches!(f.guard,GuardState::Accepted)||!f.already_paid.is_empty()||!f.targets.is_empty()||f.chosen_region.is_some()
            ||!matches!(f.steps.as_slice(),[Step{context,op:Op::MoveDeckTopToGraveyard{player:PlayerRef::Actor,count:1}}]if *context==f.actor)
            ||post_frame.as_ref().is_none_or(|p|!same(p.as_ref(),f)){return Err("凋零展示后置只能按本人当次展示磨一张".into());}Ok(())
    }
    pub(crate) fn validate_import20_broker_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
        self.validate_resource_frame(f)?;
        if self.validated_cross_card_frame(f)?{return Ok(());}
        if f.ability_key==SHOWN{return self.validate_bq081_post(f);}
        if f.source.card.definition=="WM057" && f.ability_key==ACT && f.actor!=f.source.card.controller {
            self.import20_broker_source(f.source.card.controller,&f.source)?;
            let Some(PaidCost::WM057Borrowed(p))=f.already_paid.last() else {return Err("敌方借用缺少发动者本人资格凭据".into());};
            self.wm057_validate_borrow(f,p)?;
        } else {self.import20_broker_source(f.actor,&f.source)?;}
        if !self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(s,f)|s<f)
            ||f.cursor>f.steps.len()||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor>0&&!matches!(f.guard,GuardState::Accepted){return Err("黑色新卡帧身份或阶段无效".into());}
        if matches!(f.ability_key.as_str(),"deploy"|"reveal"){
            if !f.steps.is_empty()||!f.targets.is_empty()||f.chosen_region!=f.source.region
                ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=catalog::card(&f.source.card.definition).cost as usize
                    &&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=f.source.card.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
                ||f.ability_key=="deploy"&&(f.source.card.face_down||!f.source.valid_character_play())
                ||f.ability_key=="reveal"&&(!f.source.card.face_down||f.source.play_source.is_some()){return Err("黑色新卡派遣/现身须保留真实支付与原地区".into());}return Ok(());
        }
        let ops=f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>();let a=self.import20_broker_expected(&f.source,&f.ability_key,&ops)?;
        if !a.modes.is_empty()||f.source.card.face_down||f.source.play_source.is_some()||f.steps.iter().any(|s|s.context!=f.actor)||!same(&ops,&a.ops)
            ||f.chosen_region.is_some()||f.targets.len()!=a.targets.len()||f.targets.iter().zip(&a.targets).any(|(t,s)|!same(&t.spec,s)){
            return Err("黑色新卡须保留原完整程序与目标".into());
        }
        if f.ability_key==ACT{
            let costs=if f.actor!=f.source.card.controller {&f.already_paid[..f.already_paid.len()-1]}else{f.already_paid.as_slice()};
            if f.source.card.exhausted||!matches!(costs,[PaidCost::Assets(ids),PaidCost::Exhausted(id)]if id==&f.source.card.id
                &&f.money_units(0)==2&&ids.iter().all(|n|self.mystic_instance_valid(n)&&n!=id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()){return Err("信息掮客须保留实际付款2和原实例横置收据".into());}
        }else if !f.already_paid.is_empty(){return Err("本触发没有额外支付".into());}Ok(())
    }
    pub(crate) fn validate_import20_broker_state(&self)->RuleResult<()>{
        for item in &self.stack{if let Some(f)=&item.frame{if frame_related(f){self.validate_import20_broker_frame(f)?;}}}
        for effect in &self.effects{match effect{
            Effect::Declare{declaration:d}if CARDS.contains(&d.source.card.definition.as_str())||related(&d.ability)=>self.validate_import20_broker_declaration(d)?,
            Effect::Frame{frame:f}if frame_related(f)=>self.validate_import20_broker_frame(f)?,_=>{},
        }}
        if let Some(p)=&self.pending{match &p.resolution{
            ChoiceResolution::Declare{declaration:d,..}if CARDS.contains(&d.source.card.definition.as_str())||related(&d.ability)=>self.validate_import20_broker_declaration(d)?,
            ChoiceResolution::Frame{frame:f,choice}if frame_related(f)=>{
                self.validate_import20_broker_frame(f)?;
                let (kind,title,options,min,max)=match choice{
                    FrameChoice::BQ081GraveCharacter if f.ability_key==ENTRY&&f.cursor==1=>(GRAVE,GRAVE_TITLE,self.bq081_grave_options(f.actor),1,1),
                    FrameChoice::WM057Opponent if f.ability_key==WIN&&f.cursor==2=>(OPPONENT,OPPONENT_TITLE,self.wm057_opponents(f.actor),1,1),
                    FrameChoice::WM057InspectMay if f.ability_key==WIN&&f.cursor==3=>(INSPECT,INSPECT_TITLE,self.wm057_peek_options(f),0,1),
                    FrameChoice::WM057Inspection{inspected:c}if f.ability_key==WIN&&f.cursor==3=>{
                        if f.targets.first().is_none_or(|t|t.id!=c.id)||!c.face_down||self.in_play(&c.id).is_none_or(|(_,live)|live!=c)||!self.valid_binding(f.actor,&f.source,&inspection_target(),&c.id){return Err("信息掮客确认须保留原暗牌".into());}
                        (CONFIRM,CONFIRM_TITLE,vec![],0,0)
                    },
                    FrameChoice::RepressOne{seat,regions} if f.ability_key==WIN&&f.cursor==2=>{
                        if !self.wm057_opponents(f.actor).iter().any(|o|o.id==player_id(*seat))||p.seat!=*seat||regions.is_empty()
                            ||!same(regions,&self.wm057_repress_regions(*seat))||!same(&p.choice.options,&self.wm057_repress_options(*seat,regions))
                            ||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false){return Err("信息掮客遏制须由原敌方玩家移除势力".into());}return Ok(());
                    },
                    FrameChoice::MysticCommit{..}|FrameChoice::MysticReward{..}if f.ability_key==ACT=>return Ok(()),
                    _=>return Err("黑色新卡续体不属于当前执行阶段".into()),
                };
                if p.seat!=f.actor||p.choice.player_id!=player_id(f.actor)||p.choice.kind!=kind||p.choice.title!=title
                    ||p.choice.min!=Some(min)||p.choice.max!=Some(max)||p.choice.allow_decline!=Some(min==0&&max>0)
                    ||p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options){return Err("黑色新卡待选须保留本人、次数和原选项".into());}
            },_=>{},
        }}Ok(())
    }
}
