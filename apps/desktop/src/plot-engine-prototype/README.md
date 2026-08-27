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

The command succeeds when it writes `plot-benchmark-results.json`. Both
candidates must report `keyboardTableReachable: true` and
`cleanupCanvasCount: 0`. Install Playwright Chromium again if the browser
executable is missing. Stop a process that uses port 1420 if the development
server cannot start.

Run the interactive prototype:

```sh
bun run prototype:plots
```

Open `http://127.0.0.1:1420/?plot-engine-prototype&engine=uplot` or replace
`uplot` with `echarts`.

Verify that 20 plots appear. Use Tab and Enter to open a result table. Use the
theme button to verify a theme change. Use the unmount button to verify that all
plots disappear.

The fixture has 1,000 irregular lab report dates, 100,000 measurements across 250
analyte definitions, and 20 visible plots. Both candidates use the same fixture,
source reference intervals, personal target ranges, a comparability boundary, an
accessible table, and a keyboard path.

The benchmark records first render, pointer-frame latency, JavaScript heap use,
candidate chunk size, image export, theme change, keyboard table access, and
cleanup. The prototype is not production code.
