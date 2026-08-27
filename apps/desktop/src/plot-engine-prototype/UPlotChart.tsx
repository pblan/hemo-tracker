import { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

import type { AnalyteSeries } from "./fixture";

export function UPlotChart({
  series,
  dark,
}: {
  series: AnalyteSeries;
  dark: boolean;
}) {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!host.current) return;
    const color = dark ? "#e6fffa" : "#234e52";
    const grid = dark ? "#4a5568" : "#e2e8f0";
    const data: uPlot.AlignedData = [
      series.points.map((point) => point.date),
      series.points.map((point) => point.low),
      series.points.map((point) => point.high),
      series.points.map((point) => point.value),
      series.points.map(() => series.targetLow),
      series.points.map(() => series.targetHigh),
    ];
    const plot = new uPlot(
      {
        width: 560,
        height: 220,
        cursor: { drag: { x: true, y: false } },
        axes: [
          { stroke: color, grid: { stroke: grid } },
          { stroke: color, grid: { stroke: grid } },
        ],
        series: [
          {},
          { label: "Source low", stroke: "#90cdf4", width: 1 },
          { label: "Source high", stroke: "#90cdf4", width: 1 },
          {
            label: series.name,
            stroke: "#319795",
            width: 2,
            points: { show: false },
          },
          { label: "Target low", stroke: "#d69e2e", dash: [5, 5], width: 1 },
          { label: "Target high", stroke: "#d69e2e", dash: [5, 5], width: 1 },
        ],
        bands: [{ series: [1, 2], fill: dark ? "#2a4365aa" : "#bee3f8aa" }],
        hooks: {
          draw: [
            (chart) => {
              const context = chart.ctx;
              const boundaryX = chart.valToPos(
                series.points[500]!.date,
                "x",
                true,
              );
              context.save();
              context.strokeStyle = "#e53e3e";
              context.setLineDash([6, 4]);
              context.beginPath();
              context.moveTo(boundaryX, chart.bbox.top);
              context.lineTo(boundaryX, chart.bbox.top + chart.bbox.height);
              context.stroke();
              context.setLineDash([]);
              context.fillStyle = "#e53e3e";
              for (const point of series.points.filter(
                (candidate) => candidate.flagged,
              )) {
                const x = chart.valToPos(point.date, "x", true);
                const y = chart.valToPos(point.value, "y", true);
                context.beginPath();
                context.arc(x, y, 3, 0, Math.PI * 2);
                context.fill();
              }
              context.restore();
            },
          ],
        },
      },
      data,
      host.current,
    );
    document.body.dataset.activePlotInstances = String(
      Number(document.body.dataset.activePlotInstances ?? 0) + 1,
    );
    return () => {
      plot.destroy();
      document.body.dataset.activePlotInstances = String(
        Number(document.body.dataset.activePlotInstances ?? 1) - 1,
      );
    };
  }, [dark, series]);

  return <div ref={host} aria-hidden="true" />;
}
