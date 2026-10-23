//! MSJC14: printed gray construction and two paid, exhausted society actions.
use crate::{engine::RuleResult, model::*, rules::*};
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut anchor = target(Zone::Board, EntityKind::CharacterOrHidden, Relation::Any, Range::Anywhere);
    anchor.predicate = Some(TargetPredicate::SentinelLockedAnchor);
    let mut actions = vec![
        ability("sentinel-lock", "行动3，横置：锁定目标敌方暗藏者", Timing::Standard,
            vec![Cost::Assets(3), Cost::ExhaustSource],
            vec![target(Zone::Board, EntityKind::Hidden, Relation::EnemyTeam, Range::Anywhere)], vec![Op::SentinelLock], None),
        ability("sentinel-move", "行动3，横置：移动本方角色至锁定目标所在地区", Timing::Standard,
            vec![Cost::Assets(3), Cost::ExhaustSource],
            vec![target(Zone::Board, EntityKind::Character, Relation::ControlledByActor, Range::Anywhere), anchor], vec![Op::SentinelMove], None),
    ];
    for a in &mut actions { a.activation_only = true; }
    with_abilities(actions)
}
pub(crate) fn society() -> crate::society::SocietyDefinition {
    let mut card: crate::catalog::CardDefinition = serde_json::from_value(serde_json::json!({
        "id":"MSJC14","name":"哨兵","kind":"society","type":"秘社/机构","subtypes":["机构"],
        "color":"灰","society":"哨兵","unique":true,"supported":true,
        "text":"持续：构筑时你的牌组中只能包含灰和中立牌，以及其他派系不具有魔法领域图标的牌。行动3，横置：在目标敌方暗藏者上放置一个锁定标志。行动3，横置：选择目标本方角色，将其移动到目标具有锁定标志的角色或暗藏者所在的地区。"
    })).unwrap();
    card.abilities = definition().abilities.into_iter().map(|a| crate::catalog::AbilitySummary {
        key:a.key, label:a.label, timing:"standard".into(), costs:a.costs, triggered:false,
    }).collect();
    crate::society::SocietyDefinition { card, subtitle:"国际超自然监控中心".into(), printed_cost:None,
        starting_hand:6, deck_constraints:vec![crate::society::SocietyDeckConstraint::GrayNeutralOrNoMagic], unresolved_abilities:Default::default() }
}
fn owns(op: &Op) -> bool {
    match op {
        Op::SentinelLock | Op::SentinelMove => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => b.iter().any(owns),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(exhausted) || owns(ready),
        _ => false,
    }
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("sentinel-") || a.ops.iter().any(owns) || a.modes.iter().any(|m| m.ops.iter().any(owns))
        || a.targets.iter().chain(a.modes.iter().flat_map(|m| &m.targets)).any(|t| t.predicate == Some(TargetPredicate::SentinelLockedAnchor))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id == "MSJC14" && definition().abilities.iter().any(|p| same(p,a)) { Ok(()) }
    else { Err("哨兵仅准入完整印刷行动、费用及双目标程序".into()) }
}
impl Game {
    // Manual printed p11: attachment markers also count on their host.
    // Query only the current legal descendant chain, never copy counters or
    // inherit a parent's/sibling's markers back down onto an attachment.
    pub(crate) fn sentinel_has_effective_lock(&self, card: &Card) -> bool {
        let mut pending = vec![card.id.clone()];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id.clone()) { continue; }
            if self.in_play(&id).is_some_and(|(_, c)| c.lock_markers > 0) { return true; }
            for a in &self.attachments {
                if a.host_id == id && self.attachment_host_valid(a)
                    && (!a.card.face_down || crate::batch4_plans::secret(&a.card)) {
                    pending.push(a.card.id.clone());
                }
            }
        }
        false
    }

    pub(crate) fn sentinel_candidates(&self, actor: usize, c: &Card) -> Vec<(Action,String)> {
        let source = self.source_snapshot(c,None);
        let mut result = vec![];
        for ability in definition().abilities {
            for first in self.binding_options(actor,&source,&ability.targets[0]) {
                let choices = if ability.targets.len() == 1 { vec![None] } else {
                    self.binding_options(actor,&source,&ability.targets[1]).into_iter().map(Some).collect()
                };
                for second in choices {
                    let a = Action { card_id:Some(c.id.clone()), ability_id:Some(ability.key.clone()),
                        target_id:Some(first.id.clone()), selected:second.as_ref().map(|t| vec![t.id.clone()]), ..Action::new("activate") };
                    if self.sentinel_bind(actor,&source,&ability,&a).is_ok() {
                        result.push((a,format!("{} → {} [{}]{}",ability.label,first.label,first.id,
                            second.map(|t|format!("；锁定锚点 {} [{}]",t.label,t.id)).unwrap_or_default())));
                    }
                }
            }
        }
        result
    }
    pub(crate) fn sentinel_bind(&self, actor: usize, source: &SourceSnapshot, spec: &AbilitySpec, a: &Action) -> RuleResult<Vec<BoundTarget>> {
        validate_ability(&source.card.definition,spec)?;
        if a.kind != "activate" || *a != (Action { card_id:a.card_id.clone(), ability_id:a.ability_id.clone(), target_id:a.target_id.clone(), selected:a.selected.clone(), ..Action::new("activate") }) {
            return Err("哨兵仅声明原来源、行动与印刷目标".into());
        }
        let mut ids = vec![a.target_id.clone().ok_or("请选择第一目标")?];
        if spec.targets.len() == 2 {
            let selected = a.selected.as_ref().filter(|v| v.len()==1).ok_or("请选择独立锁定锚点")?;
            ids.push(selected[0].clone());
            if self.board(&ids[0]).zip(self.board(&ids[1])).is_none_or(|((from,_),(to,_))| from==to) {
                return Err("哨兵不能选择原地移动".into());
            }
        } else if a.selected.is_some() { return Err("锁定行动只有一个目标".into()); }
        ids.into_iter().zip(&spec.targets).map(|(id,slot)| {
            if !self.valid_binding(actor,source,slot,&id) || self.board(&id).is_none() { return Err("哨兵目标不满足印刷条件".into()); }
            Ok(BoundTarget { public:self.public_target(actor,source,slot,&id),id,spec:slot.clone(),region_instance:None })
        }).collect()
    }
}
