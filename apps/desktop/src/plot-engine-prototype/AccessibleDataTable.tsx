import type { AnalyteSeries } from "./fixture";

export function AccessibleDataTable({ series }: { series: AnalyteSeries }) {
  return (
    <details>
      <summary>Accessible data for {series.name}</summary>
      <div
        tabIndex={0}
        role="region"
        aria-label={`${series.name} result table`}
      >
        <table>
          <thead>
            <tr>
              <th>Date</th>
              <th>Value</th>
              <th>Source interval</th>
              <th>Flag</th>
            </tr>
          </thead>
          <tbody>
            {series.points.slice(-20).map((point) => (
              <tr key={point.date}>
                <td>
                  {new Date(point.date * 1_000).toISOString().slice(0, 10)}
                </td>
                <td>
                  {point.value.toFixed(2)} {series.unit}
                </td>
                <td>
                  {point.low.toFixed(2)}–{point.high.toFixed(2)}
                </td>
                <td>{point.flagged ? "Source flag" : "None"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}
