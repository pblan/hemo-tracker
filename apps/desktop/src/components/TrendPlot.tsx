import { Box, Table, Text } from "@chakra-ui/react";
import "uplot/dist/uPlot.min.css";
import { useEffect, useMemo, useRef, useState } from "react";

export type TrendPoint = {
  reportId?: string;
  date: string;
  value: number | null;
  unit: string;
  sourceValue?: string;
  sourceUnit?: string;
  sourceReferenceInterval?: string;
  flag?: string;
  targetStatus?: "below target" | "in target" | "above target";
};

export type TimeRange = [number, number];

export function TrendPlot({
  title,
  points,
  onOpenReport,
  timeRange,
  onTimeRangeChange,
}: {
  title: string;
  points: TrendPoint[];
  onOpenReport?: (reportId: string) => void;
  timeRange?: TimeRange;
  onTimeRangeChange?: (range: TimeRange) => void;
}) {
  const numeric = useMemo(
    () => points.filter((point) => point.value !== null),
    [points],
  );
  const values = numeric.map((point) => point.value as number);
  const min = values.length ? Math.min(...values) : 0;
  const max = values.length ? Math.max(...values) : 1;
  const range = max - min || 1;
  const width = 640;
  const height = 180;
  const path = numeric
    .map((point, index) => {
      const x = (index / Math.max(numeric.length - 1, 1)) * (width - 24) + 12;
      const y =
        height - 12 - (((point.value as number) - min) / range) * (height - 24);
      return `${index ? "L" : "M"}${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(" ");
  const plotHost = useRef<HTMLDivElement>(null);
  const [plotReady, setPlotReady] = useState(false);
  useEffect(() => {
    const host = plotHost.current;
    if (!host || numeric.length < 2) {
      setPlotReady(false);
      return;
    }
    const timestamps = numeric.map((point) => Date.parse(point.date) / 1000);
    if (timestamps.some((timestamp) => !Number.isFinite(timestamp))) {
      setPlotReady(false);
      return;
    }
    let plot: { destroy: () => void } | undefined;
    let cancelled = false;
    void import("uplot")
      .then(({ default: UPlot }) => {
        if (cancelled) return;
        plot = new UPlot(
          {
            width: host.clientWidth || 640,
            height: 180,
            scales: {
              x: {
                time: true,
                range: timeRange ? () => timeRange : undefined,
              },
            },
            cursor: { drag: { x: true, y: false, setScale: true } },
            hooks: {
              setScale: [
                (currentPlot, scaleKey) => {
                  if (scaleKey !== "x" || !onTimeRangeChange) return;
                  const scale = currentPlot.scales.x;
                  if (
                    scale &&
                    scale.min !== undefined &&
                    scale.max !== undefined
                  )
                    onTimeRangeChange([scale.min, scale.max]);
                },
              ],
            },
            series: [{}, { label: title, stroke: "currentColor", width: 2 }],
            axes: [{}, { label: "Value" }],
          },
          [timestamps, numeric.map((point) => point.value as number)],
          host,
        );
        setPlotReady(true);
      })
      .catch(() => {
        // Keep the SVG fallback when a restricted webview cannot create a canvas.
        setPlotReady(false);
      });
    return () => {
      cancelled = true;
      plot?.destroy();
      setPlotReady(false);
    };
  }, [numeric, onTimeRangeChange, timeRange, title]);
  return (
    <Box
      borderWidth="1px"
      borderColor="border"
      borderRadius="xl"
      p="5"
      bg="bg.panel"
    >
      <Text fontWeight="semibold" mb="3">
        {title}
      </Text>
      <Text color="fg.muted" fontSize="sm" mb="2">
        Drag across the plot to zoom the collection-time range. Use the table
        for keyboard inspection and exact values.
      </Text>
      <Box ref={plotHost} aria-hidden="true" minH="180px" />
      {!plotReady ? (
        <svg
          role="img"
          aria-label={`${title} trend plot`}
          viewBox={`0 0 ${width} ${height}`}
          width="100%"
          height="180px"
          preserveAspectRatio="none"
        >
          <path d={path} fill="none" stroke="currentColor" strokeWidth="3" />
        </svg>
      ) : null}
      <Table.Root size="sm" variant="outline" mt="4">
        <Table.Caption>Accessible data table for {title}</Table.Caption>
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeader>Date</Table.ColumnHeader>
            <Table.ColumnHeader>Source value</Table.ColumnHeader>
            <Table.ColumnHeader>Source interval</Table.ColumnHeader>
            <Table.ColumnHeader>Normalized value</Table.ColumnHeader>
            <Table.ColumnHeader>Flag</Table.ColumnHeader>
            <Table.ColumnHeader>Personal target</Table.ColumnHeader>
            <Table.ColumnHeader>Source report</Table.ColumnHeader>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {points.map((point, index) => (
            <Table.Row key={`${point.date}-${point.value}-${index}`}>
              <Table.Cell>{point.date}</Table.Cell>
              <Table.Cell>
                {point.value === null
                  ? "Missing"
                  : point.sourceValue
                    ? `${point.sourceValue}${point.sourceUnit ? ` ${point.sourceUnit}` : ""}`
                    : "Not recorded"}
              </Table.Cell>
              <Table.Cell>{point.sourceReferenceInterval || "—"}</Table.Cell>
              <Table.Cell>
                {point.value === null
                  ? "Not evaluated"
                  : `${point.value} ${point.unit}`}
              </Table.Cell>
              <Table.Cell>{point.flag || "—"}</Table.Cell>
              <Table.Cell>{point.targetStatus || "Not evaluated"}</Table.Cell>
              <Table.Cell>
                {point.reportId && onOpenReport ? (
                  <button
                    type="button"
                    onClick={() => onOpenReport(point.reportId!)}
                  >
                    Open report
                  </button>
                ) : (
                  "—"
                )}
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}
