//! JZ38 repeats the dead Host's own registered Death ability. Host is a subtype;
//! WM035's separate observer is not a Death ability granted to that character.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const KEY: &str = "batch4-jz38-host-death";
const ECHO_TITLE: &str = "仪式雕像：为额外死亡触发选择目标";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchoProof {
    pub event_id: String,
    pub turn: u32,
    pub ritual: Attachment,
    pub region_instance: String,
    pub subtypes: Vec<String>,
    pub host_grants: Vec<Card>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDeath { pub dead: SourceSnapshot, pub proof: EchoProof }
pub(crate) fn definition() -> Definition {
    let mut d = crate::rules::attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    d.abilities.push(ability(KEY, "本方宿主死亡：令其死亡触发额外发动一次", Timing::Fast, vec![], vec![],
        vec![Op::JZ38RepeatDeath { observation: None }], Some(Event::JZ38HostDied)));
    d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|p| p.key == a.key)?;
    if a.key == KEY { if let [Op::JZ38RepeatDeath { observation }] = a.ops.as_slice() { expected.ops = vec![Op::JZ38RepeatDeath { observation: observation.clone() }]; } }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|k| k != "attach")
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") }) {
        return Err("仪式雕像只能结附于本方秘社".into());
    }
    Ok(())
}
fn death_ability(id: &str) -> RuleResult<Option<AbilitySpec>> {
    let abilities = crate::rules::definition(id).abilities.iter().filter(|a| a.event == Some(Event::Death)).cloned().collect::<Vec<_>>();
    if abilities.len() > 1 || abilities.iter().any(|a| !a.modes.is_empty() || a.targets.len() > 1 || !a.costs.is_empty()) {
        return Err("当前雕像有限程序须先补齐新增死亡能力的多项或付费处理".into());
    }
    Ok(abilities.into_iter().next())
}
impl Game {
    fn statue_instance(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }
    pub(crate) fn jz38_capture_death(&self, source: &SourceSnapshot) -> Vec<HostDeath> {
        let c = &source.card;
        if c.face_down || catalog::card(&c.definition).kind != "character" || self.players[c.controller].eliminated { return vec![]; }
        let subtypes = self.current_subtypes(c);
        if !subtypes.iter().any(|s| s == "宿主") { return vec![]; }
        let printed = catalog::card(&c.definition);
        let grants = if printed.subtypes.iter().any(|s| s == "宿主") { vec![] } else {
            self.regions.iter().flat_map(|r| &r.cards).filter(|g| g.definition == "WM035" && !g.face_down && g.controller == c.controller).cloned().collect()
        };
        let Some(r) = source.region else { return vec![]; };
        self.attachments.iter().filter(|a| a.card.definition == "JZ38" && !a.card.face_down && a.card.controller == c.controller && self.attachment_host_valid(a))
            .map(|ritual| HostDeath { dead: source.clone(), proof: EchoProof { event_id: String::new(), turn: self.turn,
                ritual: ritual.clone(), region_instance: self.regions[r].card.id.clone(), subtypes: subtypes.clone(), host_grants: grants.clone() } }).collect()
    }
    pub(crate) fn jz38_emit_death(&mut self, observations: Vec<HostDeath>) {
        if observations.is_empty() { return; }
        let event = self.id();
        for mut o in observations {
            o.proof.event_id = event.clone();
            let source = self.source_snapshot(&o.proof.ritual.card, None);
            // The source may have left in the same removal wave. Preserve its
            // captured personal Society host, without locating a replacement.
            let source = SourceSnapshot { attachment_host_instance: Some(o.proof.ritual.host_id.clone()), ..source };
            let mut a = definition().abilities.remove(1); a.ops = vec![Op::JZ38RepeatDeath { observation: Some(Box::new(o)) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    fn statue_proof(&self, actor: usize, dead: &SourceSnapshot, p: &EchoProof) -> RuleResult<()> {
        let c = &dead.card; let r = &p.ritual; let n = self.statue_instance(&p.event_id).ok_or("雕像死亡事件身份无效")?;
        let printed = catalog::catalog().cards.iter().find(|d| d.id == c.definition && d.kind == "character" && catalog::runtime_implemented(d)).ok_or("雕像死亡来源不是当前角色")?;
        let printed_host = printed.subtypes.iter().any(|t| t == "宿主");
        if actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len() || c.face_down || dead.death_echo.is_some()
            || dead.region.is_none_or(|i| i >= self.regions.len()) || dead.play_source.is_some() || dead.observed_death.is_some() || dead.observed_exposure.is_some() || dead.observed_mobility_grant.is_some()
            || p.turn != self.turn || !p.subtypes.iter().any(|s| s == "宿主") || p.subtypes.iter().collect::<BTreeSet<_>>().len() != p.subtypes.len()
            || r.card.definition != "JZ38" || r.card.face_down || r.card.controller != actor || r.card.owner >= self.players.len() || r.host_id != format!("society:p{actor}")
            || [&c.id, &r.card.id, &p.region_instance].iter().any(|id| self.statue_instance(id).is_none_or(|v| v >= n))
            || [&c.id, &r.card.id, &p.region_instance].iter().collect::<BTreeSet<_>>().len() != 3
            || self.batch3_physical_cards().any(|live| live.id == c.id || live.id == p.event_id)
            || self.regions.iter().enumerate().any(|(i, region)| region.card.id == p.region_instance && Some(i) != dead.region)
            || (!printed_host && (!printed.subtypes.iter().any(|t| t == "人类") || !p.subtypes.iter().any(|t| t == "人类") || p.host_grants.is_empty()))
            || (printed_host && !p.host_grants.is_empty()) {
            return Err("雕像须保留本方真实宿主死亡、个人秘社来源与冻结类别".into());
        }
        self.mobility_immutable_identity(c)?; self.mobility_immutable_identity(&r.card)?;
        if self.batch3_physical_cards().any(|live| live.id == r.card.id) && !self.attachments.iter().any(|a| a.card.id == r.card.id) { return Err("雕像来源不能由私有区同UID替代".into()); }
        let mut grants = BTreeSet::new();
        for g in &p.host_grants {
            if g.definition != "WM035" || g.face_down || g.controller != actor || g.owner >= self.players.len()
                || self.statue_instance(&g.id).is_none_or(|v| v >= n) || !grants.insert(&g.id) || g.id == c.id || g.id == r.card.id
                || self.batch3_physical_cards().any(|live| live.id == g.id) && self.board(&g.id).is_none() { return Err("额外宿主类别必须保留实际本方布洛克来源".into()); }
            self.mobility_immutable_identity(g)?;
        }
        Ok(())
    }
    pub(crate) fn validate_statue_echo_declaration(&self, d: &Declaration) -> RuleResult<()> {
        let p = d.source.death_echo.as_deref().ok_or("额外死亡触发缺少雕像来源")?;
        let mut dead = d.source.clone(); dead.death_echo = None; self.statue_proof(d.actor, &dead, p)?;
        let a = death_ability(&dead.card.definition)?.ok_or("死者没有自身死亡触发能力")?;
        if !same(&a, &d.ability) { return Err("雕像额外发动必须保留死者完整印刷死亡能力".into()); } Ok(())
    }
    pub(crate) fn validate_statue_echo_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let a = death_ability(&f.source.card.definition)?.ok_or("额外死亡帧不是已登记死亡能力")?;
        self.validate_statue_echo_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        let steps = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
        let p = f.source.death_echo.as_ref().unwrap();
        if self.statue_instance(&f.frame_id).is_none_or(|n| self.statue_instance(&p.event_id).is_none_or(|e| n <= e))
            || f.ability_key != a.key || !same(&f.steps, &steps) || !f.already_paid.is_empty() || f.chosen_region.is_some()
            || f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t,s)| !same(&t.spec,s) || t.id.is_empty() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some())
            || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)) || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)) {
            return Err("雕像额外死亡帧不能改写完整程序、目标、费用、环境或游标".into());
        }
        Ok(())
    }
    fn statue_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "JZ38" || self.statue_instance(&c.id).is_none() || c.face_down || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len()
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some() || s.death_echo.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() { return Err("仪式雕像原个人秘社来源无效".into()); }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_statue_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.statue_source(d.actor,&d.source)?;
        if d.ability.key != KEY || canonical(&d.ability).is_none_or(|a| !same(&a,&d.ability)) { return Err("雕像死亡观察程序不完整".into()); }
        let [Op::JZ38RepeatDeath { observation: Some(o) }] = d.ability.ops.as_slice() else { return Err("雕像缺少宿主死亡事实".into()); };
        self.statue_proof(d.actor,&o.dead,&o.proof)?;
        if !same(&d.source.card,&o.proof.ritual.card) || d.source.attachment_host_instance.as_ref() != Some(&o.proof.ritual.host_id) { return Err("雕像死亡触发不能换用其他雕像或秘社".into()); } Ok(())
    }
    pub(crate) fn validate_statue_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.statue_source(f.actor,&f.source)?;
        if f.frame_id.is_empty() || f.frame_id == f.source.card.id || f.cursor > f.steps.len() || (f.cursor > 0 && !matches!(f.guard,GuardState::Accepted)) { return Err("雕像帧身份或游标无效".into()); }
        if f.ability_key == "attach" {
            let host = format!("society:p{}",f.actor); let t=f.targets.first().ok_or("雕像缺少个人秘社目标")?;
            if !f.steps.is_empty() || f.targets.len()!=1 || !same(&t.spec,&definition().abilities[0].targets[0]) || t.id!=host || t.region_instance.is_some()
                || t.public.instance_id!=host || t.public.kind!="society" || t.public.region.is_some() || !t.public.valid || t.public.status!="valid" || t.public.invalid_reason.is_some()
                || t.public.controller.as_deref()!=Some(&player_id(f.actor)) || t.public.owner.as_deref()!=Some(&player_id(f.actor)) || t.public.label!=format!("{}的秘社",self.players[f.actor].name)
                || f.source.attachment_host_instance.is_some() || f.chosen_region.is_some() || self.batch3_physical_cards().any(|c|c.id==f.source.card.id)
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if ids.len()<=2&&ids.iter().all(|id|!id.is_empty()&&*id!=f.source.card.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()) { return Err("雕像结附只能保留原个人秘社和已支付费用".into()); }
        } else {
            let mut a=definition().abilities.remove(1);a.ops=f.steps.iter().map(|s|s.op.clone()).collect();self.validate_statue_declaration(&Declaration{actor:f.actor,source:f.source.clone(),ability:a})?;
            if f.ability_key!=KEY||!f.targets.is_empty()||!f.already_paid.is_empty()||f.chosen_region.is_some()||f.steps.iter().any(|s|s.context!=f.actor)
                ||(f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)){return Err("雕像观察必须保留完整单步程序".into());}
        }
        Ok(())
    }
    fn statue_echo_options(&self, d: &Declaration) -> Vec<ChoiceOption> {
        let Some(t)=d.ability.targets.first()else{return vec![];};
        if t.range==Range::SourceRegion && !d.source.region.is_some_and(|r|d.source.death_echo.as_ref().is_some_and(|p|self.regions[r].card.id==p.region_instance&&!self.regions[r].vacant)){return vec![];}
        self.binding_options(d.actor,&d.source,t)
    }
    pub(crate) fn statue_repeat_death(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_statue_frame(f)?;
        let [Step { op:Op::JZ38RepeatDeath{observation:Some(o)},.. }]=f.steps.as_slice()else{return Err("雕像结算缺少死亡观察".into());};
        if f.cursor!=1||!matches!(f.guard,GuardState::Accepted){return Err("雕像只能执行一次完整死亡触发".into());}
        let Some(a)=death_ability(&o.dead.card.definition)?else{return Ok(());};
        let mut source=o.dead.clone();source.death_echo=Some(Box::new(o.proof.clone()));
        let d=Declaration{actor:f.actor,source,ability:a};self.validate_statue_echo_declaration(&d)?;
        if d.ability.targets.is_empty(){return self.choose_declaration(d,DeclareChoice::Accept,vec!["accept".into()]);}
        let options=self.statue_echo_options(&d);
        if !options.is_empty(){self.choice(f.actor,"trigger",ECHO_TITLE.into(),options,1,1,None,ChoiceResolution::JZ38EchoTarget{declaration:Box::new(d)});}
        Ok(())
    }
    pub(crate) fn statue_echo_target(&mut self,d:Declaration,actor:usize,a:&Action,selected:Vec<String>)->RuleResult<()> {
        self.validate_statue_echo_declaration(&d)?;
        if actor!=d.actor||selected.len()!=1||!self.statue_echo_options(&d).iter().any(|o|o.id==selected[0])||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}) {return Err("额外死亡触发须由原操控者选择一个真实新目标".into());}
        self.choose_declaration(d,DeclareChoice::Target,selected)
    }
    pub(crate) fn statue_choice_payload(&self,f:&ResolutionFrame,choice:&FrameChoice,actor:usize,a:&Action)->RuleResult<()> {
        if f.source.death_echo.is_none(){return Ok(());}
        self.validate_statue_echo_frame(f)?;
        let expected=if matches!(choice,FrameChoice::Forecast{..}) {Action{choice_id:a.choice_id.clone(),top:a.top.clone(),bottom:a.bottom.clone(),..Action::new("choose")}}
            else{Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}};
        let correct_actor=match choice{FrameChoice::Discard{seat,..}|FrameChoice::Forecast{seat}|FrameChoice::Search{seat,..}=>*seat,_=>f.actor};
        if actor!=correct_actor||*a!=expected{return Err("额外死亡续体只能由原待选玩家提交原字段".into());}Ok(())
    }
    fn statue_echo_player(&self,f:&ResolutionFrame,player:PlayerRef)->RuleResult<usize>{
        match player{PlayerRef::Actor|PlayerRef::Context=>Ok(f.actor),PlayerRef::Target(i)=>f.targets.get(i).and_then(|t|t.id.strip_prefix('p')).and_then(|s|s.parse::<usize>().ok()).filter(|s|*s<self.players.len()).ok_or("额外死亡待选玩家无效".into())}
    }
    fn statue_echo_pending(&self,p:&Pending,f:&ResolutionFrame,choice:&FrameChoice)->RuleResult<()> {
        let step=&f.steps[f.cursor-1].op;
        let (seat,kind,title,options,min,max,amount)=match(step,choice){
            (Op::Search{player,filter,to_top,optional,visibility},FrameChoice::Search{seat,to_top:actual,visibility:shown})if *actual==*to_top&&same(shown,visibility)&&*seat==self.statue_echo_player(f,*player)?=>{
                let options=self.players[*seat].deck.iter().filter(|c|self.filter_card(filter,c)).map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();
                (*seat,"search","检索并洗牌".into(),options,usize::from(!*optional),1,None)
            },
            (Op::Discard{player,count,redraw,optional},FrameChoice::Discard{seat,redraw:actual})if *actual==*redraw&&*seat==self.statue_echo_player(f,*player)?=>{
                let options=self.players[*seat].hand.iter().map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();let n=count.unwrap_or(options.len()).min(options.len());
                (*seat,"discard",if *redraw{"弃任意手牌并抓等量".into()}else{format!("弃 {n} 张手牌")},options,if *optional{0}else{n},n,None)
            },
            (Op::JZ30ForecastFrozenTime,FrameChoice::Forecast{seat})if *seat==f.actor=>{
                let options=self.players[*seat].deck.iter().take(f.source.card.time_markers as usize).map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();let n=options.len();
                (*seat,"investigation",format!("预测 {n}"),options,n,n,Some(n as u32))
            },
            (Op::Forecast{player,count},FrameChoice::Forecast{seat})if *seat==self.statue_echo_player(f,*player)?=>{
                let options=self.players[*seat].deck.iter().take(*count).map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();let n=options.len();
                (*seat,"investigation",format!("预测 {n}"),options,n,n,Some(n as u32))
            },
            (Op::BQ078ReturnNamelessCorpse,FrameChoice::BQ078CorpseReturn)=>{
                let options=self.players[f.actor].graveyard.iter().filter(|c|catalog::card(&c.definition).name=="无名尸体").map(|c|self.option(c,f.actor,None,None)).collect::<Vec<_>>();
                (f.actor,"bq078_corpse_return","将你墓地中的一张无名尸体置于你的手中".into(),options,1,1,None)
            },
            _=>return Err("额外死亡续体必须属于原程序的实际待选步骤".into()),
        };
        let expected=Choice{id:p.choice.id.clone(),kind:kind.into(),title,description:if kind=="investigation"{"将所有选项按所选顺序分到牌库顶和底。".into()}else{"选择符合数量限制的选项，然后确认。".into()},player_id:player_id(seat),options,preview_cards:vec![],min:Some(min),max:Some(max),amount,allow_decline:Some(min==0)};
        if p.seat!=seat||p.choice.id.is_empty()||expected.options.is_empty()||!same(&p.choice,&expected){return Err("额外死亡续体的原玩家、私有选项或数量被改写".into());}Ok(())
    }
    pub(crate) fn validate_statue_state(&self)->RuleResult<()> {
        let mut frames=BTreeSet::new();let mut observations=BTreeSet::new();let mut echoes=BTreeSet::new();
        let mut remember_observation=|s:&SourceSnapshot,ops:&[Op]|->RuleResult<()>{if let [Op::JZ38RepeatDeath{observation:Some(o)}]=ops{
            if !observations.insert((s.card.id.clone(),o.proof.event_id.clone())){return Err("同一雕像不能重复消费同次宿主死亡".into());}}
            Ok(())};
        let mut remember_echo=|s:&SourceSnapshot,key:&str|->RuleResult<()>{if let Some(p)=&s.death_echo{
            if !echoes.insert((s.card.id.clone(),p.ritual.card.id.clone(),p.event_id.clone(),key.to_string())){return Err("同一雕像不能重复发动同一宿主死亡能力".into());}}
            Ok(())};
        for item in &self.stack{if let Some(f)=&item.frame{
            if f.source.death_echo.is_some(){self.validate_statue_echo_frame(f)?;remember_echo(&f.source,&f.ability_key)?;
                if !frames.insert(f.frame_id.clone())||item.id!=f.frame_id||item.controller!=f.actor||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||item.card.is_some()||item.deploy_region.is_some()||item.reveal||item.target!=f.targets.first().map(|t|t.id.clone()){return Err("额外死亡响应帧外壳无效或重复".into());}
            }else if f.source.card.definition=="JZ38"{self.validate_statue_frame(f)?;remember_observation(&f.source,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;
                if !frames.insert(f.frame_id.clone())||item.id!=f.frame_id||item.controller!=f.actor||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||item.card.is_some()!=(f.ability_key=="attach")||item.deploy_region.is_some()||item.reveal||item.target!=f.targets.first().map(|t|t.id.clone())
                    ||item.card.as_ref().is_some_and(|c|c.definition!="JZ38"||c.face_down||c.owner!=f.source.card.owner||c.controller!=f.actor){return Err("雕像响应外壳或实例重复".into());}
            }
        }}
        for e in &self.effects{match e{
            Effect::Declare{declaration:d}if d.source.death_echo.is_some()=>return Err("额外死亡能力不能保存为新的可放弃声明".into()),
            Effect::Declare{declaration:d}if d.source.card.definition=="JZ38"=>{self.validate_statue_declaration(d)?;remember_observation(&d.source,&d.ability.ops)?;},
            Effect::Frame{frame:f}if f.source.death_echo.is_some()||f.source.card.definition=="JZ38"=>return Err("雕像不能保存跳过响应的续体".into()),_=>{},
        }}
        if let Some(p)=&self.pending{match &p.resolution{
            ChoiceResolution::JZ38EchoTarget{declaration:d}=>{
                self.validate_statue_echo_declaration(d)?;remember_echo(&d.source,&d.ability.key)?;
                let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:ECHO_TITLE.into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options:self.statue_echo_options(d),preview_cards:vec![],min:Some(1),max:Some(1),amount:None,allow_decline:Some(false)};
                if p.seat!=d.actor||p.choice.id.is_empty()||expected.options.is_empty()||!same(&p.choice,&expected){return Err("额外死亡触发的强制新目标待选无效".into());}
            },
            ChoiceResolution::Declare{declaration:d,..}if d.source.death_echo.is_some()=>return Err("额外死亡发动不能改写为可放弃原声明".into()),
            ChoiceResolution::Declare{declaration:d,stage}if d.source.card.definition=="JZ38"=>{
                self.validate_statue_declaration(d)?;remember_observation(&d.source,&d.ability.ops)?;
                let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",catalog::card("JZ38").name,d.ability.label),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options:vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}],preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
                if !matches!(stage,DeclareChoice::Accept)||p.seat!=d.actor||p.choice.id.is_empty()||!same(&p.choice,&expected){return Err("雕像触发待选被改写".into());}
            },
            ChoiceResolution::Frame{frame:f,choice}if f.source.death_echo.is_some()=>{
                self.validate_statue_echo_frame(f)?;remember_echo(&f.source,&f.ability_key)?;
                if !frames.insert(f.frame_id.clone())||f.cursor==0||!matches!(f.guard,GuardState::Accepted){return Err("额外死亡续体帧身份或阶段无效".into());}
                self.statue_echo_pending(p,f,choice)?;
                // Existing per-card and shared choice validators still inspect
                // the original program's complete private options and cursor.
            },
            ChoiceResolution::Frame{frame:f,..}if f.source.card.definition=="JZ38"=>return Err("雕像单步程序没有其他续体".into()),_=>{},
        }}Ok(())
    }
}
