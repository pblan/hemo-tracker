# Plot engine research

Date: 2026-08-27

## Scope

This note compares plot engines for a React 19 desktop client. The client will show longitudinal laboratory results. It must work offline. It must support irregular dates, reference intervals, tooltips, zoom, and small multiples.

ADR 0004 selects uPlot after the product benchmark. The earlier sections record the evidence that defined the two-candidate prototype.

## Evidence limits

The libraries do not publish one neutral benchmark that uses the same data, device, and test method. The uPlot project publishes a direct comparison. The project also implements that comparison. Treat its results as first-party claims, not as an independent test. Its published test used Chrome 113 and hardware from 2023. It excluded network transfer time. It used older library versions than the current versions.

Package unpacked size is not application bundle size. It includes files that a production bundle can omit. Tree shaking, partial bundles, compression, and source maps change the result. Measure the final application bundle before a decision.

## Current package state

The npm registry showed these versions on 2026-08-27:

| Engine | Version | Unpacked package size | Last npm change |
| --- | ---: | ---: | --- |
| uPlot | 1.6.32 | 0.55 MB | 2025-03-14 |
| Apache ECharts | 6.1.0 | 60.3 MB | 2026-05-19 |
| Plotly.js | 4.0.0 | 94.3 MB | 2026-08-26 |
| Vega | 6.4.0 | 3.73 MB | 2026-08-14 |
| Vega-Lite | 6.4.3 | 5.81 MB | 2026-05-13 |
| Recharts | 3.10.1 | 7.45 MB | 2026-08-23 |

Sources: [uPlot on npm](https://www.npmjs.com/package/uplot), [ECharts on npm](https://www.npmjs.com/package/echarts), [Plotly.js on npm](https://www.npmjs.com/package/plotly.js), [Vega on npm](https://www.npmjs.com/package/vega), [Vega-Lite on npm](https://www.npmjs.com/package/vega-lite), and [Recharts on npm](https://www.npmjs.com/package/recharts).

## Facts

### uPlot

uPlot uses Canvas 2D. Its minified distribution is about 50 KB before application compression. The project reports a 34 ms cold render for 166,650 points in its comparison test. It reports low memory use and fast cursor movement. It also states that visible data above about 100,000 points can affect 60-frame-per-second updates. These numbers are first-party results and need an application benchmark. [uPlot README and benchmark](https://github.com/leeoniya/uPlot/blob/master/README.md)

uPlot requires sorted, unique numeric x values. It supports Unix time and calendar-aware time scales. This supports irregular collection dates for one series. All series in one plot must use one aligned x array. Sparse series can require many `null` values. This rule can make a plot with several unrelated analytes inefficient. Separate small plots avoid most of this problem. [uPlot data format](https://github.com/leeoniya/uPlot/blob/master/docs/README.md#data-format)

uPlot has native high and low bands. It has scales, axes, cursors, and hooks. Custom annotations, HTML tooltips, linked cursors, keyboard control, and export need application code or plug-ins. Canvas output does not expose each point to assistive technology. The application must provide an accessible text or table view. The core project lists a third-party React wrapper. It does not provide an official React component. [uPlot documentation](https://github.com/leeoniya/uPlot/blob/master/docs/README.md)

The library and its assets can ship inside the desktop application. It does not need a network service.

### Apache ECharts

ECharts supports Canvas and SVG. Its official guidance says that Canvas is more suitable when a chart has many elements. ECharts supports progressive rendering, typed arrays, and large-data modes. The project claims support for tens of millions of data points. The claim does not define this product's chart shape or small-multiple workload. It needs a local benchmark. [ECharts features](https://echarts.apache.org/en/feature.html) and [Canvas versus SVG](https://echarts.apache.org/handbook/en/best-practices/canvas-vs-svg/)

ECharts provides time axes, tooltips, `dataZoom`, brushing, and linked actions. `markArea`, `markLine`, and custom series can show reference intervals and event annotations. The official builder shows that applications can include only required charts and components. ECharts can export chart images through its toolbox and instance APIs. [ECharts builder](https://echarts.apache.org/en/builder.html) and [ECharts API](https://echarts.apache.org/en/api.html)

ECharts can generate ARIA descriptions. Its accessibility module also supports decal patterns. The application must import the ARIA component in a modular build. Generated descriptions do not by themselves give full point-by-point keyboard navigation. An accessible table remains useful. [ECharts ARIA guidance](https://apache.github.io/echarts-handbook/en/best-practices/aria/)

ECharts has no official React component in the Apache project. A small local adapter can own the chart instance and call `setOption`. This approach avoids a dependency on a community wrapper. Chakra theme tokens can generate an ECharts theme or option object. The engine can run fully offline.

### Plotly.js

Plotly.js uses SVG for many traces. It offers WebGL trace types such as `scattergl` for larger point sets. Plotly states that `scattergl` is about one order of magnitude faster than its SVG scatter trace. This is a first-party general claim. WebGL context limits and export differences need a desktop test. [Plotly.js overview](https://plotly.com/javascript/)

Plotly provides date axes, hover labels, zoom, pan, range selectors, annotations, and shapes. Shapes can show reference intervals. It has a mode bar and built-in image export. Complete and partial official bundles exist. Custom bundles can reduce the full package. [Plotly.js repository](https://github.com/plotly/plotly.js) and [Plotly configuration options](https://plotly.com/javascript/configuration-options/)

The official React component uses `Plotly.react`. Its default complete Plotly bundle is more than 2 MB after minification, according to its README. The wrapper uses shallow identity checks for updates. The README warns that the component can mutate `data` and `layout` props after user interaction. These behaviors need care with React state. [Official React wrapper](https://github.com/plotly/react-plotly.js)

Plotly supplies semantic descriptions and keyboard access for some controls, but Canvas and WebGL marks do not become a complete accessible data view. Verify the required keyboard path with the actual trace type. Provide a data table as a fallback.

Plotly can run offline when the bundle and fonts are local. The complete bundle has the largest default download cost in this comparison.

### Vega and Vega-Lite

Vega-Lite compiles a declarative specification to Vega. Vega uses a dataflow runtime and supports Canvas and SVG. Temporal scales support real dates. Layered `rect`, `area`, and `rule` marks can show source reference intervals, personal target ranges, and events. Vega-Lite supports tooltips, facets, concatenated views, and repeated views. [Vega-Lite overview](https://vega.github.io/vega-lite/docs/) and [temporal scales](https://vega.github.io/vega-lite/docs/scale.html)

Interval selections can bind to scales for pan and zoom. Shared scales can synchronize several views. This is useful for aligned small multiples. [Vega-Lite scale binding](https://vega.github.io/vega-lite/docs/bind.html#scale-binding)

Vega can render Canvas or SVG. It can export PNG and SVG in the client. SVG output can contain generated ARIA labels and descriptions. [Vega View API](https://vega.github.io/vega/docs/api/view/) and [Vega accessibility configuration](https://vega.github.io/vega/docs/config/)

The Vega team maintains `react-vega`. Changes to its `spec` or `options` recreate the view. The View API can update data without recreation. This distinction is important for responsive small multiples. [Official React wrapper](https://github.com/vega/react-vega)

The grammar makes complex plots consistent. It also adds compilation and dataflow work. The official sources do not give a benchmark for this product's many time-series points. Test Canvas rendering and update cost with the planned dashboard. All packages can ship locally for offline use.

### Recharts

Recharts is a React-first SVG library. It provides line and area charts, `ReferenceArea`, `ReferenceLine`, `Tooltip`, `Brush`, and responsive containers. A numeric time domain can preserve irregular dates when the x axis uses a numeric scale. Its component model can use Chakra color tokens directly. [Recharts API](https://recharts.github.io/en-US/api/)

Recharts 3 enables its accessibility layer by default. It supports keyboard movement between points and screen-reader feedback through its tooltip. This is the strongest documented point-navigation behavior in this comparison. [Recharts accessibility](https://github.com/recharts/recharts/blob/main/storybook/stories/API/Accessibility.mdx)

SVG creates DOM work for marks and paths. Recharts does not publish a first-party benchmark that proves suitability for very large time-series sets or many dense small multiples. The current mood-tracker use does not prove performance for this product. Downsampling or a different renderer can become necessary when visible data grows.

Recharts works offline. SVG also gives direct vector export through browser APIs, but Recharts does not supply a complete export workflow.

## Comparison

| Criterion | uPlot | ECharts | Plotly.js | Vega and Vega-Lite | Recharts |
| --- | --- | --- | --- | --- | --- |
| Dense time-series performance | Strong first-party evidence | Strong large-data features | WebGL option | Must test | Main risk |
| Many small plots | Low engine cost; custom coordination | Built-in coordination; higher cost | Rich but heavy | Strong composition model | Simple React composition; SVG cost |
| Irregular time axis | Yes | Yes | Yes | Yes | Yes with numeric time scale |
| Changing reference bands | Native bands; custom data shaping | `markArea` or custom series | Shapes or filled traces | Layered rect or area marks | `ReferenceArea` or area data |
| Tooltip | Custom or plug-in | Built in | Built in | Built in or plug-in | Built in |
| Zoom and pan | Hooks and scale changes | `dataZoom` | Built in | Scale-bound selection | Brush; custom wheel zoom |
| Export | Custom Canvas export | Built-in APIs and toolbox | Built in | PNG and SVG APIs | Custom SVG export |
| Accessibility | Application must add it | Generated ARIA and decals | Verify by trace | ARIA with SVG | Default keyboard point navigation |
| React integration | Local adapter or community wrapper | Local adapter or community wrapper | Official wrapper | Official wrapper | Native React components |
| Theme integration | Custom option factory | Theme or option factory | Layout template | Config object | Direct component props |
| Default package cost | Lowest | Medium to high | Highest | Medium | Medium |
| Offline use | Yes | Yes | Yes | Yes | Yes |

## Recommendations for a prototype, not a product decision

1. Benchmark uPlot and ECharts with the same product fixture. These engines cover the two main positions: minimum runtime cost and richer built-in interaction.
2. Include at least 30 small plots. Use irregular dates and changing reference intervals. Test 100, 1,000, 10,000, and 100,000 points per visible plot. The upper levels are stress tests. They are not expected laboratory history sizes.
3. Measure startup time, first plot time, pan latency, cursor latency, memory, and bundle size. Test the packaged desktop build on target hardware.
4. Build the same accessible data table for every candidate. Do not let the chart engine define the clinical accessibility boundary.
5. Test export with reference bands, annotations, dark mode, and local fonts.
6. Test React 19 strict development behavior. Verify cleanup, resize, theme changes, and data replacement.
7. Keep the plot engine behind a narrow application interface. The interface should accept normalized points, source reference intervals, personal target ranges, events, and theme tokens. It should not expose engine option objects to feature code.

Plotly.js is a useful candidate when built-in scientific interaction and export have more value than package size. Vega-Lite is a useful candidate when a shared declarative grammar will support several future chart types. Recharts is a useful baseline for accessibility and implementation speed. The evidence does not justify a final choice without the product fixture benchmark.

## Product benchmark result

Issue 8 implemented both candidates in the throwaway [`prototype/plot-engine-benchmark`](https://github.com/pblan/hemo-tracker/tree/prototype/plot-engine-benchmark/apps/desktop/src/plot-engine-prototype) branch. The benchmark used one headless Chromium session per candidate on the same macOS computer.

| Measurement | uPlot | Apache ECharts | Target |
| --- | ---: | ---: | ---: |
| First render | 43.6 ms | 1,061.4 ms | At most 500 ms |
| Pointer-frame latency, 95th percentile | 10.3 ms | 10.3 ms | At most 50 ms |
| JavaScript heap | 17.8 MB | 100.8 MB | Lower is better |
| Candidate chunk, before compression | 55,491 bytes | 564,569 bytes | Lower is better |
| Theme change | 125.4 ms | 555.2 ms | Recorded, no V1 gate |
| PNG export | Passed | Passed | Must pass |
| Keyboard table path | Passed | Passed | Must pass |
| Canvas cleanup | Passed | Passed | Must pass |

Both engines met the pointer target. Only uPlot met the first-render target. ADR 0004 selects uPlot behind a narrow application adapter. The adapter and the shared accessible table own the product behavior that is not native to the engine.
