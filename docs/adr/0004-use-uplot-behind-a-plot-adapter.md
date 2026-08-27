# ADR 0004: Use uPlot behind a plot adapter

Date: 2026-08-28

Status: Accepted

## Context

Hemo Tracker must show 20 longitudinal laboratory plots at one time. Each plot must support irregular dates, changing source intervals, personal targets, source flags, and comparability boundaries. The first render target is 500 ms. The pointer interaction target is 50 ms.

Issue 8 compared uPlot 1.6.32 and Apache ECharts 6.1.0 with the same browser, fixture, accessible table, and keyboard workflow. The fixture contained 1,000 reports, 100,000 measurements, 250 analytes, and 20 visible plots. The throwaway prototype is in branch [`prototype/plot-engine-benchmark`](https://github.com/pblan/hemo-tracker/tree/prototype/plot-engine-benchmark/apps/desktop/src/plot-engine-prototype).

uPlot rendered the first view in 42.4 ms. Its 95th percentile pointer-frame latency was 10.4 ms. ECharts rendered the first view in 1,154.7 ms. Its pointer latency was 10.3 ms. uPlot used about 17.7 MB of JavaScript heap. ECharts used about 103.6 MB. The candidate chunks were about 55 KB and 558 KB before compression.

Both candidates exported an image, changed theme, released their canvases, and used the same keyboard-accessible table. ECharts provides more built-in plot behavior, but its first render did not meet the product target with the required changing intervals.

## Decision

Use uPlot for V1 longitudinal plots.

Keep uPlot behind a local plot adapter. Feature code supplies normalized points, source intervals, personal targets, source flags, comparability boundaries, and theme tokens. Feature code must not supply uPlot option objects.

Use an accessible data table as the semantic view. Do not use the canvas as the only source of result information or keyboard access.

## Consequences

The desktop bundle and plot memory use stay small. The first render and pointer interaction meet the V1 targets in the product benchmark.

The application must implement tooltips, annotations, linked cursors, keyboard commands, and export behavior in the adapter. Tests must cover these behaviors because uPlot does not provide all of them.

The application can replace uPlot without changing feature code if the adapter remains stable. Re-run the product benchmark before a replacement.

## Rejected alternative

Do not use Apache ECharts for V1. Its built-in zoom, tooltip, annotation, and accessibility features reduce adapter work. However, the benchmark first render exceeded the target by more than two times, and its measured memory and candidate chunk were much larger.

## Evidence limits

The benchmark ran in headless Chromium on one macOS development computer. It measures comparative product behavior, not all supported computers. Packaged macOS and Windows release tests must verify that the production adapter still meets the targets.
