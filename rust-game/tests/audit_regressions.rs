//! Audit regressions: explicit valid midgame layouts, followed by real public commands.
//! These are native round-trip tests, not browser or natural-deal playthroughs.

mod time_reward {
    // Synthetic initial layout, followed exclusively by public engine actions.
    use hegemony_server::model::*;
    use hegemony_server::room::{RoomCommand, RoomEnvelope, SessionAction};
    fn pass(g: &mut Game) -> Result<(), String> {
        let s = (0..g.players.len())
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("pass available");
        g.apply(s, Action::new("pass"))
    }
    fn pass_top(g: &mut Game) {
        let n = g.stack.len();
        for _ in 0..16 {
            if g.stack.len() < n {
                return;
            }
            pass(g).unwrap();
        }
        panic!("stack");
    }
    fn select(g: &mut Game, selected: Vec<String>) {
        let p = g.pending.clone().unwrap();
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    fn run(add_time: bool) -> bool {
        let mut g = Game::new(
            "audit-rat".into(),
            "AUDIT".into(),
            "duel".into(),
            "P0".into(),
            "watchers".into(),
            7,
        )
        .unwrap();
        g.join("P1".into(), "watchers".into()).unwrap();
        for p in &mut g.players {
            p.ready = true;
        }
        g.apply(0, Action::new("start")).unwrap();
        while g.pending.is_some() {
            select(&mut g, vec![]);
        }
        for p in &mut g.players {
            p.hand.clear();
            p.assets.clear();
        }
        for r in &mut g.regions {
            r.cards.clear();
            r.influence = [0, 0];
        }
        g.effects.clear();
        g.stack.clear();
        g.pending = None;
        g.window = Some(Window::Action(0));
        g.first_team = 0;
        g.active_team = 0;
        g.priority_team = 0;
        g.passed.clear();
        g.team_passed = [false; 2];
        g.jc035_preparation = None;
        let rat = g.make_card("BQ028", 0);
        let rat_id = rat.id.clone();
        g.regions[0].cards.push(rat);
        for def in ["XQ18", "XQ18", "JC074", "JC074"] {
            let c = g.make_card(def, 0);
            g.players[0].assets.push(c);
        }
        let grant = g.make_card("JC074", 0);
        let grant_id = grant.id.clone();
        g.players[0].hand.push(grant);
        if add_time {
            let mut timer = g.make_card("XQ18", 0);
            timer.face_down = true;
            let timer_id = timer.id.clone();
            g.regions[0].cards.push(timer);
            g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
            g.apply(
                0,
                Action {
                    card_id: Some(timer_id),
                    ..Action::new("reveal")
                },
            )
            .unwrap();
            pass_top(&mut g);
            select(&mut g, vec!["add-time".into()]);
            select(&mut g, vec![rat_id.clone()]);
            pass_top(&mut g);
            assert_eq!(
                g.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.id == rat_id)
                    .unwrap()
                    .time_markers,
                1
            );
        }
        g.apply(
            0,
            Action {
                card_id: Some(grant_id),
                target_id: Some(rat_id.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_top(&mut g);
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        for _ in 0..64 {
            if let Some(p) = g.pending.clone() {
                if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="renown")
                {
                    let bytes = serde_json::to_string(&g).unwrap();
                    let restored = Game::from_persisted(&bytes);
                    println!("add_time={add_time}: public actions reached renown choice; source_time={}; restore={:?}",declaration_time(&p), restored.as_ref().map(|_|"ok").map_err(|e|e.as_str()));
                    return restored.is_ok();
                }
                if p.choice.kind == "investigation" {
                    let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
                    g.apply(
                        p.seat,
                        Action {
                            choice_id: Some(p.choice.id),
                            top: Some(ids),
                            bottom: Some(vec![]),
                            ..Action::new("choose")
                        },
                    )
                    .unwrap();
                } else {
                    select(&mut g, vec![]);
                }
            } else {
                if add_time && g.window == Some(Window::Before(0, 2)) && g.priority_team == 1 {
                    let envelope = RoomEnvelope::from_game(g.clone());
                    let envelope =
                        RoomEnvelope::from_persisted(&serde_json::to_string(&envelope).unwrap())
                            .unwrap();
                    let transition = envelope
                        .transition(
                            1,
                            Some(RoomCommand {
                                command_id: "audit-final-pass".into(),
                                expected_version: envelope.revision,
                                action: SessionAction::Game {
                                    action: Action::new("pass"),
                                },
                            }),
                            1000,
                        )
                        .unwrap();
                    let restored = RoomEnvelope::from_persisted(&transition.state);
                    println!(
                        "room last pass: outcome={} changed={} next_restore={:?}",
                        transition.outcome,
                        transition.changed,
                        restored.as_ref().map(|_| "ok").map_err(|e| e.as_str())
                    );
                    assert_eq!(transition.outcome, "accepted");
                    assert!(transition.changed);
                    assert!(restored.is_ok());
                }
                let before = serde_json::to_string(&g).unwrap();
                let window = g.window.clone();
                if let Err(e) = pass(&mut g) {
                    assert_eq!(before, serde_json::to_string(&g).unwrap());
                    println!("add_time={add_time}: closing {window:?} rejected: {e}; state rollback=true");
                    return false;
                }
            }
        }
        panic!("renown not reached");
    }
    fn declaration_time(p: &Pending) -> u32 {
        if let ChoiceResolution::Declare { declaration, .. } = &p.resolution {
            declaration.source.card.time_markers
        } else {
            0
        }
    }
    #[test]
    fn public_commands_restore() {
        assert!(run(false));
        assert!(run(true));
    }
}

mod renown {
    // Cross-card runtime grant audit; synthetic, documented pre-confrontation layout.
    use hegemony_server::{catalog, model::*};
    fn step_pass(g: &mut Game) -> Result<(), String> {
        let s = (0..2)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .ok_or("no pass")?;
        g.apply(s, Action::new("pass"))
    }
    #[test]
    fn public_commands_restore() {
        let mut base = Game::new(
            "audit-grants".into(),
            "AUDIT".into(),
            "duel".into(),
            "P0".into(),
            "watchers".into(),
            7,
        )
        .unwrap();
        base.join("P1".into(), "watchers".into()).unwrap();
        for p in &mut base.players {
            p.ready = true;
        }
        base.apply(0, Action::new("start")).unwrap();
        while let Some(p) = base.pending.clone() {
            base.apply(
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            )
            .unwrap();
        }
        for p in &mut base.players {
            p.hand.clear();
            p.assets.clear();
        }
        for r in &mut base.regions {
            r.cards.clear();
            r.influence = [0, 3];
        }
        base.effects.clear();
        base.stack.clear();
        base.pending = None;
        base.window = Some(Window::Before(0, 2));
        base.first_team = 0;
        base.priority_team = 0;
        base.passed.clear();
        base.team_passed = [false; 2];
        base.jc035_preparation = None;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut skipped_ids = Vec::new();
        let mut checked = 0;
        for d in catalog::catalog()
            .cards
            .iter()
            .filter(|d| d.kind == "character" && d.supported)
        {
            checked += 1;
            let mut g = base.clone();
            let c = g.make_card(&d.id, 0);
            let id = c.id.clone();
            g.regions[0].cards.push(c);
            let spell = g.make_card("JC074", 0);
            let spell_id = spell.id.clone();
            g.players[0].hand.push(spell);
            for _ in 0..2 {
                let c = g.make_card("JC074", 0);
                g.players[0].assets.push(c);
            }
            Game::from_persisted(&serde_json::to_string(&g).unwrap())
                .expect("valid initial layout");
            let action = Action {
                card_id: Some(spell_id),
                target_id: Some(id.clone()),
                ..Action::new("play")
            };
            if let Err(e) = g.apply(0, action) {
                println!("SKIP {} grant rejected: {e}", d.id);
                skipped += 1;
                skipped_ids.push(d.id.clone());
                continue;
            }
            while !g.stack.is_empty() {
                step_pass(&mut g).expect("resolve grant");
            }
            if !g.view(0).regions[0]
                .characters
                .iter()
                .any(|c| c.instance_id == id && c.current_renown == Some(true))
            {
                println!("SKIP {} no live granted renown", d.id);
                skipped += 1;
                skipped_ids.push(d.id.clone());
                continue;
            }
            let mut failure = None;
            // JC067 adds a complete influence contest. Close its normal before/
            // after windows until the region has really ended, without forging
            // influence progress or bypassing any public command.
            for _ in 0..12 {
                if g.pending.is_some() {
                    break;
                }
                let before = serde_json::to_string(&g).unwrap();
                if let Err(e) = step_pass(&mut g) {
                    assert_eq!(before, serde_json::to_string(&g).unwrap());
                    failure = Some(format!("close rejected: {e}"));
                    break;
                }
            }
            if failure.is_none() {
                if let Err(e) = Game::from_persisted(&serde_json::to_string(&g).unwrap()) {
                    failure = Some(format!("restore rejected: {e}"));
                }
            }
            if let Some(e)=failure{println!("FAIL {} {}: {e}",d.id,d.name);failed+=1;}
  else if g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="renown")){passed+=1;}
  else{println!("SKIP {} no renown pending window={:?}",d.id,g.window);skipped+=1;skipped_ids.push(d.id.clone());}
        }
        assert_eq!(skipped_ids, ["XQ46"]);
        assert_eq!((passed, failed, skipped), (checked - 1, 0, 1));
        println!("SUMMARY passed={passed} failed={failed} skipped={skipped}");
    }
}

mod glory {
    // Regression: valid synthetic pre-combat layout; all grants and resolution use public commands.
    use hegemony_server::{catalog, model::*};
    fn apply(g: &mut Game, s: usize, a: Action) -> Result<(), String> {
        let old = serde_json::to_string(g).unwrap();
        if let Err(e) = g.apply(s, a) {
            assert_eq!(old, serde_json::to_string(g).unwrap());
            return Err(format!("command: {e}"));
        }
        Game::from_persisted(&serde_json::to_string(g).unwrap())
            .map(|_| ())
            .map_err(|e| format!("restore: {e}"))
    }
    fn pass(g: &mut Game) -> Result<(), String> {
        let s = (0..2)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .ok_or("no pass")?;
        apply(g, s, Action::new("pass"))
    }
    #[test]
    fn public_commands_restore() {
        let mut base = Game::new(
            "audit-glory".into(),
            "AUDIT".into(),
            "duel".into(),
            "P0".into(),
            "watchers".into(),
            7,
        )
        .unwrap();
        base.join("P1".into(), "watchers".into()).unwrap();
        for p in &mut base.players {
            p.ready = true;
        }
        base.apply(0, Action::new("start")).unwrap();
        while let Some(p) = base.pending.clone() {
            base.apply(
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            )
            .unwrap();
        }
        for p in &mut base.players {
            p.hand.clear();
            p.assets.clear();
        }
        for r in &mut base.regions {
            r.cards.clear();
            r.influence = [0, 3];
        }
        base.effects.clear();
        base.stack.clear();
        base.pending = None;
        base.window = Some(Window::Action(0));
        base.first_team = 0;
        base.priority_team = 0;
        base.passed.clear();
        base.team_passed = [false; 2];
        base.jc035_preparation = None;
        let mut counts = [0; 3];
        let mut checked = 0;
        for d in catalog::catalog()
            .cards
            .iter()
            .filter(|d| d.kind == "character" && d.supported)
        {
            checked += 1;
            let mut g = base.clone();
            let c = g.make_card(&d.id, 0);
            let id = c.id.clone();
            g.regions[0].cards.push(c);
            // Real static friendly defense bonuses keep printed-defense-one targets alive under the curse.
            for _ in 0..2 {
                let c = g.make_card("JC059", 0);
                g.regions[0].cards.push(c);
            }
            let spell = g.make_card("JC089", 0);
            let spell_id = spell.id.clone();
            g.players[0].hand.push(spell);
            for _ in 0..2 {
                let c = g.make_card("JC084", 0);
                g.players[0].assets.push(c);
            }
            Game::from_persisted(&serde_json::to_string(&g).unwrap())
                .expect("valid initial layout");
            if let Err(e) = apply(
                &mut g,
                0,
                Action {
                    card_id: Some(spell_id),
                    target_id: Some(id.clone()),
                    ..Action::new("play")
                },
            ) {
                println!("SKIP {} curse rejected {e}", d.id);
                counts[2] += 1;
                continue;
            }
            let mut fail = None;
            for _ in 0..8 {
                if g.stack.is_empty() {
                    break;
                }
                if let Err(e) = pass(&mut g) {
                    fail = Some(e);
                    break;
                }
            }
            if fail.is_none()
                && !g
                    .attachments
                    .iter()
                    .any(|a| a.host_id == id && a.card.definition == "JC089")
            {
                println!("SKIP {} curse not attached", d.id);
                counts[2] += 1;
                continue;
            }
            let mut success = false;
            for _ in 0..80 {
                if fail.is_some() {
                    break;
                }
                if let Some(p) = g.pending.clone() {
                    if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="jc089-combat-glory")
                    {
                        if let Err(e) = apply(
                            &mut g,
                            p.seat,
                            Action {
                                choice_id: Some(p.choice.id),
                                selected: Some(vec!["accept".into()]),
                                ..Action::new("choose")
                            },
                        ) {
                            fail = Some(e);
                            break;
                        }
                        for _ in 0..8 {
                            if g.stack.is_empty() {
                                success = true;
                                break;
                            }
                            if let Err(e) = pass(&mut g) {
                                fail = Some(e);
                                break;
                            }
                        }
                        break;
                    } else if p.choice.kind == "recipient" {
                        let a = Action {
                            choice_id: Some(p.choice.id),
                            selected: Some(vec![p.choice.options[0].id.clone()]),
                            ..Action::new("choose")
                        };
                        if let Err(e) = apply(&mut g, p.seat, a) {
                            fail = Some(e);
                        }
                    } else if p.choice.kind == "investigation" {
                        let a = Action {
                            choice_id: Some(p.choice.id),
                            top: Some(p.choice.options.iter().map(|o| o.id.clone()).collect()),
                            bottom: Some(vec![]),
                            ..Action::new("choose")
                        };
                        if let Err(e) = apply(&mut g, p.seat, a) {
                            fail = Some(e);
                        }
                    } else if matches!(&p.resolution, ChoiceResolution::Declare { .. })
                        && p.choice.min == Some(0)
                    {
                        let a = Action {
                            choice_id: Some(p.choice.id),
                            selected: Some(vec![]),
                            ..Action::new("choose")
                        };
                        if let Err(e) = apply(&mut g, p.seat, a) {
                            fail = Some(e);
                        }
                    } else {
                        fail = Some(format!(
                            "unhandled probe choice {} min={:?}",
                            p.choice.kind, p.choice.min
                        ));
                    }
                } else if let Err(e) = pass(&mut g) {
                    fail = Some(e);
                }
            }
            if let Some(e) = fail {
                println!("FAIL {} {}: {}", d.id, d.name, e);
                counts[1] += 1;
            } else if success {
                counts[0] += 1;
            } else {
                println!("SKIP {} no reward window={:?}", d.id, g.window);
                counts[2] += 1;
            }
        }
        assert_eq!(counts, [checked, 0, 0]);
        println!(
            "SUMMARY passed={} failed={} skipped={}",
            counts[0], counts[1], counts[2]
        );
    }
}

mod unique {
    // Explicit valid midgame input; from then on only public Game actions.
    use hegemony_server::model::*;
    fn apply(g: &mut Game, s: usize, a: Action) {
        g.apply(s, a).unwrap();
        *g = Game::from_persisted(&serde_json::to_string(g).unwrap()).unwrap();
    }
    fn pass(g: &mut Game) {
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("public pass");
        apply(g, s, Action::new("pass"));
    }
    fn top(g: &mut Game) {
        let n = g.stack.len();
        for _ in 0..32 {
            if g.stack.len() < n {
                return;
            }
            pass(g);
        }
        panic!("response loop");
    }
    fn choose(g: &mut Game) {
        let p = g.pending.clone().unwrap();
        let mut a = Action {
            choice_id: Some(p.choice.id),
            ..Action::new("choose")
        };
        if matches!(
            p.choice.kind.as_str(),
            "investigation" | "order" | "forecast"
        ) {
            a.top = Some(p.choice.options.iter().map(|o| o.id.clone()).collect());
            a.bottom = Some(vec![]);
        } else if p.choice.kind == "damage" {
            panic!("unexpected damage");
        } else {
            a.selected = Some(
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect(),
            );
        }
        apply(g, p.seat, a);
    }
    #[test]
    fn public_commands_restore() {
        let mut g = Game::new(
            "unique-public".into(),
            "AUDIT".into(),
            "teams".into(),
            "P0".into(),
            "watchers".into(),
            7,
        )
        .unwrap();
        for s in 1..4 {
            g.join(format!("P{s}"), "watchers".into()).unwrap();
        }
        for p in &mut g.players {
            p.ready = true;
        }
        g.apply(0, Action::new("start")).unwrap();
        while g.pending.is_some() {
            choose(&mut g);
        }
        for p in &mut g.players {
            p.hand.clear();
            p.assets.clear();
        }
        for r in &mut g.regions {
            r.cards.clear();
            r.influence = [0, 0];
        }
        g.effects.clear();
        g.stack.clear();
        g.pending = None;
        g.window = Some(Window::Action(1));
        g.first_team = 1;
        g.active_team = 1;
        g.priority_team = 1;
        g.passed.clear();
        g.team_passed = [false; 2];
        g.jc035_preparation = None;
        let mut targets = vec![];
        let mut copies = vec![];
        let mut spells = vec![];
        for (r, def) in ["LC23", "LC01"].into_iter().enumerate() {
            let c = g.make_card(def, 0);
            targets.push(c.id.clone());
            g.regions[r].cards.push(c);
            let c = g.make_card(def, 0);
            copies.push(c.id.clone());
            g.players[0].hand.push(c);
            let c = g.make_card("JC129", 2);
            spells.push(c.id.clone());
            g.players[2].hand.push(c);
        }
        for _ in 0..6 {
            let c = g.make_card("JC104", 2);
            g.players[2].assets.push(c);
        }
        for _ in 0..8 {
            let c = g.make_card("LC01", 0);
            g.players[0].assets.push(c);
        }
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        for i in 0..2 {
            apply(
                &mut g,
                2,
                Action {
                    card_id: Some(spells[i].clone()),
                    target_id: Some(targets[i].clone()),
                    ..Action::new("play")
                },
            );
            top(&mut g);
        }
        println!(
            "two real JC129 casts accepted; control effects={}",
            g.control_effects.len()
        );
        for _ in 0..20 {
            if g.window == Some(Window::Action(0)) {
                break;
            }
            if g.pending.is_some() {
                choose(&mut g);
            } else {
                pass(&mut g);
            }
        }
        assert_eq!(g.window, Some(Window::Action(0)));
        for (r, id) in copies.into_iter().enumerate() {
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    region: Some(r),
                    ..Action::new("deploy")
                },
            );
            top(&mut g);
            while g.pending.is_some() {
                choose(&mut g);
            }
        }
        println!("two replacement deployments accepted, each name still split across P0/P2");
        for _ in 0..600 {
            if g.pending
                .as_ref()
                .is_some_and(|p| matches!(p.resolution, ChoiceResolution::Unique))
            {
                break;
            }
            if g.pending.is_some() {
                choose(&mut g);
            } else {
                pass(&mut g);
            }
        }
        let p = g
            .pending
            .clone()
            .expect("unique check after actual end cleanup");
        assert!(matches!(p.resolution, ChoiceResolution::Unique));
        println!(
            "actual end cleanup offers {} choices; controls={}",
            p.choice.options.len(),
            g.control_effects.len()
        );
        choose(&mut g);
        let counts = ["LC23", "LC01"].map(|def| {
            g.regions
                .iter()
                .flat_map(|r| &r.cards)
                .filter(|c| c.definition == def && c.controller == 0 && !c.face_down)
                .count()
        });
        let queued = g
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Unique { .. }))
            .count();
        println!("after single required sacrifice and restore: counts={counts:?}, pending={}, queued_unique={queued}, window={:?}",g.pending.is_some(),g.window);
        assert!(counts.contains(&2));
        assert!(g
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.resolution, ChoiceResolution::Unique)));
        choose(&mut g);
        let final_counts = ["LC23", "LC01"].map(|def| {
            g.regions
                .iter()
                .flat_map(|r| &r.cards)
                .filter(|c| c.definition == def && c.controller == 0 && !c.face_down)
                .count()
        });
        assert_eq!(final_counts, [1, 1]);
        assert!(!g
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.resolution, ChoiceResolution::Unique)));
    }
}

mod attachment_icons {
    // Explicit valid midgame layout; all following actions are public engine commands.
    use hegemony_server::model::*;
    fn apply(g: &mut Game, s: usize, a: Action) {
        g.apply(s, a).unwrap();
        *g = Game::from_persisted(&serde_json::to_string(g).unwrap()).unwrap();
    }
    fn pass(g: &mut Game) {
        let s = (0..g.players.len())
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("pass");
        apply(g, s, Action::new("pass"));
    }
    #[test]
    fn public_commands_restore() {
        for def in ["XQ25", "JC035"] {
            let mut g = Game::new(
                format!("icons-{def}"),
                "AUDIT".into(),
                "duel".into(),
                "P0".into(),
                "watchers".into(),
                7,
            )
            .unwrap();
            g.join("P1".into(), "watchers".into()).unwrap();
            for p in &mut g.players {
                p.ready = true;
            }
            g.apply(0, Action::new("start")).unwrap();
            while let Some(p) = g.pending.clone() {
                apply(
                    &mut g,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                );
            }
            for p in &mut g.players {
                p.hand.clear();
                p.assets.clear();
            }
            for r in &mut g.regions {
                r.cards.clear();
                r.influence = [0, 0];
            }
            g.effects.clear();
            g.stack.clear();
            g.pending = None;
            g.window = Some(Window::Action(0));
            g.first_team = 0;
            g.active_team = 0;
            g.priority_team = 0;
            g.passed.clear();
            g.team_passed = [false; 2];
            g.jc035_preparation = None;
            for _ in 0..5 {
                let c = g.make_card(def, 0);
                g.players[0].assets.push(c);
            }
            let c = g.make_card(def, 0);
            let id = c.id.clone();
            g.players[0].hand.push(c);
            g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();

            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    region: Some(0),
                    ..Action::new("play")
                },
            );
            for _ in 0..16 {
                if g.stack.is_empty() {
                    break;
                }
                pass(&mut g);
            }
            assert_eq!(g.attachments.len(), 1);
            let a = &g.attachments[0];
            assert!(!a.card.exhausted && !a.card.face_down);
            assert_eq!(a.card.controller, 0);
            println!(
                "{def}: actual paid play, cost={}, unexhausted first-team attachment, icons={:?}",
                g.players[0].assets.iter().filter(|c| c.exhausted).count(),
                g.icons(&a.card, 0)
            );
            let mut investigated = false;
            for _ in 0..48 {
                if let Some(p) = &g.pending {
                    assert_eq!(p.choice.kind, "investigation");
                    investigated = true;
                    println!(
                        "{def}: actual investigation reward pending: seat={}, options={}",
                        p.seat,
                        p.choice.options.len()
                    );
                    break;
                }
                if g.window == Some(Window::After(0, 0)) {
                    println!("{def}: investigation completed without reward");
                    break;
                }
                pass(&mut g);
            }
            assert!(
                investigated,
                "{def} must contribute its printed investigation icon"
            );
        }
    }
}
