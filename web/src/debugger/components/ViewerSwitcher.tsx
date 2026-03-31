import type { ViewerId } from "../protocol";

// Purpose: Provides the required mock per-player view switcher without introducing external state libraries.
type ViewerSwitcherProps = {
  selectedViewer: ViewerId;
  onViewerSelected: (viewerId: ViewerId) => void;
};

const viewers: ViewerId[] = ["P1", "P2", "spectator"];

export function ViewerSwitcher({ selectedViewer, onViewerSelected }: ViewerSwitcherProps) {
  return (
    <section className="panel">
      <h2>Viewer</h2>
      <div className="viewer-switcher" role="group" aria-label="Viewer Switcher">
        {viewers.map((viewerId) => (
          <button
            key={viewerId}
            type="button"
            className={viewerId === selectedViewer ? "viewer-button viewer-button--active" : "viewer-button"}
            onClick={() => onViewerSelected(viewerId)}
            aria-pressed={viewerId === selectedViewer}
            aria-label={`View as ${viewerId}`}
          >
            {viewerId}
          </button>
        ))}
      </div>
    </section>
  );
}
