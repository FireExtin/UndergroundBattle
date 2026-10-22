//! Hidden contests use the existing paid frame, response stack and private
//! Pending choices. They are not ordinary regional icon confrontations.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const COMMIT_KIND: &str = "mystic-commit";
pub const REWARD_KIND: &str = "mystic-reward";
const COMMIT_TITLE: &str = "隐秘对抗：秘密选择一张手牌，全部提交后同时展示";
const REWARD_TITLE: &str = "隐秘对抗胜利：选择本人的秘社标志奖励";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysticShown {
    pub seat: usize,
    pub card: Card,
    pub manor: Option<Attachment>,
    pub vampire_observers: Vec<SourceSnapshot>,
    pub destroyed_regions: Vec<String>,
    pub destruction_markers: u32,
    pub value: u32,
}

/// Public only after every participant has committed. `first_winners` belongs
/// to players, so an observer that enters later cannot reset the first win.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContestOutcome {
    pub bonuses: Vec<MysticBonus>,
    pub contest_id: String,
    pub turn: u32,
    pub frame_id: String,
    pub actor: usize,
    pub source: SourceSnapshot,
    pub ability_key: String,
    pub actor_bonus: u32,
    pub participants: Vec<usize>,
    pub shown: Vec<MysticShown>,
    pub totals: [u32; 2],
    pub winner_team: Option<usize>,
    pub first_winners: Vec<usize>,
}

/// A resolved start modifier retains its whole original frame. The card pack
/// that registers this modifier must validate its proof and exact amount.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysticBonus { pub contest_id: String, pub frame: Box<ResolutionFrame>, pub amount: u32 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MysticReward { Secrecy, Exposure, Decline }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MysticPhase {
    /// The integration hook can enqueue ordinary respondable bonus triggers
    /// before the single MysticContinue effect. Nothing has been moved yet.
    BeforeSelection,
    Selecting { committed: Vec<(usize, String)> },
    Rewards { outcome: Box<ContestOutcome>, awarded: Vec<(usize, MysticReward)>, society_before: Vec<(u32, u32)> },
    After { outcome: Box<ContestOutcome>, next_shown: usize, post_frame: Option<Box<ResolutionFrame>> },
    /// The nested procedure and original frame have completed. Its emitted
    /// trigger/response chain must drain before the parent queue is released.
    Finishing { outcome: Box<ContestOutcome> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysticContest {
    /// An interrupted contest retains its original paid frame and continuation
    /// in the same effects queue. Only the innermost contest can ask a choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<MysticContest>>,
    pub response_base: Vec<String>,
    pub bonuses: Vec<MysticBonus>,
    pub contest_id: String,
    pub turn: u32,
    pub frame: Box<ResolutionFrame>,
    pub actor_bonus: u32,
    pub participants: Vec<usize>,
    pub phase: MysticPhase,
}

pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. }, .. })
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("隐秘对抗只能提交当前本人的原选项，不能修改费用或对抗结果".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn mystic_states(&self) -> impl Iterator<Item = &MysticContest> {
        std::iter::successors(self.mystic_contest.as_ref(), |s| s.parent.as_deref())
    }
    /// Existing card programs append emitted triggers. While a contest is
    /// paused, those new effects belong before its continuation, not behind the
    /// older queue tail. A non-serializable fence identifies only new appends;
    /// removals (elimination), front inserts and entry-order references keep
    /// their existing semantics. No paid frame or effect is copied/replayed.
    pub(crate) fn mystic_effect_scope(&mut self, run: impl FnOnce(&mut Self) -> RuleResult<()>) -> RuleResult<()> {
        if self.mystic_contest.is_none() { return run(self); }
        let barrier = self.effects.iter().find_map(|e| match e {
            Effect::MysticContinue { contest_id } | Effect::MysticFinish { contest_id }
                if self.mystic_states().any(|s| s.contest_id == *contest_id) => Some((contest_id.clone(),matches!(e,Effect::MysticFinish { .. }))),
            _ => None,
        });
        let Some(barrier) = barrier else { return run(self); };
        let depth = self.effects.iter().filter(|e| matches!(e, Effect::MysticQueueFence { .. })).count();
        self.effects.push_back(Effect::MysticQueueFence { depth });
        let result = run(self);
        if let Some(index) = self.effects.iter().position(|e| matches!(e, Effect::MysticQueueFence { depth:d } if *d == depth)) {
            let mut appended = self.effects.split_off(index);
            appended.pop_front();
            let index = self.effects.iter().position(|e| match e {
                Effect::MysticContinue { contest_id } => !barrier.1 && *contest_id == barrier.0,
                Effect::MysticFinish { contest_id } => barrier.1 && *contest_id == barrier.0,
                _ => false,
            });
            if let Some(index) = index {
                let tail = self.effects.split_off(index);
                self.effects.extend(appended);
                self.effects.extend(tail);
            } else if self.status == "playing" {
                return Err("隐秘效果队列丢失原续体边界".into());
            }
        } else if self.status == "playing" {
            return Err("隐秘效果队列丢失临时边界".into());
        }
        result
    }
    pub(crate) fn mystic_instance_valid(&self, id: &str) -> bool {
        self.mystic_instance_number(id).is_some()
    }
    pub(crate) fn mystic_instance_number(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i').and_then(|v| v.parse::<u64>().ok())
            .filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }
    pub(crate) fn mystic_op_bonus(frame: &ResolutionFrame) -> Option<u32> {
        match frame.cursor.checked_sub(1).and_then(|i| frame.steps.get(i)).map(|s| &s.op) {
            Some(Op::MysticContest { actor_bonus }) => Some(*actor_bonus), _ => None,
        }
    }
    pub(crate) fn mystic_contest_start(&mut self, frame: ResolutionFrame, actor_bonus: u32) -> RuleResult<()> {
        if self.mystic_contest.as_ref().is_some_and(|s| !matches!(s.phase,MysticPhase::BeforeSelection | MysticPhase::After { .. } | MysticPhase::Finishing { .. }))
            || Self::mystic_op_bonus(&frame) != Some(actor_bonus)
            || !matches!(frame.guard, GuardState::Accepted) {
            return Err("隐秘对抗必须来自同一已付费、已接纳的执行阶段".into());
        }
        let participants = self.players.iter().filter(|p| !p.eliminated).map(|p| p.seat).collect::<Vec<_>>();
        // No source rule has yet settled the no-hand/no-deck participant. Keep
        // this explicit boundary; never invent a zero-cost display card.
        if participants.iter().any(|s| self.players[*s].hand.is_empty() && self.players[*s].deck.is_empty()) {
            return Err("隐秘对抗参与者无手牌且牌库为空：原规则边界待裁定".into());
        }
        let contest_id = self.id();
        let parent = self.mystic_contest.take().map(Box::new);
        self.mystic_contest = Some(MysticContest { contest_id: contest_id.clone(), turn: self.turn,
            frame: Box::new(frame), actor_bonus, participants, phase: MysticPhase::BeforeSelection,
            parent, bonuses: vec![], response_base: self.stack.iter().map(|s| s.id.clone()).collect() });
        // Existing lower stack items are not responses to this contest. Start
        // hooks, their responses and all resulting triggers finish first.
        let tail = std::mem::take(&mut self.effects);
        self.mystic_start_observers(&contest_id)?;
        self.effects.push_back(Effect::MysticContinue { contest_id:contest_id.clone() });
        if self.mystic_contest.as_ref().unwrap().parent.is_some() {
            self.effects.push_back(Effect::MysticFinish { contest_id });
        }
        self.effects.extend(tail);
        Ok(())
    }
    pub(crate) fn mystic_responses_pending(&self) -> bool {
        self.mystic_contest.as_ref().is_some_and(|s| matches!(s.phase,MysticPhase::BeforeSelection | MysticPhase::After { .. } | MysticPhase::Finishing { .. })
            && self.stack.iter().any(|f| !s.response_base.contains(&f.id)))
    }
    #[allow(dead_code)] // Integration endpoint consumed by pack B.
    pub(crate) fn mystic_add_bonus(&mut self, frame: ResolutionFrame, amount: u32) -> RuleResult<()> {
        let s = self.mystic_contest.as_ref().ok_or("加值缺少原隐秘对抗")?;
        if !matches!(s.phase,MysticPhase::BeforeSelection) || !s.participants.contains(&frame.actor)
            || s.bonuses.iter().any(|b| b.frame.frame_id == frame.frame_id) { return Err("隐秘加值仅能在原对抗开始阶段消费一次".into()); }
        let bonus = MysticBonus { contest_id: s.contest_id.clone(), frame: Box::new(frame), amount };
        self.mystic_validate_bonus(&bonus)?;
        self.mystic_contest.as_mut().unwrap().bonuses.push(bonus); Ok(())
    }
    pub(crate) fn mystic_contest_resume(&mut self, id: &str) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().ok_or("隐秘对抗续体缺少原状态")?;
        if state.contest_id != id || self.mystic_responses_pending() {
            return Err("隐秘对抗必须等待原触发与响应结束".into());
        }
        if state.contest_id == id && matches!(state.phase, MysticPhase::After { .. }) {
            return self.mystic_after_next();
        }
        if !matches!(state.phase, MysticPhase::BeforeSelection) {
            return Err("隐秘对抗必须等待原开始触发与响应结束，且只移入顶牌一次".into());
        }
        let participants = state.participants.iter().copied().filter(|s| !self.players[*s].eliminated).collect::<Vec<_>>();
        if participants.iter().any(|s| self.players[*s].hand.is_empty() && self.players[*s].deck.is_empty()) {
            return Err("隐秘对抗参与者无手牌且牌库为空：原规则边界待裁定".into());
        }
        self.mystic_contest.as_mut().unwrap().participants = participants.clone();
        self.mystic_contest.as_mut().unwrap().bonuses.retain(|b| participants.contains(&b.frame.actor));
        for seat in participants {
            if !self.players[seat].deck.is_empty() {
                let c = self.players[seat].deck.remove(0);
                let c = self.fresh(c);
                self.players[seat].hand.push(c);
            }
        }
        self.mystic_contest.as_mut().unwrap().phase = MysticPhase::Selecting { committed: vec![] };
        self.mystic_next_commit()
    }
    fn mystic_next_commit(&mut self) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().ok_or("缺少隐秘对抗")?;
        let MysticPhase::Selecting { committed } = &state.phase else { return Err("非秘密选牌阶段".into()); };
        if committed.len() == state.participants.len() { return self.mystic_reveal_all(); }
        let seat = state.participants[committed.len()];
        let options = self.players[seat].hand.iter().map(|c| self.option(c, seat, None, None)).collect();
        let frame = state.frame.clone(); let contest_id = state.contest_id.clone();
        self.choice(seat, COMMIT_KIND, COMMIT_TITLE.into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame, choice: FrameChoice::MysticCommit { contest_id } });
        Ok(())
    }
    pub(crate) fn mystic_commit(&mut self, frame: &ResolutionFrame, id: &str, seat: usize, selected: &[String]) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().ok_or("原隐秘对抗已结束")?;
        let MysticPhase::Selecting { committed } = &state.phase else { return Err("已过秘密提交阶段".into()); };
        if id != state.contest_id || !same(frame, state.frame.as_ref()) || state.participants.get(committed.len()) != Some(&seat)
            || selected.len() != 1 || !self.players[seat].hand.iter().any(|c| c.id == selected[0]) {
            return Err("只能由当前参与者提交其一张原手牌".into());
        }
        let MysticPhase::Selecting { committed } = &mut self.mystic_contest.as_mut().unwrap().phase else { unreachable!() };
        committed.push((seat, selected[0].clone()));
        self.mystic_next_commit()
    }
    fn mystic_printed_bonus(shown: &MysticShown) -> u32 {
        match shown.card.definition.as_str() {
            "WM021" => 5,
            "WM032" if !shown.destroyed_regions.is_empty() || shown.destruction_markers > 0 => 3,
            "WM061" => 2,
            _ => 0,
        }
    }
    fn mystic_value(&self, shown: &MysticShown, actor: usize, actor_bonus: u32, bonuses: &[MysticBonus]) -> RuleResult<u32> {
        let printed = catalog::catalog().cards.iter().find(|c| c.id == shown.card.definition).ok_or("展示牌未登记")?;
        let vampire = printed.subtypes.iter().any(|s| s == "吸血鬼")
            || shown.manor.is_some() && printed.kind == "character" && printed.subtypes.iter().any(|s| s == "人类");
        if !vampire && !shown.vampire_observers.is_empty() { return Err("非吸血鬼展示牌不能使用秘闻记录官加值".into()); }
        let added = bonuses.iter().filter(|b| b.frame.actor == shown.seat).try_fold(0u32, |a,b| a.checked_add(b.amount)).ok_or("隐秘加值溢出")?;
        printed.cost.checked_add(Self::mystic_printed_bonus(shown))
            .and_then(|n| n.checked_add(self.mystic_extra_shown_bonus(shown).ok()?))
            .and_then(|n| n.checked_add(added))
            .and_then(|n| n.checked_add(if shown.seat == actor { actor_bonus } else { 0 }))
            .and_then(|n| n.checked_add((shown.vampire_observers.len() as u32).checked_mul(2)?))
            .ok_or_else(|| "隐秘对抗数值超出表示范围".into())
    }
    fn mystic_reveal_all(&mut self) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().unwrap().clone();
        let MysticPhase::Selecting { committed } = &state.phase else { unreachable!() };
        let mut shown = vec![]; let mut totals = [0u32; 2];
        for (seat, id) in committed {
            let c = self.players[*seat].hand.iter().find(|c| c.id == *id).ok_or("秘密承诺的原手牌已失效")?.clone();
            let vampire = self.current_subtypes(&c).iter().any(|s| s == "吸血鬼");
            let mut vampire_observers = vec![];
            if vampire {
                for (r, region) in self.regions.iter().enumerate().filter(|(_,r)| !r.vacant) {
                    for observer in &region.cards {
                        if observer.definition == "WM021" && !observer.face_down && observer.controller == *seat {
                            vampire_observers.push(self.source_snapshot(observer, Some(r)));
                        }
                    }
                }
            }
            let manor = if self.xq15_owner_aura(&c) { self.attachments.iter().find(|a| a.card.definition == "XQ15" && !a.card.face_down
                && a.card.controller == c.owner && self.society_host_seat(&a.host_id) == Some(c.owner) && self.attachment_host_valid(a)).cloned() } else { None };
            let mut item = MysticShown { seat: *seat, card: c, manor, vampire_observers, value: 0,
                destroyed_regions: self.regions.iter().filter(|r| !r.vacant && r.destroyed).map(|r| r.card.id.clone()).collect(),
                destruction_markers: self.players[*seat].destruction_markers };
            item.value = self.mystic_value(&item, state.frame.actor, state.actor_bonus, &state.bonuses)?;
            totals[self.team(*seat)] = totals[self.team(*seat)].checked_add(item.value).ok_or("隐秘队伍合计溢出")?;
            shown.push(item);
        }
        let winner_team = if totals[0] == totals[1] { None } else { Some(usize::from(totals[1] > totals[0])) };
        let first_winners = state.participants.iter().copied().filter(|seat| winner_team == Some(self.team(*seat))
            && !self.mystic_history.iter().any(|o| o.turn == self.turn && o.winner_team == Some(self.team(*seat)) && o.participants.contains(seat))).collect();
        let outcome = ContestOutcome { bonuses: state.bonuses.clone(), contest_id: state.contest_id.clone(), turn: self.turn, frame_id: state.frame.frame_id.clone(),
            actor: state.frame.actor, source: state.frame.source.clone(), ability_key: state.frame.ability_key.clone(), actor_bonus: state.actor_bonus,
            participants: state.participants.clone(), shown, totals, winner_team, first_winners };
        // All private commitments are now complete. No earlier log contains a
        // submitted card ID, definition, cost or a teammate's private choice.
        for s in &outcome.shown { self.note(format!("{} 隐秘对抗展示 {}，数值 {}", self.players[s.seat].name, catalog::card(&s.card.definition).name, s.value)); }
        self.note(format!("隐秘对抗：团队1 {}，团队2 {}；{}", totals[0], totals[1], winner_team.map_or("平局，无胜方".into(), |t| format!("团队{}获胜",t+1))));
        let mut shuffled = BTreeSet::new();
        for s in &outcome.shown {
            let index = self.players[s.seat].hand.iter().position(|c| c.id == s.card.id).unwrap();
            let c = self.players[s.seat].hand.remove(index); let c = self.reset_zone_card(c); let owner = c.owner;
            self.players[owner].deck.push(c); shuffled.insert(owner);
        }
        for owner in shuffled { self.shuffle_player(owner); }
        self.mystic_history.push(outcome.clone());
        let previous = self.effects.len();
        self.pack_a_observe_mystic_win(&outcome);
        self.mystic_win_observers(&outcome)?;
        self.batch3_register_entry_order(&outcome.contest_id, previous);
        let society_before = self.players.iter().map(|p| (p.society_zone.exposure,p.society_zone.secrecy)).collect();
        self.mystic_contest.as_mut().unwrap().phase = MysticPhase::Rewards { outcome: Box::new(outcome), awarded: vec![], society_before };
        self.mystic_next_reward()
    }
    pub(crate) fn mystic_opposite(&self, seat: usize) -> usize { if self.mode == "duel" { 1-seat } else { seat ^ 2 } }
    fn mystic_reward_options(&self, seat: usize) -> Vec<ChoiceOption> {
        let mut out = vec![ChoiceOption { id: "secrecy".into(), label: "本人的秘社放置隐秘1".into(), card: None }];
        let enemy = self.mystic_opposite(seat);
        if !self.players[enemy].eliminated { out.push(ChoiceOption { id: "exposure".into(), label: format!("正对面的 {} 秘社放置暴露1",self.players[enemy].name), card: None }); }
        out
    }
    fn mystic_next_reward(&mut self) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().unwrap();
        let MysticPhase::Rewards { outcome, awarded, .. } = &state.phase else { return Err("非隐秘奖励阶段".into()); };
        let winners = outcome.participants.iter().copied().filter(|s| outcome.winner_team == Some(self.team(*s))).collect::<Vec<_>>();
        if awarded.len() == winners.len() { return self.mystic_finish(); }
        let seat = winners[awarded.len()]; let options = self.mystic_reward_options(seat);
        let frame = state.frame.clone(); let contest_id = state.contest_id.clone();
        self.choice(seat, REWARD_KIND, REWARD_TITLE.into(), options, 0, 1, None,
            ChoiceResolution::Frame { frame, choice: FrameChoice::MysticReward { contest_id } });
        Ok(())
    }
    pub(crate) fn mystic_reward(&mut self, frame: &ResolutionFrame, id: &str, seat: usize, selected: &[String]) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().ok_or("原隐秘对抗已结束")?;
        let MysticPhase::Rewards { outcome, awarded, .. } = &state.phase else { return Err("非隐秘奖励阶段".into()); };
        let winners = outcome.participants.iter().copied().filter(|s| outcome.winner_team == Some(self.team(*s))).collect::<Vec<_>>();
        if state.contest_id != id || !same(frame,state.frame.as_ref()) || winners.get(awarded.len()) != Some(&seat)
            || selected.len() > 1 || selected.first().is_some_and(|id| !self.mystic_reward_options(seat).iter().any(|o| o.id == *id)) {
            return Err("隐秘胜利奖励只由原胜方本人领取一次".into());
        }
        let reward = match selected.first().map(String::as_str) {
            Some("secrecy") => { self.add_society_markers(seat,0,1); MysticReward::Secrecy },
            Some("exposure") => { self.add_society_markers(self.mystic_opposite(seat),1,0); MysticReward::Exposure },
            None => MysticReward::Decline,
            _ => return Err("未知隐秘对抗奖励".into()),
        };
        let MysticPhase::Rewards { awarded, .. } = &mut self.mystic_contest.as_mut().unwrap().phase else { unreachable!() };
        awarded.push((seat,reward)); self.mystic_next_reward()
    }
    fn mystic_finish(&mut self) -> RuleResult<()> {
        let state = self.mystic_contest.as_mut().ok_or("隐秘对抗不可重复结束")?;
        let MysticPhase::Rewards { outcome, .. } = &state.phase else { return Err("隐秘对抗尚未展示并分配奖励".into()); };
        state.phase = MysticPhase::After { outcome: outcome.clone(), next_shown: 0, post_frame: None };
        self.mystic_after_next()
    }
    fn mystic_after_next(&mut self) -> RuleResult<()> {
        let state = self.mystic_contest.as_ref().ok_or("隐秘对抗后置效果缺少原状态")?.clone();
        let MysticPhase::After { outcome, next_shown, .. } = state.phase else { return Err("非隐秘对抗后置阶段".into()); };
        if let Some(shown) = outcome.shown.get(next_shown) {
            let MysticPhase::After { next_shown: cursor, .. } = &mut self.mystic_contest.as_mut().unwrap().phase else { unreachable!() };
            *cursor += 1;
            self.effects.push_front(Effect::MysticContinue { contest_id: state.contest_id });
            let post = (shown.card.definition == "WM061").then(|| ResolutionFrame {
                frame_id: self.id(), ability_key: crate::pack_a_contest::EARTH_SHOWN.into(), actor: shown.seat,
                source: self.source_snapshot(&shown.card, None), targets: vec![], already_paid: vec![],
                guard: GuardState::Accepted, cursor: 1,
                steps: vec![Step { context: shown.seat, op: Op::MoveDeckTopToGraveyard { player: PlayerRef::Actor, count: 1 } }],
                chosen_region: None,
            });
            let MysticPhase::After { post_frame, .. } = &mut self.mystic_contest.as_mut().unwrap().phase else { unreachable!() };
            *post_frame = post.clone().map(Box::new);
            self.mystic_effect_scope(|g| {
                if !g.players[shown.seat].eliminated {
                    if let Some(frame) = post { g.mill_cards_with_cause(shown.seat, 1, Some(&frame))?; }
                    else { g.mystic_shown_after(&outcome, next_shown)?; }
                }
                Ok(())
            })?;
            return Ok(());
        }
        if state.parent.is_some() {
            self.mystic_contest.as_mut().unwrap().phase = MysticPhase::Finishing { outcome:outcome.clone() };
        } else {
            self.mystic_contest = None;
        }
        self.pack_a_mystic_after(&state.frame, &outcome)?;
        self.resolve_frame(*state.frame)
    }
    pub(crate) fn mystic_finish_nested(&mut self, id: &str) -> RuleResult<()> {
        let state=self.mystic_contest.as_ref().ok_or("嵌套隐秘结束边界缺少原状态")?;
        if state.contest_id != id || state.parent.is_none() || !matches!(state.phase,MysticPhase::Finishing { .. })
            || self.mystic_responses_pending() {
            return Err("嵌套隐秘对抗须完成本层后置及触发链后才能恢复父层".into());
        }
        self.mystic_contest=self.mystic_contest.take().unwrap().parent.map(|p|*p);
        Ok(())
    }
    pub(crate) fn mystic_revealed_hands(&self, viewer: usize) -> Vec<RevealedHandView> {
        let Some(MysticContest { phase: MysticPhase::Rewards { outcome, .. } | MysticPhase::After { outcome, .. } | MysticPhase::Finishing { outcome }, .. }) = &self.mystic_contest else { return vec![]; };
        outcome.shown.iter().map(|s| RevealedHandView { player_id: player_id(s.seat), cards: vec![self.card_view(&s.card,viewer,None,None)] }).collect()
    }
    pub(crate) fn validate_mystic_outcome(&self, o: &ContestOutcome) -> RuleResult<()> {
        if !self.mystic_instance_valid(&o.contest_id) || !self.mystic_instance_valid(&o.frame_id) || o.turn != self.turn
            || o.actor >= self.players.len() || o.source.card.controller != o.actor || o.participants.is_empty()
            || o.participants.windows(2).any(|w| w[0] >= w[1]) || o.participants.iter().any(|s| *s >= self.players.len())
            || self.players.iter().any(|p| !p.eliminated && !o.participants.contains(&p.seat))
            || o.shown.iter().map(|s| s.seat).collect::<Vec<_>>() != o.participants
            || !crate::pack_a_contest::admitted_start(&o.source.card.definition, &o.ability_key, o.actor_bonus) {
            return Err("隐秘对抗记录的来源、参与者或程序无效".into());
        }
        self.mobility_immutable_identity(&o.source.card)?;
        self.mystic_validate_bonuses(&o.contest_id, &o.participants, &o.bonuses)?;
        let mut ids = BTreeSet::new(); let mut totals = [0u32; 2];
        for shown in &o.shown {
            if !self.mystic_instance_valid(&shown.card.id) || !ids.insert(&shown.card.id) || shown.card.face_down
                || shown.card.owner >= self.players.len() || shown.card.controller != shown.seat
                || shown.value != self.mystic_value(shown,o.actor,o.actor_bonus,&o.bonuses)?
                || shown.destroyed_regions.iter().collect::<BTreeSet<_>>().len() != shown.destroyed_regions.len()
                || shown.destroyed_regions.iter().any(|id| !self.mystic_instance_valid(id)) {
                return Err("隐秘展示身份、加值或队伍贡献被改写".into());
            }
            self.mobility_immutable_identity(&shown.card)?;
            if let Some(a) = &shown.manor {
                if a.card.definition != "XQ15" || a.card.face_down || a.card.controller != shown.card.owner
                    || a.host_id != format!("society:p{}",shown.card.owner) || !self.mystic_instance_valid(&a.card.id) {
                    return Err("展示牌的庄园类别见证无效".into());
                }
                self.mobility_immutable_identity(&a.card)?;
            }
            let mut observers = BTreeSet::new();
            for s in &shown.vampire_observers {
                if s.card.definition != "WM021" || s.card.face_down || s.card.controller != shown.seat
                    || !self.mystic_instance_valid(&s.card.id) || !observers.insert(&s.card.id)
                    || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id))
                    || s.play_source.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.death_echo.is_some() {
                    return Err("秘闻记录官须保留各个本人正面场内来源见证".into());
                }
                self.mobility_immutable_identity(&s.card)?;
            }
            totals[self.team(shown.seat)] = totals[self.team(shown.seat)].checked_add(shown.value).ok_or("隐秘对抗合计溢出")?;
        }
        let winner = if totals[0] == totals[1] { None } else { Some(usize::from(totals[1] > totals[0])) };
        if o.totals != totals || o.winner_team != winner { return Err("隐秘对抗胜负不符合实际展示数值".into()); }
        Ok(())
    }
    fn mystic_validate_bonuses(&self, id: &str, seats: &[usize], bonuses: &[MysticBonus]) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for b in bonuses {
            if b.contest_id != id || !seats.contains(&b.frame.actor) || !seen.insert(&b.frame.frame_id) { return Err("隐秘加值收据重复或对抗不符".into()); }
            self.mystic_validate_bonus(b)?;
        }
        Ok(())
    }
    pub(crate) fn validate_mystic_state(&self) -> RuleResult<()> {
        let mut contests = BTreeSet::new(); let mut won = BTreeSet::new();
        for o in &self.mystic_history {
            self.validate_mystic_outcome(o)?;
            let first = o.participants.iter().copied().filter(|s| o.winner_team == Some(self.team(*s)) && !won.contains(s)).collect::<Vec<_>>();
            if !contests.insert(o.contest_id.as_str()) || o.first_winners != first { return Err("隐秘对抗记录重复或首次获胜记录被改写".into()); }
            won.extend(o.participants.iter().copied().filter(|s| o.winner_team == Some(self.team(*s))));
        }
        let continuations = self.effects.iter().filter_map(|e| match e {
            Effect::MysticContinue { contest_id } => Some((contest_id.as_str(),false)),
            Effect::MysticFinish { contest_id } => Some((contest_id.as_str(),true)),
            _ => None,
        }).collect::<Vec<_>>();
        let pending = self.pending.as_ref().filter(|p| matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. }, .. }));
        let states = self.mystic_states().collect::<Vec<_>>();
        if states.is_empty() {
            return if continuations.is_empty() && pending.is_none() { Ok(()) } else { Err("隐秘对抗待选或续体缺少原协议".into()) };
        }
        if self.status != "playing" { return Err("终局不得残留活动隐秘对抗".into()); }
        let expected = states.iter().flat_map(|s| {
            let resume=matches!(s.phase,MysticPhase::BeforeSelection | MysticPhase::After { .. }).then_some((s.contest_id.as_str(),false));
            let finish=s.parent.is_some().then_some((s.contest_id.as_str(),true));
            resume.into_iter().chain(finish)
        }).collect::<Vec<_>>();
        if continuations != expected { return Err("隐秘嵌套续体必须按内层到外层各保留一次".into()); }
        let mut ids = BTreeSet::new();
        for (index,state) in states.iter().enumerate() {
            if !ids.insert(state.contest_id.as_str()) || !ids.insert(state.frame.frame_id.as_str())
                || !self.mystic_instance_number(&state.frame.frame_id).zip(self.mystic_instance_number(&state.contest_id))
                    .is_some_and(|(frame,contest)| frame < contest)
                || index > 0 && (!matches!(state.phase,MysticPhase::BeforeSelection | MysticPhase::After { .. } | MysticPhase::Finishing { .. })
                    || !self.mystic_instance_number(&state.contest_id).zip(self.mystic_instance_number(&states[index-1].frame.frame_id))
                        .is_some_and(|(parent,child_frame)| parent < child_frame)) {
                return Err("隐秘嵌套必须保留各层原帧、阶段及开始先后关系".into());
            }
            self.mystic_validate_contest(state,index==0,&contests)?;
        }
        Ok(())
    }
    fn mystic_validate_contest(&self, state: &MysticContest, active: bool, contests: &BTreeSet<&str>) -> RuleResult<()> {
        let pending = self.pending.as_ref().filter(|p| active && matches!(p.resolution,
            ChoiceResolution::Frame { choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. }, .. }));
        if !self.mystic_instance_valid(&state.contest_id) || state.turn != self.turn || state.frame.actor >= self.players.len()
            || !matches!(state.frame.guard, GuardState::Accepted) || Self::mystic_op_bonus(&state.frame) != Some(state.actor_bonus)
            || state.participants.windows(2).any(|w| w[0] >= w[1]) || state.participants.iter().any(|s| *s >= self.players.len())
            || if matches!(state.phase,MysticPhase::BeforeSelection | MysticPhase::After { .. } | MysticPhase::Finishing { .. }) {
                self.players.iter().any(|p| !p.eliminated && !state.participants.contains(&p.seat))
            } else { state.participants != self.players.iter().filter(|p| !p.eliminated).map(|p| p.seat).collect::<Vec<_>>() }
            || self.stack.iter().any(|s| s.id == state.frame.frame_id)
            || self.effects.iter().any(|e| matches!(e, Effect::Frame { frame } if frame.frame_id == state.frame.frame_id)) {
            return Err("隐秘对抗阶段、原执行帧或存活参与者无效".into());
        }
        self.validate_pack_a_frame(&state.frame)?;
        self.mystic_validate_bonuses(&state.contest_id, &state.participants, &state.bonuses)?;
        if state.response_base.iter().any(|id| !self.mystic_instance_number(id).zip(self.mystic_instance_number(&state.frame.frame_id))
                .is_some_and(|(base,frame)| base < frame))
            || state.response_base.iter().collect::<BTreeSet<_>>().len() != state.response_base.len() { return Err("隐秘开始响应基线被改写".into()); }
        match &state.phase {
            MysticPhase::Finishing { outcome } => {
                if state.parent.is_none() || pending.is_some()
                    || self.mystic_history.iter().find(|o|o.contest_id==state.contest_id).is_none_or(|o|!same(o,outcome.as_ref()))
                    || outcome.contest_id != state.contest_id || outcome.actor_bonus != state.actor_bonus
                    || outcome.actor != state.frame.actor || outcome.ability_key != state.frame.ability_key || outcome.participants != state.participants
                    || outcome.frame_id != state.frame.frame_id || !same(&outcome.source,&state.frame.source) || !same(&outcome.bonuses,&state.bonuses) {
                    return Err("嵌套隐秘结束边界必须绑定已完成的本层结果".into());
                }
                return Ok(());
            },
            MysticPhase::BeforeSelection => {
                if pending.is_some() || contests.contains(state.contest_id.as_str()) { return Err("隐秘开始续体重复或过早公开".into()); }
                return Ok(());
            },
            MysticPhase::After { outcome, next_shown, post_frame } => {
                if pending.is_some() || *next_shown == 0 || *next_shown > outcome.shown.len()
                    || self.mystic_history.iter().find(|o| o.contest_id == state.contest_id).is_none_or(|o| !same(o,outcome.as_ref()))
                    || outcome.contest_id != state.contest_id || outcome.actor_bonus != state.actor_bonus
                    || outcome.actor != state.frame.actor || outcome.ability_key != state.frame.ability_key || outcome.participants != state.participants
                    || outcome.frame_id != state.frame.frame_id || !same(&outcome.source,&state.frame.source) || !same(&outcome.bonuses,&state.bonuses) {
                    return Err("隐秘展示后置效果的原结果或消费游标被改写".into());
                }
                if let Some(frame) = post_frame { self.validate_pack_a_frame(frame)?; }
                return Ok(());
            },
            MysticPhase::Selecting { committed } => {
                if !active || contests.contains(state.contest_id.as_str()) || committed.len() >= state.participants.len()
                    || committed.iter().zip(&state.participants).any(|((s,id),expected)| s != expected || !self.players[*s].hand.iter().any(|c| c.id == *id)) {
                    return Err("隐秘选牌必须保留私有承诺的原玩家与原手牌".into());
                }
            },
            MysticPhase::Rewards { outcome, awarded, society_before } => {
                if !active || self.mystic_history.last().is_none_or(|last| !same(last,outcome.as_ref()))
                    || outcome.contest_id != state.contest_id || outcome.frame_id != state.frame.frame_id || outcome.actor_bonus != state.actor_bonus
                    || !same(&outcome.source,&state.frame.source) || !same(&outcome.bonuses,&state.bonuses) || outcome.participants != state.participants
                    || society_before.len() != self.players.len() { return Err("隐秘奖励必须绑定本次完整结果与原帧".into()); }
                let winners = outcome.participants.iter().copied().filter(|s| outcome.winner_team == Some(self.team(*s))).collect::<Vec<_>>();
                if awarded.len() >= winners.len() || awarded.iter().zip(&winners).any(|((s,_),expected)| s != expected) { return Err("隐秘奖励不可重复或换人领取".into()); }
                let mut expected = society_before.clone();
                for (s,r) in awarded { let (seat,sun,moon) = match r { MysticReward::Secrecy => (*s,0u64,1u64), MysticReward::Exposure => (self.mystic_opposite(*s),1,0), MysticReward::Decline => continue };
                    let a = u64::from(expected[seat].0)+sun; let b = u64::from(expected[seat].1)+moon;
                    expected[seat] = (a.saturating_sub(b).min(u64::from(u32::MAX)) as u32,b.saturating_sub(a).min(u64::from(u32::MAX)) as u32);
                }
                if expected != self.players.iter().map(|p| (p.society_zone.exposure,p.society_zone.secrecy)).collect::<Vec<_>>() { return Err("隐秘标志奖励与一次领取收据不符".into()); }
            },
        }
        let p = pending.ok_or("隐秘对抗缺少当前私有待选")?;
        let ChoiceResolution::Frame { frame, choice } = &p.resolution else { unreachable!() };
        let (seat,kind,title,options,min) = match (&state.phase,choice) {
            (MysticPhase::Selecting { committed },FrameChoice::MysticCommit { contest_id }) if *contest_id == state.contest_id => {
                let s = state.participants[committed.len()]; (s,COMMIT_KIND,COMMIT_TITLE,self.players[s].hand.iter().map(|c| self.option(c,s,None,None)).collect(),1)
            },
            (MysticPhase::Rewards { outcome, awarded, .. },FrameChoice::MysticReward { contest_id }) if *contest_id == state.contest_id => {
                let s = outcome.participants.iter().copied().filter(|s| outcome.winner_team == Some(self.team(*s))).nth(awarded.len()).unwrap();
                (s,REWARD_KIND,REWARD_TITLE,self.mystic_reward_options(s),0)
            },
            _ => return Err("隐秘待选类型与原阶段不符".into()),
        };
        let expected = Choice { id: p.choice.id.clone(),kind:kind.into(),title:title.into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(seat),options,preview_cards:vec![],min:Some(min),max:Some(1),amount:None,allow_decline:Some(min==0) };
        if p.seat != seat || !self.mystic_instance_valid(&p.choice.id) || !same(frame.as_ref(),state.frame.as_ref()) || !same(&p.choice,&expected) { return Err("隐秘私有待选外壳被改写".into()); }
        Ok(())
    }
}
