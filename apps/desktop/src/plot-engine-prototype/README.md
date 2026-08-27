# Plot engine prototype

This throwaway prototype answers one question: Which engine, uPlot or Apache
ECharts, meets the Hemo Tracker V1 plot requirements with the lower product
cost?

Install the repository dependencies and Playwright Chromium. Run the benchmark:

```sh
bun install --frozen-lockfile
bunx playwright install chromium
bun run prototype:plots:benchmark
```

Run the interactive prototype:

```sh
bun run prototype:plots
```

Open `http://127.0.0.1:1420/?plot-engine-prototype&engine=uplot` or replace
`uplot` with `echarts`.

The fixture has 1,000 irregular report dates, 100,000 measurements across 250
analyte definitions, and 20 visible plots. Both candidates use the same fixture,
source intervals, personal targets, comparability boundary, accessible table,
and keyboard path.

The benchmark records first render, pointer-frame latency, JavaScript heap use,
candidate chunk size, image export, theme change, keyboard table access, and
cleanup. The prototype is not production code.
