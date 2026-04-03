import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SetupWizard } from "./SetupWizard";

describe("SetupWizard", () => {
  it("blocks duplicate society selections before starting setup", async () => {
    const onStartSetup = vi.fn().mockResolvedValue(undefined);

    render(
      <SetupWizard
        setupState={null}
        pending={false}
        errorMessage=""
        onStartSetup={onStartSetup}
        onAdvanceSetup={vi.fn().mockResolvedValue(undefined)}
        onRefreshSetup={vi.fn().mockResolvedValue(undefined)}
      />
    );

    fireEvent.change(screen.getByLabelText("P1 派系 B"), {
      target: { value: "方碑序列" }
    });
    fireEvent.click(screen.getByRole("button", { name: "开始开局设置" }));

    expect(onStartSetup).not.toHaveBeenCalled();
    expect(await screen.findByText("每位玩家必须选择两个不同派系。")).toBeInTheDocument();
  });

  it("blocks duplicate society selections when advancing step one", async () => {
    const onAdvanceSetup = vi.fn().mockResolvedValue(undefined);

    render(
      <SetupWizard
        setupState={activeStepOneState()}
        pending={false}
        errorMessage=""
        onStartSetup={vi.fn().mockResolvedValue(undefined)}
        onAdvanceSetup={onAdvanceSetup}
        onRefreshSetup={vi.fn().mockResolvedValue(undefined)}
      />
    );

    fireEvent.change(screen.getByLabelText("P2 派系 B"), {
      target: { value: "王座会" }
    });
    fireEvent.click(screen.getByRole("button", { name: "执行下一步" }));

    expect(onAdvanceSetup).not.toHaveBeenCalled();
    expect(await screen.findByText("每位玩家必须选择两个不同派系。")).toBeInTheDocument();
  });
});

function activeStepOneState() {
  return {
    active: true,
    completed: false,
    currentStep: 1,
    seed: 20260402,
    steps: [
      { step: 1, title: "玩家选择牌组", completed: false },
      { step: 2, title: "设置世界牌库", completed: false },
      { step: 3, title: "整理标志", completed: false },
      { step: 4, title: "设置玩家牌库", completed: false },
      { step: 5, title: "翻开地区牌", completed: false },
      { step: 6, title: "抓取起始手牌", completed: false },
      { step: 7, title: "确定先手玩家", completed: false }
    ],
    markerPoolReady: false,
    worldDeckCount: 0,
    playerDeckCount: { P1: 0, P2: 0 },
    playerHandCount: { P1: 0, P2: 0 },
    mulliganUsed: { P1: false, P2: false }
  };
}
