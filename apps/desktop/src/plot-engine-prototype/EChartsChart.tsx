import { useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import {
  AriaComponent,
  DataZoomComponent,
  GridComponent,
  MarkAreaComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";

import type { AnalyteSeries } from "./fixture";

echarts.use([
  AriaComponent,
  CanvasRenderer,
  DataZoomComponent,
  GridComponent,
  LineChart,
  MarkAreaComponent,
  MarkLineComponent,
  TooltipComponent,
]);

export function EChartsChart({
  series,
  dark,
}: {
  series: AnalyteSeries;
  dark: boolean;
}) {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!host.current) return;
    const chart = echarts.init(host.current, dark ? "dark" : undefined, {
      renderer: "canvas",
    });
    chart.setOption({
      animation: false,
      aria: { enabled: true },
      backgroundColor: "transparent",
      grid: { left: 50, right: 20, top: 20, bottom: 45 },
      tooltip: { trigger: "axis" },
      dataZoom: [{ type: "inside" }, { type: "slider", height: 15 }],
      xAxis: { type: "time" },
      yAxis: { type: "value", scale: true },
      series: [
        {
          type: "line",
          name: series.name,
          showSymbol: false,
          data: series.points.map((point) => ({
            value: [point.date * 1_000, point.value],
            symbol: point.flagged ? "circle" : "none",
            symbolSize: point.flagged ? 6 : 0,
            itemStyle: { color: point.flagged ? "#e53e3e" : "#319795" },
          })),
          markArea: {
            silent: true,
            data: series.points.slice(0, -1).map((point, index) => [
              { xAxis: point.date * 1_000, yAxis: point.low },
              {
                xAxis: series.points[index + 1]!.date * 1_000,
                yAxis: point.high,
              },
            ]),
          },
          markLine: {
            symbol: "none",
            data: [
              { yAxis: series.targetLow, name: "Target low" },
              { yAxis: series.targetHigh, name: "Target high" },
              {
                xAxis: series.points[500]!.date * 1_000,
                name: "Comparability boundary",
              },
            ],
          },
        },
      ],
    });
    host.current.dataset.exportBytes = String(
      chart.getDataURL({ type: "png" }).length,
    );
    return () => chart.dispose();
  }, [dark, series]);

  return (
    <div ref={host} aria-hidden="true" style={{ width: 560, height: 220 }} />
  );
}
