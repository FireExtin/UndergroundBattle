import { useMemo, useState } from "react";

import type { SetupAdvanceInput, SetupStartInput, SetupState } from "../../debugger/live";

// Purpose: Renders the seven-step setup wizard before battle starts.

const societyOptions = [
  "方碑序列",
  "帷幕守望",
  "王座会",
  "国家机构",
  "黑榜",
  "黯月",
  "雾野",
  "星环"
];

type SetupWizardProps = {
  setupState: SetupState | null;
  pending: boolean;
  errorMessage: string;
  onStartSetup: (input: SetupStartInput) => Promise<void>;
  onAdvanceSetup: (input: SetupAdvanceInput) => Promise<void>;
  onRefreshSetup: () => Promise<void>;
};

export function SetupWizard({
  setupState,
  pending,
  errorMessage,
  onStartSetup,
  onAdvanceSetup,
  onRefreshSetup
}: SetupWizardProps) {
  const [seed, setSeed] = useState<string>("20260402");
  const [p1SocietyA, setP1SocietyA] = useState<string>("方碑序列");
  const [p1SocietyB, setP1SocietyB] = useState<string>("帷幕守望");
  const [p2SocietyA, setP2SocietyA] = useState<string>("王座会");
  const [p2SocietyB, setP2SocietyB] = useState<string>("国家机构");
  const [mulliganP1, setMulliganP1] = useState<string>("0");
  const [mulliganP2, setMulliganP2] = useState<string>("0");
  const [startingPlayerId, setStartingPlayerId] = useState<"P1" | "P2">("P1");
  const [usePreviousLoserChoice, setUsePreviousLoserChoice] = useState<boolean>(false);

  const selectedP1Societies = useMemo(() => {
    return uniqueSocieties([p1SocietyA, p1SocietyB]);
  }, [p1SocietyA, p1SocietyB]);
  const selectedP2Societies = useMemo(() => {
    return uniqueSocieties([p2SocietyA, p2SocietyB]);
  }, [p2SocietyA, p2SocietyB]);

  if (!setupState?.active) {
    return (
      <section className="panel battle-setup" aria-label="开局设置向导">
        <h1>隐秘世界 开局设置</h1>
        <p className="muted">准备开始《隐秘世界》卡牌游戏，请按规则顺序完成初始设置。</p>
        {errorMessage ? <p className="custom-action-error">{errorMessage}</p> : null}
        <ol className="simple-list">
          <li>玩家选择牌组（快速组牌：选择两个派系）。</li>
          <li>设置世界牌库（地区牌 DQJC107~DQJC116）。</li>
          <li>整理标志指示物。</li>
          <li>设置玩家牌库（洗牌与互洗语义）。</li>
          <li>翻开 3 张地区牌作为争夺区域。</li>
          <li>双方抓 6 并可执行一次再调度。</li>
          <li>确定先手玩家并进入正式对战。</li>
        </ol>
        <div className="battle-actions__grid">
          <label className="battle-actions__field">
            <span>随机种子</span>
            <input
              aria-label="随机种子"
              type="number"
              value={seed}
              onChange={(event) => setSeed(event.target.value)}
              disabled={pending}
            />
          </label>
          <SocietySelectors
            prefix="P1"
            first={p1SocietyA}
            second={p1SocietyB}
            onFirstChanged={setP1SocietyA}
            onSecondChanged={setP1SocietyB}
            disabled={pending}
          />
          <SocietySelectors
            prefix="P2"
            first={p2SocietyA}
            second={p2SocietyB}
            onFirstChanged={setP2SocietyA}
            onSecondChanged={setP2SocietyB}
            disabled={pending}
          />
        </div>
        <div className="battle-actions__buttons">
          <button
            type="button"
            className="action-button"
            disabled={pending}
            onClick={() =>
              void onStartSetup({
                seed: parsePositiveInt(seed),
                p1Societies: selectedP1Societies,
                p2Societies: selectedP2Societies
              })
            }
          >
            开始开局设置
          </button>
          <button
            type="button"
            className="action-button action-button--secondary"
            disabled={pending}
            onClick={() => void onRefreshSetup()}
          >
            刷新设置状态
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="panel battle-setup" aria-label="开局设置向导">
      <h1>开局设置进行中</h1>
      <p className="muted">
        当前步骤：第 {setupState.currentStep} / 7 步
        {setupState.lastStepMessage ? ` · ${setupState.lastStepMessage}` : ""}
      </p>
      {errorMessage ? <p className="custom-action-error">{errorMessage}</p> : null}
      <ol className="simple-list">
        {setupState.steps.map((step) => (
          <li key={step.step}>
            {step.step}. {step.title} {step.completed ? "（已完成）" : ""}
          </li>
        ))}
      </ol>

      {setupState.revealedRegions && setupState.revealedRegions.length > 0 ? (
        <section>
          <h2>已翻开地区</h2>
          <ul className="simple-list">
            {setupState.revealedRegions.map((region) => (
              <li key={region.cardId}>
                <details>
                  <summary>
                    地区{region.regionOrder} · {region.name} · 势力值 {region.influenceLimit} · 分值 {region.score}
                  </summary>
                  <p>{region.description || "（暂无描述）"}</p>
                  {region.faq ? <p className="muted">FAQ：{region.faq}</p> : null}
                </details>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <p className="muted">
        牌库：P1={setupState.playerDeckCount.P1 ?? 0} / P2={setupState.playerDeckCount.P2 ?? 0}，
        手牌：P1={setupState.playerHandCount.P1 ?? 0} / P2={setupState.playerHandCount.P2 ?? 0}
      </p>

      <div className="battle-actions__grid">
        {setupState.currentStep === 1 ? (
          <>
            <SocietySelectors
              prefix="P1"
              first={p1SocietyA}
              second={p1SocietyB}
              onFirstChanged={setP1SocietyA}
              onSecondChanged={setP1SocietyB}
              disabled={pending}
            />
            <SocietySelectors
              prefix="P2"
              first={p2SocietyA}
              second={p2SocietyB}
              onFirstChanged={setP2SocietyA}
              onSecondChanged={setP2SocietyB}
              disabled={pending}
            />
          </>
        ) : null}

        {setupState.currentStep === 6 ? (
          <>
            <label className="battle-actions__field">
              <span>P1 再调度置底数</span>
              <input
                aria-label="P1 再调度置底数"
                type="number"
                min={0}
                max={6}
                value={mulliganP1}
                onChange={(event) => setMulliganP1(event.target.value)}
                disabled={pending}
              />
            </label>
            <label className="battle-actions__field">
              <span>P2 再调度置底数</span>
              <input
                aria-label="P2 再调度置底数"
                type="number"
                min={0}
                max={6}
                value={mulliganP2}
                onChange={(event) => setMulliganP2(event.target.value)}
                disabled={pending}
              />
            </label>
          </>
        ) : null}

        {setupState.currentStep === 7 ? (
          <>
            <label className="battle-actions__field">
              <span>先手玩家</span>
              <select
                aria-label="先手玩家"
                value={startingPlayerId}
                onChange={(event) => setStartingPlayerId(event.target.value as "P1" | "P2")}
                disabled={pending}
              >
                <option value="P1">P1</option>
                <option value="P2">P2</option>
              </select>
            </label>
            <label className="battle-actions__field">
              <span>败者指定先手</span>
              <select
                aria-label="败者指定先手"
                value={usePreviousLoserChoice ? "yes" : "no"}
                onChange={(event) => setUsePreviousLoserChoice(event.target.value === "yes")}
                disabled={pending}
              >
                <option value="no">否</option>
                <option value="yes">是</option>
              </select>
            </label>
          </>
        ) : null}
      </div>

      <div className="battle-actions__buttons">
        <button
          type="button"
          className="action-button"
          disabled={pending || setupState.completed}
          onClick={() => {
            const input: SetupAdvanceInput = {};
            if (setupState.currentStep === 1) {
              input.p1Societies = selectedP1Societies;
              input.p2Societies = selectedP2Societies;
            } else if (setupState.currentStep === 6) {
              input.mulliganBottomCount = {
                P1: parseNonNegativeInt(mulliganP1),
                P2: parseNonNegativeInt(mulliganP2)
              };
            } else if (setupState.currentStep === 7) {
              input.startingPlayerId = startingPlayerId;
              input.usePreviousLoserChoice = usePreviousLoserChoice;
            }
            void onAdvanceSetup(input);
          }}
        >
          执行下一步
        </button>
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={pending}
          onClick={() => void onRefreshSetup()}
        >
          刷新设置状态
        </button>
      </div>
    </section>
  );
}

type SocietySelectorsProps = {
  prefix: "P1" | "P2";
  first: string;
  second: string;
  onFirstChanged: (value: string) => void;
  onSecondChanged: (value: string) => void;
  disabled: boolean;
};

function SocietySelectors({
  prefix,
  first,
  second,
  onFirstChanged,
  onSecondChanged,
  disabled
}: SocietySelectorsProps) {
  return (
    <>
      <label className="battle-actions__field">
        <span>{prefix} 派系 A</span>
        <select
          aria-label={`${prefix} 派系 A`}
          value={first}
          onChange={(event) => onFirstChanged(event.target.value)}
          disabled={disabled}
        >
          {societyOptions.map((option) => (
            <option key={`${prefix}-a-${option}`} value={option}>
              {option}
            </option>
          ))}
        </select>
      </label>
      <label className="battle-actions__field">
        <span>{prefix} 派系 B</span>
        <select
          aria-label={`${prefix} 派系 B`}
          value={second}
          onChange={(event) => onSecondChanged(event.target.value)}
          disabled={disabled}
        >
          {societyOptions.map((option) => (
            <option key={`${prefix}-b-${option}`} value={option}>
              {option}
            </option>
          ))}
        </select>
      </label>
    </>
  );
}

function uniqueSocieties(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter((value) => value !== "")));
}

function parsePositiveInt(raw: string): number | undefined {
  const parsed = Number(raw);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }
  return Math.floor(parsed);
}

function parseNonNegativeInt(raw: string): number {
  const parsed = Number(raw);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return 0;
  }
  return Math.floor(parsed);
}
