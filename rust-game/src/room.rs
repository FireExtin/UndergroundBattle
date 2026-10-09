//! Finite, trusted-time session policy around the unchanged deterministic rules.
//! Hosts authenticate seats and check original receipts before calling this reducer.
use crate::{catalog, engine::RuleResult, model::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INTENT_DURATION_MS: u64 = 5_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCommand {
    pub command_id: String,
    pub expected_version: u64,
    pub action: SessionAction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SessionAction {
    PauseRoom,
    ResumeRoom,
    Game {
        action: Action,
    },
    BeginResponse {
        #[serde(rename = "windowId")]
        window_id: String,
        #[serde(rename = "intentId")]
        intent_id: String,
    },
    PassResponse {
        #[serde(rename = "windowId")]
        window_id: String,
    },
    CancelAndPass {
        #[serde(rename = "windowId")]
        window_id: String,
        #[serde(rename = "intentId")]
        intent_id: String,
    },
    SubmitResponse {
        #[serde(rename = "windowId")]
        window_id: String,
        #[serde(rename = "intentId")]
        intent_id: String,
        action: Action,
    },
}
impl From<Action> for SessionAction {
    fn from(action: Action) -> Self {
        Self::Game { action }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Decision {
    Undecided {
        #[serde(rename = "deadlineMs")]
        deadline_ms: u64,
    },
    Composing {
        #[serde(rename = "intentId")]
        intent_id: String,
    },
    Passed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityWindow {
    pub id: String,
    pub stack_top_id: String,
    pub holder_team: usize,
    pub members: BTreeMap<usize, Decision>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pacing {
    pub window_seq: u64,
    pub last_server_now_ms: u64,
    pub window: Option<PriorityWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause: Option<RoomPause>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomPause {
    pub paused_at_ms: u64,
    pub paused_by: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomEnvelope {
    pub state_schema: u32,
    pub revision: u64,
    pub versions: Versions,
    pub game: Game,
    pub pacing: Pacing,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberView {
    pub player_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseWindowView {
    pub id: String,
    pub stack_top_id: String,
    pub holder_team: usize,
    pub members: Vec<MemberView>,
    pub can_begin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_intent_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomView {
    #[serde(flatten)]
    pub game: View,
    pub server_now_ms: u64,
    pub response_window: Option<ResponseWindowView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause: Option<RoomPause>,
    pub can_pause: bool,
}
impl std::ops::Deref for RoomView {
    type Target = View;
    fn deref(&self) -> &View {
        &self.game
    }
}
impl std::ops::Deref for RoomEnvelope {
    type Target = Game;
    fn deref(&self) -> &Game {
        &self.game
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionEvent {
    Tick {
        server_now_ms: u64,
    },
    Command {
        seat: usize,
        command: RoomCommand,
        server_now_ms: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomTransition {
    pub state: String,
    pub view: RoomView,
    pub version: u64,
    pub seat: usize,
    pub changed: bool,
    pub journal: Vec<SessionEvent>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub window_id: String,
    pub intent_id: String,
    pub draft: Option<Action>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub version: u64,
    pub window_id: String,
    pub intent_id: String,
    pub legal_actions: Vec<LegalAction>,
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_action: Option<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn response_action(action: &Action) -> bool {
    matches!(
        action.kind.as_str(),
        "play" | "activate" | "reveal" | "privilege"
    )
}
fn failure(code: &str, message: impl Into<String>) -> (String, String) {
    (code.into(), message.into())
}
impl RoomEnvelope {
    pub fn from_game(game: Game) -> Self {
        Self {
            state_schema: 3,
            revision: game.version,
            versions: game.versions.clone(),
            game,
            pacing: Pacing::default(),
        }
    }
    pub fn from_persisted(state: &str) -> RuleResult<Self> {
        #[derive(Deserialize)]
        struct Header {
            state_schema: Option<u32>,
            versions: Versions,
        }
        let incompatible = || {
            format!(
                "旧局schema/固定版本与当前{}不兼容；请使用对应旧核，不能静默迁移",
                catalog::ENGINE_VERSION
            )
        };
        let header: Header = serde_json::from_str(state).map_err(|_| incompatible())?;
        if header.state_schema != Some(3)
            || header.versions.rules != catalog::RULES_VERSION
            || header.versions.card_pool != catalog::POOL_VERSION
            || header.versions.engine != catalog::ENGINE_VERSION
        {
            return Err(incompatible());
        }
        let room: Self = serde_json::from_str(state).map_err(|_| "schema3房间状态无效")?;
        if room.pacing.pause.as_ref().is_some_and(|p| {
            room.game.status != "playing"
                || p.paused_by >= room.game.players.len()
                || p.paused_at_ms != room.pacing.last_server_now_ms
        }) {
            return Err("暂停记录与房间状态/冻结时间不一致".into());
        }
        room.game.validate_sealed_cards()?;
        room.game.validate_jz50_search_choice()?;
        room.game.validate_entry_search_choice()?;
        room.game.validate_deck_seal_choice()?;
        if room.game.state_schema != 2
            || room.game.version != room.revision
            || room.game.versions.rules != room.versions.rules
            || room.game.versions.card_pool != room.versions.card_pool
            || room.game.versions.engine != room.versions.engine
            || room.game.players.is_empty()
            || room.game.players.len() > room.game.capacity()
            || !matches!(room.game.mode.as_str(), "duel" | "teams")
            || room
                .game
                .players
                .iter()
                .enumerate()
                .any(|(i, p)| p.seat != i)
        {
            return Err("房间规则状态与revision/身份不一致".into());
        }
        let active_top = room
            .game
            .stack
            .last()
            .filter(|_| room.game.status == "playing" && room.game.pending.is_none());
        match (active_top, &room.pacing.window) {
            (None, None) => {}
            (Some(top), Some(window))
                if window.stack_top_id == top.id
                    && window.holder_team == room.game.priority_team
                    && window.id == format!("response:{}", room.pacing.window_seq)
                    && room.pacing.window_seq > 0 =>
            {
                let living = room
                    .game
                    .players
                    .iter()
                    .filter(|p| !p.eliminated && room.game.team(p.seat) == window.holder_team);
                if living.clone().count() != window.members.len()
                    || living
                        .into_iter()
                        .any(|p| match window.members.get(&p.seat) {
                            Some(Decision::Passed) => !room.game.passed.contains(&p.seat),
                            Some(Decision::Composing { intent_id }) => {
                                room.game.passed.contains(&p.seat)
                                    || intent_id.is_empty()
                                    || intent_id.len() > 128
                            }
                            Some(Decision::Undecided { deadline_ms }) => {
                                room.game.passed.contains(&p.seat)
                                    || *deadline_ms <= room.pacing.last_server_now_ms
                            }
                            None => true,
                        })
                {
                    return Err("房间响应成员状态与规则优先权不一致".into());
                }
            }
            _ => return Err("房间响应窗口与堆顶/优先权不一致".into()),
        }
        Ok(room)
    }
    fn synchronize_window(&mut self, now: u64, force: bool) -> RuleResult<()> {
        let top = self
            .game
            .stack
            .last()
            .filter(|_| self.game.status == "playing" && self.game.pending.is_none());
        let Some(top) = top else {
            self.pacing.window = None;
            return Ok(());
        };
        if !force
            && self.pacing.window.as_ref().is_some_and(|w| {
                w.stack_top_id == top.id && w.holder_team == self.game.priority_team
            })
        {
            return Ok(());
        }
        let deadline = now.checked_add(INTENT_DURATION_MS).ok_or("响应期限溢出")?;
        self.pacing.window_seq = self
            .pacing
            .window_seq
            .checked_add(1)
            .ok_or("响应窗口序号溢出")?;
        let members = self
            .game
            .players
            .iter()
            .filter(|p| !p.eliminated && self.game.team(p.seat) == self.game.priority_team)
            .map(|p| {
                (
                    p.seat,
                    if self.game.passed.contains(&p.seat) {
                        Decision::Passed
                    } else {
                        Decision::Undecided {
                            deadline_ms: deadline,
                        }
                    },
                )
            })
            .collect();
        self.pacing.window = Some(PriorityWindow {
            id: format!("response:{}", self.pacing.window_seq),
            stack_top_id: top.id.clone(),
            holder_team: self.game.priority_team,
            members,
        });
        Ok(())
    }
    fn offers(&self, seat: usize) -> Vec<LegalAction> {
        self.game
            .legal_actions(seat)
            .into_iter()
            .filter(|a| response_action(&a.action))
            .collect()
    }
    pub fn view(&self, seat: usize, server_now_ms: u64) -> RoomView {
        let mut view = self.game.view(seat);
        view.version = self.revision;
        let response_window = self.pacing.window.as_ref().map(|w| {
            let mine = w.members.get(&seat);
            let can_begin = self.pacing.pause.is_none()
                && matches!(mine, Some(Decision::Undecided { .. }))
                && view
                    .legal_actions
                    .iter()
                    .any(|a| response_action(&a.action));
            if !matches!(mine, Some(Decision::Composing { .. })) {
                view.legal_actions.retain(|a| a.action.kind == "pass");
            }
            ResponseWindowView {
                id: w.id.clone(),
                stack_top_id: w.stack_top_id.clone(),
                holder_team: w.holder_team,
                members: w
                    .members
                    .iter()
                    .map(|(seat, d)| MemberView {
                        player_id: format!("p{seat}"),
                        status: match d {
                            Decision::Undecided { .. } => "undecided",
                            Decision::Composing { .. } => "composing",
                            Decision::Passed => "passed",
                        }
                        .into(),
                        deadline_ms: if let Decision::Undecided { deadline_ms } = d {
                            Some(*deadline_ms)
                        } else {
                            None
                        },
                    })
                    .collect(),
                can_begin,
                my_intent_id: if let Some(Decision::Composing { intent_id }) = mine {
                    Some(intent_id.clone())
                } else {
                    None
                },
            }
        });
        if self.pacing.pause.is_some() {
            view.legal_actions.clear();
        }
        RoomView {
            game: view,
            server_now_ms: self.pacing.pause.as_ref().map_or_else(
                || server_now_ms.max(self.pacing.last_server_now_ms),
                |p| p.paused_at_ms,
            ),
            response_window,
            pause: self.pacing.pause.clone(),
            can_pause: self.game.status == "playing",
        }
    }
    fn expire_due(&mut self, now: u64, revision: u64) -> RuleResult<bool> {
        if self.pacing.pause.is_some() {
            return Ok(false);
        }
        let Some(w) = self.pacing.window.clone() else {
            return Ok(false);
        };
        let due: Vec<_> = w
            .members
            .iter()
            .filter_map(|(s, d)| match d {
                Decision::Undecided { deadline_ms } if now >= *deadline_ms => Some(*s),
                _ => None,
            })
            .collect();
        if due.is_empty() {
            return Ok(false);
        }
        for seat in due {
            if self
                .pacing
                .window
                .as_ref()
                .is_none_or(|current| current.id != w.id)
            {
                break;
            }
            self.pacing
                .window
                .as_mut()
                .unwrap()
                .members
                .insert(seat, Decision::Passed);
            self.game
                .apply_at_revision(seat, Action::new("pass"), revision)?;
            self.synchronize_window(now, false)?;
        }
        self.revision = revision;
        self.game.version = revision;
        self.pacing.last_server_now_ms = now;
        Ok(true)
    }
    fn require_member(&self, seat: usize, id: &str) -> Result<&Decision, (String, String)> {
        let w = self
            .pacing
            .window
            .as_ref()
            .filter(|w| w.id == id)
            .ok_or_else(|| failure("window_expired", "响应窗口已经改变或关闭"))?;
        w.members
            .get(&seat)
            .ok_or_else(|| failure("invalid_action", "此座位不持有该响应窗口的优先权"))
    }
    fn commit_action(
        &mut self,
        seat: usize,
        command: &RoomCommand,
        now: u64,
        revision: u64,
    ) -> Result<(), (String, String)> {
        if command.command_id.is_empty() || command.command_id.len() > 128 {
            return Err(failure("invalid_action", "commandId长度须为1至128字节"));
        }
        let strict = matches!(
            command.action,
            SessionAction::Game { .. }
                | SessionAction::SubmitResponse { .. }
                | SessionAction::PauseRoom
                | SessionAction::ResumeRoom
        );
        if (strict && command.expected_version != self.revision)
            || command.expected_version > self.revision
        {
            return Err(failure(
                "version_conflict",
                "房间已更新，请重新报价并确认动作",
            ));
        }
        if self.pacing.pause.is_some() && !matches!(command.action, SessionAction::ResumeRoom) {
            return Err(failure("room_paused", "此桌已暂停，请先恢复对局"));
        }
        match &command.action {
            SessionAction::PauseRoom => {
                if self.game.status != "playing" {
                    return Err(failure("invalid_action", "只有进行中的对局可以暂停"));
                }
                self.pacing.pause = Some(RoomPause {
                    paused_at_ms: now,
                    paused_by: seat,
                });
            }
            SessionAction::ResumeRoom => {
                let pause = self
                    .pacing
                    .pause
                    .as_ref()
                    .ok_or_else(|| failure("invalid_action", "此桌尚未暂停"))?;
                if let Some(window) = &mut self.pacing.window {
                    for decision in window.members.values_mut() {
                        if let Decision::Undecided { deadline_ms } = decision {
                            let remaining = deadline_ms
                                .checked_sub(pause.paused_at_ms)
                                .ok_or_else(|| failure("invalid_action", "暂停响应期限无效"))?;
                            *deadline_ms = now
                                .checked_add(remaining)
                                .ok_or_else(|| failure("invalid_action", "恢复响应期限溢出"))?;
                        }
                    }
                    // A late pre-pause window command cannot enter the resumed window.
                    // Reuse the existing window sequence; preserve composing intents and paid stack.
                    self.pacing.window_seq = self
                        .pacing
                        .window_seq
                        .checked_add(1)
                        .ok_or_else(|| failure("invalid_action", "响应窗口序号溢出"))?;
                    window.id = format!("response:{}", self.pacing.window_seq);
                }
                self.pacing.pause = None;
            }
            SessionAction::Game { action } => {
                if self.pacing.window.is_some() {
                    return Err(failure(
                        "invalid_action",
                        "请先决定是否连锁，再通过响应提交或明确让过",
                    ));
                }
                self.game
                    .apply_at_revision(seat, action.clone(), revision)
                    .map_err(|m| failure("invalid_action", m))?;
                self.synchronize_window(now, true)
                    .map_err(|m| failure("invalid_action", m))?;
            }
            SessionAction::BeginResponse {
                window_id,
                intent_id,
            } => {
                if intent_id.is_empty() || intent_id.len() > 128 {
                    return Err(failure("invalid_action", "intentId长度须为1至128字节"));
                }
                match self.require_member(seat, window_id)? {
                    Decision::Composing {
                        intent_id: original,
                    } if original == intent_id => {}
                    Decision::Undecided { .. } => {}
                    _ => {
                        return Err(failure(
                            "window_expired",
                            "本席已决定或已让过，不能重复开始",
                        ))
                    }
                }
                if self.offers(seat).is_empty() {
                    return Err(failure(
                        "invalid_action",
                        "你当前没有合法响应动作，可明确让过",
                    ));
                }
                self.pacing.window.as_mut().unwrap().members.insert(
                    seat,
                    Decision::Composing {
                        intent_id: intent_id.clone(),
                    },
                );
            }
            SessionAction::PassResponse { window_id } => {
                if !matches!(
                    self.require_member(seat, window_id)?,
                    Decision::Undecided { .. }
                ) {
                    return Err(failure(
                        "window_expired",
                        "本席已决定；编辑中的响应应使用取消并让过",
                    ));
                }
                self.pacing
                    .window
                    .as_mut()
                    .unwrap()
                    .members
                    .insert(seat, Decision::Passed);
                self.game
                    .apply_at_revision(seat, Action::new("pass"), revision)
                    .map_err(|m| failure("invalid_action", m))?;
                self.synchronize_window(now, false)
                    .map_err(|m| failure("invalid_action", m))?;
            }
            SessionAction::CancelAndPass {
                window_id,
                intent_id,
            } => {
                if !matches!(self.require_member(seat, window_id)?, Decision::Composing { intent_id: current } if current == intent_id)
                {
                    return Err(failure(
                        "window_expired",
                        "响应编辑意图已改变，不能撤回已付款声明",
                    ));
                }
                self.pacing
                    .window
                    .as_mut()
                    .unwrap()
                    .members
                    .insert(seat, Decision::Passed);
                self.game
                    .apply_at_revision(seat, Action::new("pass"), revision)
                    .map_err(|m| failure("invalid_action", m))?;
                self.synchronize_window(now, false)
                    .map_err(|m| failure("invalid_action", m))?;
            }
            SessionAction::SubmitResponse {
                window_id,
                intent_id,
                action,
            } => {
                if !matches!(self.require_member(seat, window_id)?, Decision::Composing { intent_id: current } if current == intent_id)
                {
                    return Err(failure("window_expired", "请在当前窗口开始连锁并重新确认"));
                }
                if !response_action(action) {
                    return Err(failure("invalid_action", "该动作不是可提交的普通响应"));
                }
                self.game
                    .apply_at_revision(seat, action.clone(), revision)
                    .map_err(|m| failure("invalid_action", m))?;
                self.synchronize_window(now, true)
                    .map_err(|m| failure("invalid_action", m))?;
            }
        }
        self.revision = revision;
        self.game.version = revision;
        self.pacing.last_server_now_ms = now;
        Ok(())
    }
    pub fn transition(
        &self,
        seat: usize,
        command: Option<RoomCommand>,
        observed_now_ms: u64,
    ) -> RuleResult<RoomTransition> {
        if seat >= self.game.players.len() {
            return Err("无此认证座位".into());
        }
        let now = observed_now_ms.max(self.pacing.last_server_now_ms);
        let revision = self.revision.checked_add(1).ok_or("房间revision溢出")?;
        let mut next = self.clone();
        let mut journal = vec![];
        let expired = next.expire_due(now, revision)?;
        if expired {
            journal.push(SessionEvent::Tick {
                server_now_ms: observed_now_ms,
            });
        }
        let mut error = None;
        let mut accepted = false;
        if let Some(command) = command {
            let mut candidate = next.clone();
            match candidate.commit_action(seat, &command, now, revision) {
                Ok(()) => {
                    next = candidate;
                    accepted = true;
                    journal.push(SessionEvent::Command {
                        seat,
                        command,
                        server_now_ms: observed_now_ms,
                    });
                }
                Err(e) => error = Some(e),
            }
        }
        let changed = expired || accepted;
        if changed {
            next.revision = revision;
            next.game.version = revision;
            next.pacing.last_server_now_ms = now;
        }
        let view = next.view(seat, now);
        Ok(RoomTransition {
            state: serde_json::to_string(&next).map_err(|_| "房间状态序列化失败")?,
            view,
            version: next.revision,
            seat,
            changed,
            journal,
            outcome: if error.is_some() {
                "rejected"
            } else {
                "accepted"
            }
            .into(),
            error_code: error.as_ref().map(|e| e.0.clone()),
            error_message: error.map(|e| e.1),
        })
    }
    pub fn replay_events(&self, events: &[SessionEvent]) -> RuleResult<Self> {
        if !matches!(
            events,
            [SessionEvent::Tick { .. }]
                | [SessionEvent::Command { .. }]
                | [SessionEvent::Tick { .. }, SessionEvent::Command { .. }]
        ) {
            return Err("SessionEvents须为一次Tick、一次Command或依次Tick/Command".into());
        }
        let revision = self.revision.checked_add(1).ok_or("房间revision溢出")?;
        let mut next = self.clone();
        for event in events {
            match event {
                SessionEvent::Tick { server_now_ms } => {
                    if !next.expire_due(
                        (*server_now_ms).max(next.pacing.last_server_now_ms),
                        revision,
                    )? {
                        return Err("journal超时事件没有到期座位".into());
                    }
                }
                SessionEvent::Command {
                    seat,
                    command,
                    server_now_ms,
                } => {
                    next.commit_action(
                        *seat,
                        command,
                        (*server_now_ms).max(next.pacing.last_server_now_ms),
                        revision,
                    )
                    .map_err(|e| e.1)?;
                }
            }
        }
        next.revision = revision;
        next.game.version = revision;
        Ok(next)
    }
    pub fn quote(&self, seat: usize, request: QuoteRequest) -> RuleResult<Quote> {
        if self.pacing.pause.is_some() {
            return Err("此桌已暂停，请先恢复对局".into());
        }
        if !matches!(self.require_member(seat, &request.window_id).map_err(|e|e.1)?, Decision::Composing { intent_id } if intent_id == &request.intent_id)
        {
            return Err("报价只可用于本席当前编辑意图".into());
        }
        let mut actions = self.offers(seat);
        let mut ready = false;
        let mut normalized = None;
        let mut error = None;
        if let Some(draft) = &request.draft {
            actions.retain(|offer| partial_matches(&offer.action, draft));
            if response_action(draft) {
                let mut trial = self.game.clone();
                match trial.apply_at_revision(seat, draft.clone(), self.revision) {
                    Ok(()) => {
                        ready = true;
                        normalized = Some(
                            actions
                                .first()
                                .map_or_else(|| draft.clone(), |a| a.action.clone()),
                        );
                    }
                    Err(e) => error = Some(e),
                }
            }
        }
        Ok(Quote {
            version: self.revision,
            window_id: request.window_id,
            intent_id: request.intent_id,
            legal_actions: actions,
            ready,
            normalized_action: normalized,
            error,
        })
    }
}
fn partial_matches(candidate: &Action, draft: &Action) -> bool {
    (draft.kind.is_empty() || draft.kind == candidate.kind)
        && draft
            .card_id
            .as_ref()
            .is_none_or(|v| candidate.card_id.as_ref() == Some(v))
        && draft
            .target_id
            .as_ref()
            .is_none_or(|v| candidate.target_id.as_ref() == Some(v))
        && draft.region.is_none_or(|v| candidate.region == Some(v))
        && draft
            .option
            .as_ref()
            .is_none_or(|v| candidate.option.as_ref() == Some(v))
        && draft
            .ability_id
            .as_ref()
            .is_none_or(|v| candidate.ability_id.as_ref() == Some(v))
        && draft
            .cost_selected
            .as_ref()
            .is_none_or(|v| candidate.cost_selected.as_ref() == Some(v))
}
