import { useEffect, useMemo, useState } from "react";

import { AccessibleDataTable } from "./AccessibleDataTable";
import type { AnalyteSeries } from "./fixture";
import { createStressFixture } from "./fixture";

type Engine = "echarts" | "uplot";
type ChartComponent = React.ComponentType<{
  series: AnalyteSeries;
  dark: boolean;
}>;

export default function PlotEnginePrototype() {
  const engine = (new URLSearchParams(location.search).get("engine") ??
    "uplot") as Engine;
  const fixture = useMemo(createStressFixture, []);
  const [Chart, setChart] = useState<ChartComponent>();
  const [dark, setDark] = useState(false);
  const [showPlots, setShowPlots] = useState(true);

  useEffect(() => {
    const started = performance.now();
    const load =
      engine === "echarts"
        ? import("./EChartsChart").then((module) => module.EChartsChart)
        : import("./UPlotChart").then((module) => module.UPlotChart);
    void load.then((LoadedChart) => {
      setChart(() => LoadedChart);
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          document.body.dataset.firstRenderMs = (
            performance.now() - started
          ).toFixed(2);
          document.body.dataset.ready = "true";
        }),
      );
    });
  }, [engine]);

  return (
    <main
      style={{
        background: dark ? "#171923" : "white",
        color: dark ? "white" : "#1a202c",
        padding: 24,
      }}
    >
      <h1>Plot engine benchmark: {engine}</h1>
      <p>
        {fixture.reportCount} reports · {fixture.measurementCount} measurements
        · {fixture.analyteCount} analytes · {fixture.visible.length} visible
        plots
      </p>
      <button onClick={() => setDark((value) => !value)}>Change theme</button>
      <button onClick={() => setShowPlots(false)}>Unmount plots</button>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(2, 600px)",
          gap: 16,
        }}
      >
        {showPlots &&
          Chart &&
          fixture.visible.map((series) => (
            <section key={series.id} data-chart>
              <h2>{series.name}</h2>
              <Chart series={series} dark={dark} />
              <AccessibleDataTable series={series} />
            </section>
          ))}
      </div>
    </main>
  );
}
