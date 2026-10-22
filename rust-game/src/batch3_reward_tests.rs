//! Explicit grant and completed-confrontation layouts, then normal reward
//! producers and choice/pass commands. Public grants/battles are separately
//! exercised by audit_regressions; these are native recovery shape checks.
use crate::{catalog, model::*, rules::*, unit_support::*};

fn reward(print: &str, glory: bool) -> Game {
    let mut g = game();
    let host = field(&mut g, print, 0);
    // Real static bonuses keep every admitted recipient alive under JC089.
    field(&mut g, "JC059", 0);
    field(&mut g, "JC059", 0);
    if glory {
        let card = g.make_card("JC089", 0);
        g.attachments.push(Attachment { card, host_id: host.clone() });
    } else {
        // Same existing turn modifier produced by JC074; the all-card public
        // integration audit separately pays and resolves that actual spell.
        g.turn_attribute_modifiers.push(TurnAttributeModifier {
            target_instance: host.clone(), defense_bonus: 0, kill_bonus: 0,
            grants_retreat: false, printed_defense_override: None,
            ordinary_icons: Icons::default(), grants_renown: true,
            grants_barrier: false, prevents_damage: false, expires_turn: g.turn,
        });
    }
    g.regions[0].influence = [0, 0];
    if glory {
        assert!(g.has_combat_glory(g.board(&host).unwrap().1), "{print}");
        assert!(g.current_icons(g.board(&host).unwrap().1, 0).combat > 0, "{print}");
        g.begin_window(Window::After(0, 1));
        g.reward(0, 0, 1, 1).unwrap();
    } else {
        g.begin_window(Window::After(0, 2));
        g.complete_influence_contest(0);
    }
    g.drive().unwrap();
    restore(&mut g);
    for _ in 0..8 {
        if let Some(p) = &g.pending {
            if let ChoiceResolution::Declare { declaration, .. } = &p.resolution {
                if declaration.ability.key == if glory { "jc089-combat-glory" } else { "renown" } {
                    assert_eq!(declaration.source.card.id, host);
                    crate::jc029_tests::checkpoint(&g);
                    return g;
                }
            }
            // Confrontation rewards may first select their normal recipient.
            // Continue through real choices instead of assuming no such step.
            resolve_choice(&mut g);
            continue;
        }
        assert!(!g.effects.is_empty(), "missing reward for {print}, glory={glory}");
        g.drive().unwrap();
    }
    panic!("bounded reward for {print}, glory={glory}");
}
fn invalid(g: &Game) {
    assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err());
    let room = crate::room::RoomEnvelope::from_game(g.clone());
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());
}

#[test]
fn batch3_runtime_rewards_keep_exact_program_and_choice_on_every_new_character() {
    let prints = catalog::catalog().cards[193..].iter().filter(|c| c.kind == "character").map(|c| c.id.as_str()).collect::<Vec<_>>();
    assert_eq!(prints.len(), 15);
    for print in prints { for glory in [false, true] {
        let mut g = reward(print, glory);
        for variant in 0..8 {
            let mut bad = g.clone();
            let p = bad.pending.as_mut().unwrap();
            let ChoiceResolution::Declare { declaration: d, stage } = &mut p.resolution else { unreachable!() };
            match variant {
                0 => *stage = DeclareChoice::Target,
                1 => p.choice.options.clear(),
                2 => p.choice.allow_decline = Some(false),
                3 => d.ability.ops = vec![Op::PlaceInfluence { region_instance: d.source.source_region_instance.clone().unwrap(), amount: 2 }],
                4 => d.source.source_region_instance = None,
                5 => d.source.play_source = Some(PlaySource::Hand),
                6 => d.actor = 1,
                _ => d.ability.response_policy = ResponsePolicy::Immediate,
            }
            invalid(&bad);
        }
        choose(&mut g, vec!["accept".into()]);
        crate::jc029_tests::checkpoint(&g);
        for variant in 0..7 {
            let mut bad = g.clone();
            let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
            match variant {
                0 => f.steps[0].context = 1,
                1 => f.steps[0].op = Op::PlaceInfluence { region_instance: f.source.source_region_instance.clone().unwrap(), amount: 2 },
                2 => f.source.source_region_instance = None,
                3 => { f.cursor = 1; f.guard = GuardState::Accepted; },
                4 => f.source.card.face_down = true,
                5 => f.already_paid = vec![PaidCost::Assets(vec![])],
                _ => f.chosen_region = Some(0),
            }
            invalid(&bad);
        }
        let before = g.regions[0].influence;
        assert_eq!(before[1], 0);
        top(&mut g);
        assert_eq!(g.regions[0].influence, [before[0] + 1, 0], "{print}, glory={glory}");
        crate::jc029_tests::checkpoint(&g);
    } }
}

#[test]
fn batch3_runtime_rewards_remain_independent_of_the_departed_original_source() {
    for print in ["BQ037", "CQ07", "JZ26", "JZ25", "BQ026"] { for glory in [false, true] {
        let mut g = reward(print, glory);
        choose(&mut g, vec!["accept".into()]);
        let source = g.stack.last().unwrap().frame.as_ref().unwrap().source.card.id.clone();
        g.remove_dead(&source, RemovalCause::Destroy);
        let replacement = field(&mut g, print, 0);
        crate::jc029_tests::checkpoint(&g);
        let before = g.regions[0].influence;
        assert_eq!(before[1], 0);
        top(&mut g);
        assert_eq!(g.regions[0].influence, [before[0] + 1, 0]);
        assert!(g.board(&replacement).is_some());
        assert!(g.board(&source).is_none());
        crate::jc029_tests::checkpoint(&g);
    } }
}
