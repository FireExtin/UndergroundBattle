#!/usr/bin/env python3
"""Read-only affiliation/ownership audit of two actual compiled finite registries."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# Conclusions below follow direct original-image review, not keyword inference.
NOTES = {
 'JC125': '无阵营限制；同名构筑例外不涉及归属。',
 'LC19': '本地区任意正面角色，不限本方；费用及横置由发动者本人承担。',
 'LC20': '进场源当前controller发动，本地区任意正面角色；无本方限制。',
 'LC21': '护卫1按对抗中的敌方/队伍分配；拥有者不定义敌我。',
 'LC22': '护卫1按敌我队伍；公开不限制其他效果翻暗。',
 'LC23': '自称只指该instance，另一本地区目标不限阵营；发动者是controller。',
 'LC24': '预测2及抓1均由进场源controller本人进行；不共享队友牌库。',
 'XQ49': '任意玩家墓地目标；OwnerDeckBottom按拥有者，后续Draw(Actor)按发动者，不能混用。',
 'JC118': '目标地区Any，无角色阵营限制；地区效果归原frame。',
 'JC002': '本地区任意正面角色，目标复验含屏障的当前controller敌我。',
 'JC003': '目标角色或暗藏者Any；费用2及横置自身按发动者，不能由队友代付。',
 'XQ03': '目标正面角色Any，敌方屏障由通用目标守卫判定。',
 'JC014': '你=当前controller；移动该instance而非同名或队友对象，2V2机动仅邻区。',
 'JC016': '敌方按当前controller的对手队伍；队友及暗藏者不算敌方角色。撤回按P18回owner；与BQ022赢区优先仍待裁定。',
 'JZ08': '屏障阻止敌方卡/能力指定目标，按目标当前controller团队；允许友方且不阻止非指定群伤。',
 'JC056': '本地区所有暗藏者，非指定集合Any，含双方，不错误限制本人。',
 'JC058': '本地区暗藏者Any；现身中栈实体不可再作场上目标，死亡归owner。',
 'JC059': '原图友方，按controller同队，包括队友，排除来源自身/暗藏者；不能改成本人限定。',
 'JC063': '两模式目标均Any；潜伏保留owner/controller并换instance；回手明确OwnerHand。',
 'JC086': '你的墓地为本人；OwnedByActor限定其拥有牌，ActorHand入本人手，当前私有区owner不变量使二者相符。',
 'XQ12': '死亡触发属于离场前controller；目标玩家Any，私密弃牌在目标玩家本人手中。',
 'JC092': '你的墓地角色为本人；OwnedByActor、HiddenInChosenRegion，换instance后owner/controller归本人。',
 'DQJC107': '每位玩家分别Search(Context)，检索与洗牌仅操作各自牌库，不按赢区者或团队汇总。',
 'DQJC112': '各Context玩家独立私密弃牌并等量抓牌，不共享手牌。',
 'DQJC113': '场上所有正面角色Any，直接群伤，不以owner/controller筛掉队友或自己。',
 'DQJC114': '模式决定后逐位Context抓/弃自己的牌，目标群体为每位玩家。',
 'JC042': '自称牺牲来源instance；后续降费绑定发动者Actor，仅其下次正面出牌，owner/队友不代领。',
 'JC049': '额外牺牲只能本人controller角色（P15），不是队友；MoveBottomToHand(Actor)仅本人牌库。',
 'JC091': '人类目标Any；行动阶段限定与屏障守卫独立于拥有者。',
 'JZ54': '目标玩家Any，包括自己/队友/敌人；由该目标玩家选择其本人controller角色牺牲，墓地回owner。',
 'DQJC108': '自赢区者Actor起逐位Context，仅各自controller的暗藏者可选；免费/忠诚沿用已批准版本FAQ。',
 'DQJC109': '所有非人类正面角色Any翻暗，保留owner/controller；普通结附清理仍回收owner。',
 'DQJC110': '所有角色翻暗；混牌按每位controller及地区分组，不能按队伍或owner合组。',
 'DQJC111': '自赢区者起逐位Context，各人牺牲本人controller角色、抓自己的牌；墓地归被牺牲牌owner，防御快照沿用版本裁定。',
 'DQJC115': '逐位Context各取本人墓地角色，正面进入同一已选地区；不从队友墓地选牌。',
 'DQJC116': '每人检索自己牌库的附属并入各自手，独立私密承诺，全部选完才共同展示/洗牌。',
 'BQ022': '宿主Any，人类/吸血鬼，不继承宿主controller。卡括注你的手中须按P17回收专门定义回附属owner；赢区组合优先未裁定。',
 'JC084': '本方/敌方势力标志由controller选择队伍；P21明确同队共用标志，此处不能机械改为个人。',
 'JC085': '你的资产按controller本人，不共享队友领域；墓地正面出牌仅本人墓地，普通费用/忠诚与instance更新。',
 'JC088': '现身目标Any，同区/印刷费<=2；自潜伏只该instance且本人付费，保留owner/controller，死亡归owner。',
 'JC001': '你的资产区心灵2按controller本人，owner/队友资产不贡献；不能被装备结附无阵营限制。',
 'BQ083': '本地区正面目标Any，包括自己/队友/敌方，屏障按目标controller复验；死亡归owner。',
 'JC006': '有领域正面目标Any，返回OwnerHand而非ActorHand，普通离场回收按各附属owner。',
 'XQ16': '53e47ca已将旧FriendlyTeam修为ControlledByActor；另一张本人controller角色，排除源；响应后转为队友控制则原目标失效。',
 'JC047': '地区Any、角色集合Any；群伤无阵营筛选，死亡各回owner。',
 'JC007': '你的牌库/手牌均Actor本人；按印刷费用与类别检索，展示后入本人手并洗牌。',
 'JC075': '本方=ControlledByActor而非OwnedByActor/FriendlyTeam；另一个正面角色返回其OwnerHand，费用2与横置由Actor承担。',
 'JC104': '你的牌库=Actor本人，私密Forecast3仅本人排序，既不共享队友牌库也不抓牌。',
}
# Added target/trait guards are independently recorded before comparison.
# equipment_host=true on BQ022 only prevents equipping newly admitted JC001.
EXTENSIONS = {'cannot_be_equipped', 'equipment_host', 'requires_magic', 'exclude_source', 'printed_cost_max'}

def canonical(value):
    if isinstance(value, dict):
        return {k: canonical(v) for k, v in value.items() if k not in EXTENSIONS}
    if isinstance(value, list):
        return [canonical(v) for v in value]
    return value

def guard_extensions(value, path=''):
    result = []
    if isinstance(value, dict):
        for k, v in value.items():
            if k in EXTENSIONS and v not in (False, None):
                result.append({'path': path + '.' + k, 'value': v})
            result += guard_extensions(v, path + '.' + k)
    elif isinstance(value, list):
        for i, v in enumerate(value):
            result += guard_extensions(v, path + f'[{i}]')
    return result

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('current', type=Path)
    parser.add_argument('published', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    current, published = [json.loads(p.read_text()) for p in (args.current, args.published)]
    raw = json.loads((ROOT / 'rust-game/data/cards.json').read_text())['cards']
    specs = json.loads((ROOT / 'docs/factions/card-specifications.json').read_text())['cards']
    active, old = current['definitions'], published['definitions']
    assert len(active) == 48 and len(old) == 39 and set(NOTES) == set(active)
    unchanged = [cid for cid in old if canonical(active[cid]) == canonical(old[cid])]
    assert set(unchanged) == set(old), set(old) - set(unchanged)
    nontrivial_guards = {cid: guard_extensions(active[cid]) for cid in old if guard_extensions(active[cid])}
    assert nontrivial_guards == {'BQ022': [
        {'path': '.abilities[0].targets[0].equipment_host', 'value': True},
        {'path': '.attachment.host.equipment_host', 'value': True},
    ]}, nontrivial_guards
    pending = ['JC016', 'BQ022']
    rows = []
    for c in raw:
        cid = c['id']
        image_path = c['source'].get('image') or c['source']['file']
        sha = hashlib.sha256((ROOT / image_path).read_bytes()).hexdigest()
        assert sha == specs[cid]['sourceVerification']['imageSha256'], cid
        rows.append({
            'id': cid, 'name': c['name'], 'scope': 'published39' if cid in old else 'local9',
            'originalImage': image_path, 'imageSha256': sha, 'actuallyViewedInThisAudit': True,
            'printedRules': specs[cid]['fields']['printedRuleTextLines'],
            'semanticConclusion': NOTES[cid],
            'affiliationOrOwnershipDefectFound': False,
            'historicalCorrection': '53e47ca' if cid == 'XQ16' else None,
            'interactionRulingPending': cid in pending,
            'compiledDefinition': active[cid],
        })
    result = {
        'sourceCommit': '8963fcd17b9dc9c6cf4e8347b936bc118cad4b11',
        'publishedSiteSource': '074def9eb5bdf49fe6a5df5ef18ebd70bbce5453',
        'scope': 'Direct original-image review plus actual compiled finite definitions; no remote site/room access or natural UI claim.',
        'totals': {'current': 48, 'published': 39, 'localOnly': 9, 'affiliationOrOwnershipDefectsFound': 0, 'interactionRulingsPending': 2},
        'publishedAffiliationOwnershipBindingsUnchanged': unchanged,
        'comparisonExcludedSchemaExtensionKeys': sorted(EXTENSIONS),
        'nontrivialPublishedGuardExtensions': nontrivial_guards,
        'guardExtensionBoundary': 'BQ022 now recognizes JC001 cannot_be_equipped; every published39 host has that trait=false. This is the prior reviewed local owl restriction, not an affiliation/ownership correction.',
        'localOnlyIds': sorted(set(active)-set(old)),
        'privateZoneInvariant': 'Normal admitted effects keep hand/deck/graveyard cards with their owner; no takeover or cross-private-zone ownership-transfer card is admitted. Explicit test control primitives are not natural-play acceptance.',
        'manualBasis': ['P15 ownership/control, sacrifice', 'P16 own/friendly/enemy and you', 'P17 Recycling=owner', 'P18 Retreat=owner, Forecast=you', 'P21 teammate privacy, shared influence markers, own/friendly/enemy'],
        'rulingBoundary': 'JC016+BQ022 Win return/retreat/recycling priority remains pending; no priority changed or combination disabled.',
        'cards': rows,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'totals': result['totals'], 'localOnlyIds': result['localOnlyIds'], 'all39AffiliationOwnershipBindingsUnchanged': True}, ensure_ascii=False))

if __name__ == '__main__':
    main()
