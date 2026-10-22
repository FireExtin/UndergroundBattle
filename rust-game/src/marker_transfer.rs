//! XQ20 prerequisite, deliberately unreachable from admitted commands.
//! This transient plan has no Deserialize/Serialize implementation. It freezes
//! physical carriers, checks every addition, then writes one counter batch.
//! Plan crossings are observed after the complete atomic counter batch. XQ20
//! still requires its own command, payment and target admission integration.
use crate::{batch4_markers::MarkerKind, catalog, engine::RuleResult, model::*};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MarkerRow {
    before: Card,
    host: Option<String>,
    after: [u32; 7],
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreparedMarkerMove {
    from: String,
    to: String,
    rows: Vec<MarkerRow>,
}

impl Game {
    fn marker_move_carrier(&self, id: &str) -> bool {
        self.in_play(id).is_some_and(|(_, c)| {
            (!c.face_down && catalog::card(&c.definition).kind == "character")
                || self.attachments.iter().any(|a| a.card.id == id
                    && self.attachment_host_valid(a)
                    && (!c.face_down || crate::batch4_plans::secret(c)))
        })
    }
    pub(crate) fn prepare_marker_move(&self, from: &str, to: &str) -> RuleResult<PreparedMarkerMove> {
        if from == to || !self.marker_move_carrier(from) || !self.marker_move_carrier(to) {
            return Err("灵魂通道需要两个不同的当前角色或附属目标".into());
        }
        if self.in_play(from).unwrap().1.controller != self.in_play(to).unwrap().1.controller {
            return Err("灵魂通道的两个目标必须由同一玩家操控".into());
        }
        // Manual P11: markers on an attachment also count as on its host.
        // Preserve physical identity; an attached destination retains its own
        // counters rather than clearing and counting them twice.
        let mut ids = vec![from.to_string()];
        let mut seen = BTreeSet::from([from.to_string()]);
        let mut cursor = 0;
        while cursor < ids.len() {
            let parent = ids[cursor].clone();
            cursor += 1;
            for a in &self.attachments {
                if a.host_id == parent && self.attachment_host_valid(a) && seen.insert(a.card.id.clone()) {
                    ids.push(a.card.id.clone());
                }
            }
        }
        if seen.insert(to.to_string()) { ids.push(to.to_string()); }
        let mut total = [0u32; 7];
        let mut rows = Vec::with_capacity(ids.len());
        for id in ids {
            let c = self.in_play(&id).ok_or("标志的物理承载者已离场")?.1;
            let mut after = [0; 7];
            for (i, k) in MarkerKind::ALL.into_iter().enumerate() {
                if id == to { after[i] = k.count(c); }
                else { total[i] = total[i].checked_add(k.count(c)).ok_or("指示物总数超过当前表示范围")?; }
            }
            rows.push(MarkerRow { before: c.clone(), host: self.attachments.iter().find(|a| a.card.id == id).map(|a| a.host_id.clone()), after });
        }
        let destination = rows.iter_mut().find(|r| r.before.id == to).unwrap();
        for (i, n) in total.into_iter().enumerate() {
            destination.after[i] = destination.after[i].checked_add(n).ok_or("目标指示物数量超过当前表示范围")?;
        }
        Ok(PreparedMarkerMove { from: from.into(), to: to.into(), rows })
    }
    pub(crate) fn commit_marker_move_counters(&mut self, prepared: PreparedMarkerMove) -> RuleResult<()> {
        // Revalidate the complete relevant snapshot before the first write.
        // No settlement/trigger dispatch can observe a half-written batch.
        if self.prepare_marker_move(&prepared.from, &prepared.to)? != prepared {
            return Err("标志移动的当前目标或物理承载者已改变".into());
        }
        let plan_changes = prepared.rows.iter().map(|r| (r.before.id.clone(), r.before.advancement_markers)).collect::<Vec<_>>();
        for row in prepared.rows {
            let c = self.board_mut(&row.before.id).expect("validated in-play physical carrier");
            for (k, n) in MarkerKind::ALL.into_iter().zip(row.after) { *k.field(c) = n; }
        }
        for (id, before) in plan_changes { self.observe_plan_advancement(&id, before); }
        Ok(())
    }
}
