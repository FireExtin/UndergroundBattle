import type {
  ActionFieldRule,
  RulesMetadata
} from "../debugger/protocol";

// Purpose: Interprets authoritative action-policy and loyalty metadata for UI validation and tests.

export type ActionValidationInput = {
  actionKind: string;
  sourceCardId: string;
  targetCardId: string;
  targetRegionCardId: string;
  targetPlayerId: string;
  markerType: string;
  markerAmount: string;
  playMode: string;
  randomMax?: string;
  sourceCardKind: string;
};

export function validateActionInput(
  metadata: RulesMetadata | undefined,
  input: ActionValidationInput
) {
  const policy = metadata?.actionPolicies.find((candidate) => candidate.actionKind === input.actionKind);
  if (!policy) {
    return "";
  }

  for (const rule of policy.fieldRules ?? []) {
    if (!fieldRuleApplies(rule, input.sourceCardKind)) {
      continue;
    }
    const validation = validateFieldRule(rule, input);
    if (validation !== "") {
      return validation;
    }
  }

  return "";
}

export function parseLoyaltyRequirements(
  loyalty: string,
  metadata: RulesMetadata | undefined
) {
  const text = loyalty.trim();
  if (text === "" || text === "-") {
    return {};
  }
  const aliases = metadata?.loyalty.colorAliases ?? [];
  const tokens = aliases
    .flatMap((mapping) => [mapping.canonical, ...(mapping.aliases ?? [])].map((token) => ({
      token,
      canonical: mapping.canonical
    })))
    .sort((left, right) => right.token.length - left.token.length || left.token.localeCompare(right.token));

  const result: Record<string, number> = {};
  const runes = Array.from(text);
  for (let cursor = 0; cursor < runes.length; ) {
    let matched = false;
    for (const token of tokens) {
      const tokenRunes = Array.from(token.token);
      if (cursor + tokenRunes.length > runes.length) {
        continue;
      }
      if (runes.slice(cursor, cursor + tokenRunes.length).join("") !== token.token) {
        continue;
      }
      result[token.canonical] = (result[token.canonical] ?? 0) + 1;
      cursor += tokenRunes.length;
      matched = true;
      break;
    }
    if (!matched) {
      cursor += 1;
    }
  }
  return result;
}

export function normalizeColor(raw: string, metadata: RulesMetadata | undefined) {
  const value = raw.trim();
  if (value === "" || value === "中立") {
    return "";
  }
  for (const mapping of metadata?.loyalty.colorAliases ?? []) {
    if (mapping.canonical === value) {
      return mapping.canonical;
    }
    if ((mapping.aliases ?? []).includes(value)) {
      return mapping.canonical;
    }
  }
  return "";
}

function fieldRuleApplies(rule: ActionFieldRule, sourceCardKind: string) {
  if (!rule.sourceKinds || rule.sourceKinds.length === 0) {
    return true;
  }
  return rule.sourceKinds.includes(sourceCardKind);
}

function validateFieldRule(rule: ActionFieldRule, input: ActionValidationInput) {
  switch (rule.field) {
    case "cardId":
      return validateRequiredText(input.sourceCardId, rule, "需要选择来源卡牌");
    case "targetCardId":
      return validateRequiredText(input.targetCardId, rule, "需要选择目标卡牌");
    case "targetRegionCardId":
      return validateRequiredText(input.targetRegionCardId, rule, "需要选择部署地区");
    case "targetPlayerId":
      return validateRequiredText(input.targetPlayerId, rule, "需要选择目标玩家");
    case "markerType":
      return validateRequiredText(input.markerType, rule, "需要输入标记类型");
    case "markerAmount":
      return validateMinimumNumber(input.markerAmount, rule, "标记数量必须大于 0");
    case "randomMax":
      return validateMinimumNumber(input.randomMax ?? "", rule, "随机上限必须大于 0");
    default:
      return "";
  }
}

function validateRequiredText(value: string, rule: ActionFieldRule, message: string) {
  if (rule.requirement !== "required") {
    return "";
  }
  return value.trim() === "" ? message : "";
}

function validateMinimumNumber(value: string, rule: ActionFieldRule, message: string) {
  const minimum = rule.minimumInt ?? 0;
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < minimum) {
    return message;
  }
  return "";
}
