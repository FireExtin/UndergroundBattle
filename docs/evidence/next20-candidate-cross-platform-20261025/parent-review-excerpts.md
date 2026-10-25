# Parent review excerpts at 9afd8a573d8e7a3c6a416537578364846ce5534d

Production methods below are unchanged from runtime 1bd0776. The two blue review fixes were already made in d0b3e325; their real-command counterexamples pass again in the complete fixed-source Native baseline.

1. Stack transaction identity: the public/general validator previously did not establish the required blue shell instance relation. The card is deliberately freshened, so literal source-card ID equality is incorrect. Both blue and black call the shared matcher. It requires the full fresh-card snapshot, source UID < transaction UID and transaction UID + 1 == frame UID. The deploy/reveal counterexamples replace only the actual transaction UID with the retired source UID and require restoration rejection; they also reject damage-only snapshot tampering.

`rust-game/src/pack_a_contest.rs:295`
```rust
295:     pub(crate) fn pack_a_transaction_matches(&self, f: &ResolutionFrame, c: &Card) -> bool {
296:         let mut expected = f.source.card.clone();
297:         // Mirror existing fresh() card fields without allocating identities or
298:         // changing payment, RNG, modifiers, or the transaction itself.
299:         expected.id = c.id.clone(); expected.face_down = false;
300:         expected.advancement_markers = 0; expected.named_character = None; expected.chosen_number = None; expected.chosen_subtype = None;
301:         expected.time_markers = 0; expected.charge_markers = 0; expected.permanent_combat_markers = 0;
302:         expected.permanent_defense_markers = 0; expected.lock_markers = 0;
303:         // fresh() allocates the transaction after its original source, and
304:         // push_stack() allocates the immediately following frame UID. Checking
305:         // only the interval would still admit a retired UID after the source.
306:         self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&c.id))
307:             .zip(self.mystic_instance_number(&f.frame_id))
308:             .is_some_and(|((source, transaction), frame)| source < transaction && transaction.checked_add(1) == Some(frame))
309:             && *c == expected
310:     }
```

Call sites: next20_blue_vampires.rs:988, next20_black_wraiths.rs:770. Counterexamples: next20_blue_core_tests.rs:394–427.

`rust-game/src/next20_blue_core_tests.rs:404`
```rust
404:     let item = &g.stack[0];
405:     let frame = item.frame.as_ref().unwrap();
406:     let transaction = item.card.as_ref().unwrap();
407:     // The engine deliberately freshens the card between source and stack. It
408:     // must retain the *particular* next-before-frame ID, not literal equality.
409:     assert_ne!(transaction.id, frame.source.card.id);
410:     assert_eq!(g.mystic_instance_number(&transaction.id).unwrap() + 1,
411:         g.mystic_instance_number(&frame.frame_id).unwrap());
412:     let original = serde_json::to_value(&g).unwrap();
413:     let mut bad = original.clone();
414:     bad["stack"][0]["card"]["id"] = source.into();
415:     assert!(Game::from_persisted(&bad.to_string()).is_err(), "retired source ID must not replace the actual paid transaction");
416:     let mut bad = original;
417:     bad["stack"][0]["card"]["damage"] = 1.into();
418:     assert!(Game::from_persisted(&bad.to_string()).is_err(), "transaction must preserve the complete fresh-card snapshot");
419:     restore(&mut g);
```

2. BQ034 shown completion only clears its own post-frame. It queues no continuation. The shared mystic_after_next advances next_shown and queues one MysticContinue before dispatch; the later shown WM061 mills once. Both no-target and no-blood tests require exactly one history, only seat 2 grave +1, no pending/stack/continuation left. The target branch rejects duplicated/missing continuation and replayed choice.

`rust-game/src/next20_blue_vampires.rs:1230`
```rust
1230:     fn bq034_complete_shown(&mut self, f: &ResolutionFrame) {
1231:         if f.ability_key == SHOWN {
1232:             if let Some(crate::mystic_contest::MysticContest {
1233:                 phase: MysticPhase::After { post_frame, .. },
1234:                 ..
1235:             }) = &mut self.mystic_contest
1236:             {
1237:                 if post_frame
1238:                     .as_ref()
1239:                     .is_some_and(|p| p.frame_id == f.frame_id)
1240:                 {
1241:                     *post_frame = None;
1242:                 }
1243:             }
1244:         }
1245:     }
```

`rust-game/src/mystic_contest.rs:371`
```rust
371:     fn mystic_after_next(&mut self) -> RuleResult<()> {
372:         let state = self.mystic_contest.as_ref().ok_or("隐秘对抗后置效果缺少原状态")?.clone();
373:         let MysticPhase::After { outcome, next_shown, .. } = state.phase else { return Err("非隐秘对抗后置阶段".into()); };
374:         if let Some(shown) = outcome.shown.get(next_shown) {
375:             let MysticPhase::After { next_shown: cursor, .. } = &mut self.mystic_contest.as_mut().unwrap().phase else { unreachable!() };
376:             *cursor += 1;
377:             self.effects.push_front(Effect::MysticContinue { contest_id: state.contest_id });
378:             let post = matches!(shown.card.definition.as_str(), "WM061" | "BQ081").then(|| ResolutionFrame { resource_payments: vec![],
```

`rust-game/src/mystic_contest.rs:396`
```rust
396:         if state.parent.is_some() {
397:             self.mystic_contest.as_mut().unwrap().phase = MysticPhase::Finishing { outcome:outcome.clone() };
398:         } else {
399:             self.mystic_contest = None;
400:         }
401:         self.pack_a_mystic_after(&state.frame, &outcome)?;
402:         self.resolve_frame(*state.frame)
403:     }
```

Counterexamples: next20_blue_core_review_no_target_continues_later_shown_exactly_once and next20_blue_core_review_no_blood_continues_later_shown_exactly_once; assertions at next20_blue_core_tests.rs:430–492.

`rust-game/src/next20_blue_core_tests.rs:476`
```rust
476:     assert_eq!(saw_target, with_target);
477:     assert!(g.mystic_contest.is_none());
478:     assert!(g.pending.is_none() && g.stack.is_empty());
479:     assert!(!g.effects.iter().any(|e| matches!(e, Effect::MysticContinue { .. } | Effect::MysticFinish { .. })));
480:     assert_eq!(g.mystic_history.len(), 1);
481:     for (seat,(p,before)) in g.players.iter().zip(grave_before).enumerate() {
482:         assert_eq!(p.graveyard.len(), before + usize::from(seat == 2));
483:     }
484:     restore(&mut g);
```

3. BQ079 None capture: commit_owner_grave only captures observers when the arriving printed name is 无名尸体 and its owner is alive (batch3_ghoul.rs:26–39). None means recapture at the physical arrival only for that printed name. The delayed pregrave branch accepts immutable XG25, whose name is not 无名尸体, so the questioned path has no applicable BQ079 observation. Ordinary simultaneous deaths still pass frozen observers. No rule or production change was made for this hypothesis. Existing simultaneous-death and non-corpse tests pass in the complete Native baseline.

The XG25 and CQ25 complete method excerpts remain in the preceding fixed-validation evidence archive; their production bytes are unchanged. See runtime-source-equality.json.
