use crate::{batch4_pathway as path, catalog, model::*, rules::*, unit_support::*};
fn play(g: &mut Game, actor: usize, definition: &str) {
    g.begin_window(Window::Action(g.team(actor))); fund(g, actor, "BQ040", 3); let id = held(g, definition, actor);
    act(g, actor, Action { card_id: Some(id), target_id: Some(format!("society:p{actor}")), ..Action::new("play") });
}
fn setup() -> (Game, String, String) {
    let mut g = game(); let vampire = field(&mut g, "JC032", 0); play(&mut g, 0, "XQ13"); top(&mut g);
    let source = g.attachments.iter().find(|a| a.card.definition == "XQ13").unwrap().card.id.clone(); (g, vampire, source)
}
fn pending(g: &mut Game) { for _ in 0..96 { if g.pending.is_some() { restore(g); return; } if !g.effects.is_empty() { g.drive().unwrap(); } else { pass(g); } } panic!("bounded Pathway choice"); }
fn bad(g: &Game) { assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err()); }
#[test]
fn batch4_pathway_exact_gold_print_triple_loyalty_and_complete_society_definition_are_closed() {
    let c = catalog::card("XQ13"); assert_eq!((c.cost, c.unique, c.supported), (1, true, false)); assert_eq!(c.loyalty, ["蓝色", "蓝色", "蓝色"]);
    let d = path::definition(); assert_eq!(d.abilities.len(), 1); assert_eq!(d.attachment.as_ref().unwrap().host.zone, Zone::Society);
    for mode in 0..3 { let mut wrong = d.clone(); match mode { 0 => wrong.modifiers.clear(), 1 => wrong.abilities[0].targets[0].relation = Relation::Any, _ => wrong.abilities[0].ops.push(Op::Draw { player: PlayerRef::Actor, count: 1, end: DeckEnd::Top }) }; assert!(crate::batch4::validate_definition("XQ13", &wrong).is_err()); }
    assert!(crate::batch4::validate_definition("JC125", &d).is_err());
}
#[test]
fn batch4_pathway_real_society_play_pays_one_without_creating_a_missing_society_card() {
    let (mut g, _, source) = setup(); assert_eq!(g.resources(0), 2); assert!(g.players[0].society_zone.card.is_none());
    assert_eq!(g.attachments.iter().find(|a| a.card.id == source).unwrap().host_id, "society:p0");
    for seat in 0..4 { let v = serde_json::to_value(g.view(seat)).unwrap(); assert!(v.to_string().contains(&source)); }
    restore(&mut g);
}
#[test]
fn batch4_pathway_insufficient_loyalty_enemy_society_and_extra_region_are_atomic_rejections() {
    let mut g = game(); fund(&mut g, 0, "BQ040", 2); let id = held(&mut g, "XQ13", 0);
    reject(&mut g, 0, Action { card_id: Some(id.clone()), target_id: Some("society:p0".into()), ..Action::new("play") });
    fund(&mut g, 0, "BQ040", 1);
    for target in ["society:p1", "society:p2"] { reject(&mut g, 0, Action { card_id: Some(id.clone()), target_id: Some(target.into()), ..Action::new("play") }); }
    reject(&mut g, 0, Action { card_id: Some(id), target_id: Some("society:p0".into()), region: Some(0), ..Action::new("play") }); assert_eq!(g.resources(0), 3);
}
#[test]
fn batch4_pathway_each_printed_permanent_and_temporary_noncombat_icon_becomes_permanent_combat() {
    let (mut g, vampire, _) = setup(); let c = g.board(&vampire).unwrap().1; let d = catalog::card(&c.definition);
    let expected = d.permanent_icons.combat + d.permanent_icons.investigation + d.permanent_icons.influence + d.temporary_icons.investigation + d.temporary_icons.influence;
    for first in [0,1] { g.first_team = first; let c = g.board(&vampire).unwrap().1; let icons = g.current_icons(c, 0);
        assert_eq!((icons.investigation, icons.influence), (0,0)); assert_eq!(icons.combat, expected + if first == 0 { d.temporary_icons.combat } else { 0 }); assert_eq!(g.current_permanent_combat(c,0), expected);
        for seat in 0..4 { let v = g.view(seat); let c = v.regions[0].characters.iter().find(|c| c.instance_id == vampire).unwrap(); assert_eq!(c.icons.as_ref().unwrap().combat, icons.combat); }
        restore(&mut g);
    }
}
#[test]
fn batch4_pathway_personal_vampires_any_region_convert_but_allies_enemies_and_humans_do_not() {
    let (mut g, vampire, _) = setup(); let far = field(&mut g, "JC032", 0); let c = g.regions[0].cards.pop().unwrap(); assert_eq!(c.id,far); g.regions[1].cards.push(c);
    let ally = field(&mut g, "JC032", 1); let enemy = field(&mut g, "JC032", 2); let human = field(&mut g, "JC125", 0);
    for id in [&vampire,&far] { let (r,c) = g.board(id).unwrap(); assert_eq!(g.current_icons(c,r).influence,0); }
    for id in [&ally,&enemy,&human] { let (r,c) = g.board(id).unwrap(); let d=catalog::card(&c.definition); let expected=d.permanent_icons.add(if g.team(c.controller)==g.first_team{d.temporary_icons}else{Icons::default()}); assert_eq!(g.current_icons(c,r),expected); } restore(&mut g);
}
#[test]
fn batch4_pathway_ordinary_turn_icons_keep_category_and_real_blood_attachment_combat_stays_combat() {
    let (mut g, vampire, _) = setup(); let current = g.current_icons(g.board(&vampire).unwrap().1,0);
    g.turn_attribute_modifiers.push(TurnAttributeModifier { target_instance:vampire.clone(),defense_bonus:0,kill_bonus:0,grants_retreat:false,printed_defense_override:None,
        ordinary_icons:Icons{investigation:2,influence:3,combat:1},grants_renown:false,grants_barrier:false,prevents_damage:false,expires_turn:g.turn });
    assert_eq!(g.current_icons(g.board(&vampire).unwrap().1,0), current.add(Icons{investigation:2,influence:3,combat:1})); restore(&mut g);
    g.begin_window(Window::Action(0)); let blood=held(&mut g,"BQ040",0); act(&mut g,0,Action{card_id:Some(blood),target_id:Some(vampire.clone()),..Action::new("play")}); top(&mut g);
    assert_eq!(g.current_icons(g.board(&vampire).unwrap().1,0).combat,current.combat+2); restore(&mut g);
}
#[test]
fn batch4_pathway_hidden_or_exhausted_recipient_and_source_departure_restore_existing_behavior() {
    let (mut g,vampire,source)=setup(); let printed=catalog::card("JC032"); let expected=printed.permanent_icons.add(printed.temporary_icons);
    g.board_mut(&vampire).unwrap().exhausted=true; assert_eq!(g.icons(g.board(&vampire).unwrap().1,0),Icons::default()); assert!(g.current_icons(g.board(&vampire).unwrap().1,0).combat>0); restore(&mut g);
    g.board_mut(&vampire).unwrap().exhausted=false; g.board_mut(&vampire).unwrap().face_down=true; assert_eq!(g.current_icons(g.board(&vampire).unwrap().1,0),Icons{influence:1,..Icons::default()}); restore(&mut g);
    g.board_mut(&vampire).unwrap().face_down=false; g.remove_dead(&source,RemovalCause::Destroy); assert_eq!(g.current_icons(g.board(&vampire).unwrap().1,0),expected); restore(&mut g);
}
#[test]
fn batch4_pathway_real_embrace_begins_conversion_and_personal_control_change_ends_it() {
    let (mut g,_,_)=setup(); let host=field(&mut g,"JC125",0); fund(&mut g,0,"BQ040",5); let id=held(&mut g,"JC036",0);
    act(&mut g,0,Action{card_id:Some(id),target_id:Some(host.clone()),..Action::new("play")}); top(&mut g); let c=g.board(&host).unwrap().1; assert!(g.current_subtypes(c).iter().any(|s|s=="吸血鬼")); assert_eq!(g.current_icons(c,0).influence,0); restore(&mut g);
    g.begin_window(Window::Action(g.team(2))); fund(&mut g,2,"JC104",3); let spell=held(&mut g,"JC129",2);
    act(&mut g,2,Action{card_id:Some(spell),target_id:Some(host.clone()),..Action::new("play")});top(&mut g);
    let c=g.board(&host).unwrap().1;assert_eq!(c.controller,2);assert_eq!(g.current_icons(c,0),catalog::card("JC125").permanent_icons);restore(&mut g);
}
#[test]
fn batch4_pathway_different_named_pathways_force_personal_sacrifice_and_remaining_one_is_live() {
    for keep_new in [false,true] { let (mut g,_,source)=setup(); play(&mut g,0,"JC034"); pending(&mut g); let p=g.pending.clone().unwrap(); assert!(p.choice.title.starts_with("道途：")); assert_eq!(p.choice.options.len(),2);
        let remove=if keep_new{source.clone()}else{p.choice.options.iter().find(|o|o.id!=source).unwrap().id.clone()}; choose(&mut g,vec![remove]); assert_eq!(g.attachments.len(),1); assert_eq!(g.attachments[0].card.definition,if keep_new{"JC034"}else{"XQ13"}); restore(&mut g);
    }
}
#[test]
fn batch4_pathway_multiple_same_and_different_pathways_require_repeated_sacrifices_without_stacking() {
    let (mut g,vampire,_) = setup(); let before=g.current_icons(g.board(&vampire).unwrap().1,0);
    for id in ["JC034","XQ13"] { let c=g.make_card(id,0);g.attachments.push(Attachment{card:c,host_id:"society:p0".into()}); }
    g.settle_deaths();g.drive().unwrap();restore(&mut g);assert_eq!(g.current_icons(g.board(&vampire).unwrap().1,0),before);
    for count in [3,2] { let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.options.len(),count);let id=p.choice.options[0].id.clone();choose(&mut g,vec![id]); }
    assert_eq!(g.attachments.len(),1);assert!(g.pending.is_none());restore(&mut g);
}
#[test]
fn batch4_pathway_different_players_each_keep_one_and_conflict_choice_rejects_forged_metadata() {
    let (mut g,_,_) = setup(); play(&mut g,1,"XQ13");top(&mut g);assert_eq!(g.attachments.len(),2);play(&mut g,0,"JC034");pending(&mut g);
    for mode in 0..4 {let mut wrong=g.clone();let p=wrong.pending.as_mut().unwrap();match mode{0=>p.seat=1,1=>{p.choice.options.pop();},2=>p.choice.title="独有".into(),_=>p.choice.min=Some(0)}bad(&wrong);}
    let p=g.pending.clone().unwrap();let id=p.choice.options[0].id.clone();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![id.clone()]),cost_selected:Some(vec![id]),..Action::new("choose")});
    let mut wrong=g.clone();wrong.pending=None;wrong.effects.retain(|e|!matches!(e,Effect::Unique{..}));bad(&wrong);
}
#[test]
fn batch4_pathway_paid_frame_program_source_target_and_duplicate_container_cannot_be_forged() {
    let mut g=game();play(&mut g,0,"XQ13");
    for mode in 0..5 {let mut wrong=g.clone();let item=wrong.stack.last_mut().unwrap();let f=item.frame.as_mut().unwrap();match mode{0=>f.steps.push(Step{context:0,op:Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top}}),1=>f.already_paid.clear(),2=>f.targets[0].id="society:p2".into(),3=>f.source.card.id=item.card.as_ref().unwrap().id.clone(),_=>f.chosen_region=Some(0)}bad(&wrong);}
    let mut wrong=g.clone();wrong.stack.push(wrong.stack[0].clone());bad(&wrong); let mut wrong=g.clone();let f=wrong.stack.pop().unwrap().frame.unwrap();wrong.effects.push_back(Effect::Frame{frame:Box::new(f)});bad(&wrong);
}
