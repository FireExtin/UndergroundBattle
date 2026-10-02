type ProjectOverviewProps = {
  heading: string;
  items: string[];
};

// Purpose: Displays short scaffold summaries without pulling in extra UI dependencies.
export function ProjectOverview({ heading, items }: ProjectOverviewProps) {
  return (
    <section className="panel">
      <h2>{heading}</h2>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </section>
  );
}

