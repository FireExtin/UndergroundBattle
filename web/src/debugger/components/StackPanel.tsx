import type { Operation } from "../protocol";

// Purpose: Renders the authoritative stack with the top item shown first for debugger readability.
type StackPanelProps = {
  stack: Operation[];
};

export function StackPanel({ stack }: StackPanelProps) {
  return (
    <section className="panel">
      <h2>Stack</h2>
      <ol className="simple-list" aria-label="Stack Items">
        {stack.map((operation) => (
          <li key={operation.id}>
            <strong>{operation.label ?? operation.id}</strong>
            <span className="muted"> {operation.kind}</span>
          </li>
        ))}
      </ol>
    </section>
  );
}
