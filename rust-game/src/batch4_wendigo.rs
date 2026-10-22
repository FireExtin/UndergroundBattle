//! JZ51: private top-eight corpse search and a separate, paid graveyard action.
//! Grave entry is placement, never a discounted printed-cost play.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ENTRY: &str = "batch4-jz51-top-eight";
pub(crate) const GRAVE: &str = "batch4-jz51-grave-action";
const PAYMENT: &str = "jz51-seal-payment";
const SEARCH: &str = "jz51-top-eight";
const REGION: &str = "jz51-entry-region";
const PAYMENT_TITLE:&str="选择本人墓地三张无名尸体，封印在本方秘社区以支付";
const SEARCH_TITLE:&str="查看顶八张，选择任意数量无名尸体放置原地区";
const REGION_TITLE:&str="选择温迪戈放置进场的地区";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraveDraft { pub source: SourceSnapshot, pub turn: u32 }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpsePayment { pub turn: u32, pub host_id: String, pub corpses: Vec<Card> }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
fn corpse(c: &Card) -> bool { let d = catalog::card(&c.definition); d.kind == "character" && d.name == "无名尸体" }
fn related(f:&ResolutionFrame)->bool {f.source.card.definition=="JZ51" || f.already_paid.iter().any(|c|matches!(c,PaidCost::JZ51Corpses(_))) || f.steps.iter().any(|s|matches!(s.op,Op::JZ51TopEightCorpses|Op::JZ51GraveEntry))}
pub(crate) fn definition() -> Definition {
    let mut grave = ability(GRAVE, "墓地行动：封印本人墓地三张无名尸体，温迪戈放置进场", Timing::Standard,
        vec![Cost::JZ51SealThreeCorpses], vec![], vec![Op::JZ51GraveEntry], None);
    grave.activation_only = true;
    with_abilities(vec![ability(ENTRY, "进场：从顶八张寻找任意无名尸体放置本地区，其余洗回", Timing::Fast,
        vec![], vec![], vec![Op::JZ51TopEightCorpses], Some(Event::Enter)), grave])
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id:a.card_id.clone(), region:a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id:a.card_id.clone(), ..Action::new("reveal") },
        "activate" if a.ability_id.as_deref() == Some(GRAVE) => Action {card_id:a.card_id.clone(), ability_id:a.ability_id.clone(), ..Action::new("activate")},
        _ => return Err("温迪戈只有派遣、现身与固定墓地行动".into()),
    };
    if *a != expected { return Err("温迪戈不能追加费用、目标或数量载荷".into()); } Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(p.resolution, ChoiceResolution::JZ51Payment {..} | ChoiceResolution::Frame {choice:FrameChoice::JZ51TopEight {..} | FrameChoice::JZ51EntryRegion {..},..})
        && *a != (Action {choice_id:a.choice_id.clone(), selected:a.selected.clone(), ..Action::new("choose")}) {
        return Err("温迪戈待选只接受当前选项集合".into());
    } Ok(())
}
impl Game {
    fn jz51_source(&self, actor:usize, s:&SourceSnapshot) -> RuleResult<()> {
        let c=&s.card;
        if actor>=self.players.len() || c.definition!="JZ51" || c.controller!=actor || c.owner>=self.players.len()
            || c.face_down || c.id.is_empty() || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some() {
            return Err("温迪戈能力必须保留完整来源与个人操控者".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_jz51_declaration(&self, d:&Declaration)->RuleResult<()> {
        self.jz51_source(d.actor,&d.source)?;
        if !same(&d.ability,&definition().abilities[0]) || d.source.region.is_none_or(|r|r>=self.regions.len())
            || d.source.source_region_instance.as_ref().is_none_or(|s|s.is_empty()) {
            return Err("温迪戈只有原地区绑定的固定进场触发".into());
        } Ok(())
    }
    fn jz51_payment_options(&self, actor:usize)->Vec<ChoiceOption> {
        self.players[actor].graveyard.iter().filter(|c|corpse(c)).map(|c|self.option(c,actor,None,None)).collect()
    }
    fn validate_jz51_draft(&self,d:&GraveDraft)->RuleResult<()> {
        let actor=d.source.card.controller; self.jz51_source(actor,&d.source)?;
        if d.turn!=self.turn || d.source.region.is_some() || d.source.source_region_instance.is_some() || d.source.card.owner!=actor
            || self.status!="playing" || self.players[actor].eliminated || !self.stack.is_empty()
            || self.priority_team!=self.team(actor) || self.window!=Some(Window::Action(self.team(actor)))
            || !self.players[actor].graveyard.iter().any(|c|c.id==d.source.card.id && same(c,&d.source.card))
            || self.jz51_payment_options(actor).len()<3 || !self.seal_host_valid(&format!("society:p{actor}")) {
            return Err("温迪戈墓地行动须保留原墓地实体、标准窗口及三张费用".into());
        } Ok(())
    }
    pub(crate) fn jz51_begin_grave(&mut self,actor:usize,a:&Action)->RuleResult<()> {
        validate_action(a)?;
        if !self.can_standard(actor) {return Err("墓地行动需要本方标准行动窗口".into());}
        let c=self.players[actor].graveyard.iter().find(|c|Some(c.id.as_str())==a.card_id.as_deref() && c.definition=="JZ51").ok_or("温迪戈不在本人墓地")?;
        let draft=GraveDraft {source:self.source_snapshot(c,None),turn:self.turn}; self.validate_jz51_draft(&draft)?;
        let options=self.jz51_payment_options(actor);
        self.choice(actor,PAYMENT,PAYMENT_TITLE.into(),options,3,3,None,ChoiceResolution::JZ51Payment{draft:Box::new(draft)});
        Ok(())
    }
    pub(crate) fn jz51_finish_payment(&mut self,draft:GraveDraft,selected:&[String])->RuleResult<()> {
        self.validate_jz51_draft(&draft)?;
        let actor=draft.source.card.controller;
        if selected.len()!=3 || selected.iter().collect::<BTreeSet<_>>().len()!=3 {return Err("须支付三张不同的无名尸体".into());}
        let mut corpses=vec![];
        for id in selected {corpses.push(self.players[actor].graveyard.iter().find(|c|c.id==*id && corpse(c)).ok_or("尸体费用实例已失效")?.clone());}
        let host_id=format!("society:p{actor}");
        // Validate the whole batch before removing any corpse or allocating IDs.
        self.players[actor].graveyard.retain(|c|!selected.contains(&c.id));
        for c in &corpses { self.seal_card_on_valid_host(c.clone(),actor,&host_id); }
        let a=definition().abilities.remove(1);
        let f=self.make_frame(actor,draft.source,&a,vec![],vec![PaidCost::JZ51Corpses(CorpsePayment{turn:self.turn,host_id,corpses})],None);
        self.note(format!("{} 发动温迪戈墓地行动，已封印三张无名尸体",self.players[actor].name));
        self.dispatch_frame(f,a.label,None,ResponsePolicy::Respondable); Ok(())
    }
    pub(crate) fn validate_jz51_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        let c=&f.source.card;
        if matches!(f.ability_key.as_str(),"deploy"|"reveal") {
            if f.actor>=self.players.len() || c.definition!="JZ51" || c.controller!=f.actor || c.owner>=self.players.len()
                || c.id.is_empty() || f.frame_id.is_empty() || f.frame_id==c.id || f.cursor!=0 || !matches!(f.guard,GuardState::Unchecked)
                || f.source.region.is_none_or(|r|r>=self.regions.len()) || f.chosen_region!=f.source.region
                || f.source.source_region_instance.as_ref().is_none_or(|s|s.is_empty()) || f.source.attachment_host_instance.is_some()
                || f.source.death_echo.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
                || !f.targets.is_empty() || !f.steps.is_empty()
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()<=5 && ids.iter().all(|id|!id.is_empty() && *id!=c.id) && ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
                || f.ability_key=="deploy" && (c.face_down || !f.source.valid_character_play())
                || f.ability_key=="reveal" && (!c.face_down || f.source.play_source.is_some()) {
                return Err("温迪戈派遣/现身须保留实际印刷支付与原地区".into());
            }
            return self.mobility_immutable_identity(c);
        }
        self.jz51_source(f.actor,&f.source)?;
        if f.frame_id.is_empty() || f.frame_id==c.id || f.cursor>f.steps.len() || f.chosen_region.is_some()
            || !f.targets.is_empty() || f.cursor==0 && !matches!(f.guard,GuardState::Unchecked)
            || f.cursor>0 && !matches!(f.guard,GuardState::Accepted) {return Err("温迪戈执行帧游标、守卫或目标被改写".into());}
        let a=definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("未知温迪戈程序")?;
        if !same(&f.steps,&a.ops.into_iter().map(|op|Step{context:f.actor,op}).collect::<Vec<_>>()) {return Err("温迪戈只准入完整有限程序".into());}
        if f.ability_key==ENTRY {
            if !f.already_paid.is_empty() || f.source.region.is_none_or(|r|r>=self.regions.len()) || f.source.source_region_instance.as_ref().is_none_or(|s|s.is_empty()) {return Err("进场检索须保留原地区且没有额外支付".into());}
        } else {
            let [PaidCost::JZ51Corpses(p)]=f.already_paid.as_slice() else {return Err("墓地行动缺少三张封印费用收据".into());};
            if p.turn!=self.turn || p.host_id!=format!("society:p{}",f.actor) || p.corpses.len()!=3 || f.source.region.is_some() || f.source.source_region_instance.is_some() || c.owner!=f.actor {return Err("温迪戈墓地费用、回合或个人秘社区错误".into());}
            let mut ids=BTreeSet::new();
            for paid in &p.corpses {
                if !corpse(paid) || paid.owner!=f.actor || paid.controller!=f.actor || paid.face_down || paid.id.is_empty() || paid.id==c.id || !ids.insert(&paid.id) {return Err("温迪戈必须支付本人墓地三个不同尸体".into());}
                self.mobility_immutable_identity(paid)?;
                if self.batch3_physical_cards().any(|now|now.id==paid.id) {return Err("已封印旧实体不能仍留在其他区域".into());}
            }
        } Ok(())
    }
    pub(crate) fn jz51_search_start(&mut self,f:ResolutionFrame)->RuleResult<()> {
        let inspected_ids=self.players[f.actor].deck.iter().take(8).map(|c|c.id.clone()).collect::<Vec<_>>();
        let options=self.players[f.actor].deck.iter().take(8).filter(|c|corpse(c)).map(|c|self.option(c,f.actor,None,None)).collect::<Vec<_>>();
        let max=options.len(); self.choice(f.actor,SEARCH,SEARCH_TITLE.into(),options,0,max,None,
            ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::JZ51TopEight{inspected_ids}}); Ok(())
    }
    pub(crate) fn jz51_search_finish(&mut self,f:&ResolutionFrame,inspected_ids:&[String],selected:&[String])->RuleResult<()> {
        let now=self.players[f.actor].deck.iter().take(8).map(|c|c.id.clone()).collect::<Vec<_>>();
        if now!=inspected_ids || selected.iter().collect::<BTreeSet<_>>().len()!=selected.len()
            || selected.iter().any(|id|!self.players[f.actor].deck.iter().take(8).any(|c|c.id==*id && corpse(c))) {return Err("温迪戈检索必须来自原顶八张且不能重复".into());}
        let region=f.source.region.filter(|r|self.region_live(*r) && Some(&self.regions[*r].card.id)==f.source.source_region_instance.as_ref());
        let mut entered=vec![];
        if let Some(r)=region {for id in selected {
            let i=self.players[f.actor].deck.iter().position(|c|c.id==*id).unwrap();
            // FAQ physical page4: Martial Law forbids playing/moving the
            // relevant characters, but explicitly permits direct placement.
            let c=self.players[f.actor].deck.remove(i); let mut c=self.reset_zone_card(c);c.controller=f.actor;
            entered.push((c.definition.clone(),c.id.clone()));self.regions[r].cards.push(c);
        }}
        for (definition,id) in entered {self.enter_triggers(f.actor,&definition,&id,false);}
        self.shuffle_player(f.actor); self.note(format!("{} 完成温迪戈顶八张检索并洗牌",self.players[f.actor].name)); Ok(())
    }
    fn jz51_regions(&self,f:&ResolutionFrame)->Vec<(usize,String)> {
        if !self.players[f.actor].graveyard.iter().any(|c|c.id==f.source.card.id && c.definition=="JZ51") {return vec![];}
        (0..self.regions.len()).filter(|r|self.region_live(*r)).map(|r|(r,self.regions[r].card.id.clone())).collect()
    }
    fn jz51_region_options(&self,regions:&[(usize,String)])->Vec<ChoiceOption> {
        regions.iter().map(|(r,_)|ChoiceOption{id:format!("region:{r}"),label:format!("地区{}：{}",r+1,catalog::card(&self.regions[*r].card.definition).name),card:None}).collect()
    }
    pub(crate) fn jz51_grave_entry(&mut self,f:ResolutionFrame)->RuleResult<()> {
        let regions=self.jz51_regions(&f);if regions.is_empty(){return Ok(());}
        let options=self.jz51_region_options(&regions);
        self.choice(f.actor,REGION,REGION_TITLE.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::JZ51EntryRegion{regions}}); Ok(())
    }
    pub(crate) fn jz51_region_finish(&mut self,f:&ResolutionFrame,regions:&[(usize,String)],selected:&[String])->RuleResult<()> {
        if selected.len()!=1 || self.jz51_regions(f)!=regions {return Err("温迪戈原墓地实体或地区快照已失效".into());}
        let (r,_)=regions.iter().find(|(r,_)|selected[0]==format!("region:{r}")).ok_or("请选择当前可进场地区")?;
        self.world_graveyard_entry(f.actor,*r,&f.source.card.id)
    }
    pub(crate) fn jz51_private_choice(&self,p:&Pending,viewer:usize)->Choice {
        let mut out=p.choice.clone();
        if let ChoiceResolution::Frame{frame,choice:FrameChoice::JZ51TopEight{inspected_ids}}=&p.resolution {
            if viewer==frame.actor {out.preview_cards=inspected_ids.iter().filter_map(|id|self.players[viewer].deck.iter().find(|c|c.id==*id)).map(|c|self.card_view(c,viewer,None,None)).collect();}
        } out
    }
    pub(crate) fn validate_jz51_state(&self)->RuleResult<()> {
        let mut ids=BTreeSet::new();
        for item in &self.stack {if let Some(f)=&item.frame {
            if !related(f) || self.validated_cross_card_frame(f)? {continue;}
            self.validate_jz51_frame(f)?;
            let play=matches!(f.ability_key.as_str(),"deploy"|"reveal");
            if !ids.insert(&f.frame_id) || item.id!=f.frame_id || item.controller!=f.actor || f.cursor!=0 || item.target.is_some() || item.card.is_some()!=play
                || item.reveal!=(f.ability_key=="reveal") || item.deploy_region!=if play {f.source.region}else{None}
                || item.card.as_ref().is_some_and(|c|c.definition!="JZ51" || c.face_down || c.controller!=f.actor || c.owner!=f.source.card.owner) {return Err("温迪戈堆叠外壳必须绑定同一执行帧".into());}
        }}
        for e in &self.effects {match e {
            Effect::Declare{declaration:d} if d.source.card.definition=="JZ51" && !self.validated_cross_card_declaration(d)? => self.validate_jz51_declaration(d)?,
            Effect::Frame{frame:f} if related(f) && !self.validated_cross_card_frame(f)? => {self.validate_jz51_frame(f)?;if !ids.insert(&f.frame_id){return Err("温迪戈不能重复恢复同一执行帧".into());}},
            _=>(),
        }}
        let Some(p)=&self.pending else{return Ok(());};
        let related=matches!(p.resolution,ChoiceResolution::JZ51Payment{..} | ChoiceResolution::Frame{choice:FrameChoice::JZ51TopEight{..}|FrameChoice::JZ51EntryRegion{..},..});
        if related && (p.choice.id.is_empty() || p.choice.player_id!=player_id(p.seat) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || p.choice.description!="选择符合数量限制的选项，然后确认。") {return Err("温迪戈待选必须保留原玩家、编号与无额外载荷".into());}
        match &p.resolution {
            ChoiceResolution::JZ51Payment{draft}=>{
                self.validate_jz51_draft(draft)?;let actor=draft.source.card.controller;
                if p.seat!=actor || p.choice.kind!=PAYMENT || p.choice.title!=PAYMENT_TITLE || p.choice.min!=Some(3) || p.choice.max!=Some(3) || p.choice.allow_decline!=Some(false) || !same(&p.choice.options,&self.jz51_payment_options(actor)) {return Err("温迪戈未支付待选费用被改写".into());}
            },
            ChoiceResolution::Frame{frame:f,choice:FrameChoice::JZ51TopEight{inspected_ids}}=>{
                self.validate_jz51_frame(f)?;
                let options=self.players[f.actor].deck.iter().take(8).filter(|c|corpse(c)).map(|c|self.option(c,f.actor,None,None)).collect::<Vec<_>>();
                let current=self.players[f.actor].deck.iter().take(8).map(|c|c.id.clone()).collect::<Vec<_>>();
                if f.ability_key!=ENTRY || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || p.seat!=f.actor || p.choice.kind!=SEARCH || p.choice.title!=SEARCH_TITLE || p.choice.min!=Some(0) || p.choice.max!=Some(options.len()) || p.choice.allow_decline!=Some(true) || inspected_ids!=&current || !same(&p.choice.options,&options) {return Err("温迪戈顶八张私有待选快照被改写".into());}
            },
            ChoiceResolution::Frame{frame:f,choice:FrameChoice::JZ51EntryRegion{regions}}=>{
                self.validate_jz51_frame(f)?;
                if f.ability_key!=GRAVE || f.cursor!=1 || p.seat!=f.actor || p.choice.kind!=REGION || p.choice.title!=REGION_TITLE || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || regions!=&self.jz51_regions(f) || regions.is_empty() || !same(&p.choice.options,&self.jz51_region_options(regions)) {return Err("温迪戈结算进场地区必须保留原实体快照".into());}
            },
            ChoiceResolution::Frame{frame:f,..} if f.source.card.definition=="JZ51" && !self.validated_cross_card_frame(f)? =>return Err("温迪戈不能移植其他待选程序".into()),
            _ if matches!(p.choice.kind.as_str(),PAYMENT|SEARCH|REGION)=>return Err("温迪戈待选程序不匹配".into()),
            _=>(),
        } Ok(())
    }
}
