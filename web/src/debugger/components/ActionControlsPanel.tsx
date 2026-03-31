import type { ViewerId } from "../protocol";
import type { LiveActionPreset } from "../live";

// Purpose: Offers a tiny set of preset actions for the live sandbox without adding a full action-authoring UI.
type ActionControlsPanelProps = {
  selectedViewer: ViewerId;
  pending: boolean;
  presets: LiveActionPreset[];
  disabledReason: string;
  onActionSelected: (presetId: LiveActionPreset["id"]) => void;
  onReload: () => void;
  onReset: () => void;
};

export function ActionControlsPanel({
  selectedViewer,
  pending,
  presets,
  disabledReason,
  onActionSelected,
  onReload,
  onReset
}: ActionControlsPanelProps) {
  const actionsDisabled = disabledReason !== "";

  return (
    <section className="panel">
      <h2>Action Controls</h2>
      <p className="muted">Actor: {selectedViewer}</p>
      {disabledReason ? <p className="muted">{disabledReason}</p> : null}
      <div className="action-controls">
        {presets.map((preset) => (
          <button
            key={preset.id}
            type="button"
            className="action-button"
            disabled={actionsDisabled || pending}
            onClick={() => onActionSelected(preset.id)}
            aria-label={preset.label}
          >
            {preset.label}
          </button>
        ))}
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={pending}
          onClick={onReload}
        >
          Reload Feed
        </button>
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={pending}
          onClick={onReset}
          aria-label="Reset Sandbox"
        >
          Reset Sandbox
        </button>
      </div>
    </section>
  );
}
