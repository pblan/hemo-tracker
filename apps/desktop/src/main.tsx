import React from "react";
import ReactDOM from "react-dom/client";

import { Provider } from "./components/ui/provider";

const isPlotPrototype = new URLSearchParams(window.location.search).has(
  "plot-engine-prototype",
);
const { default: App } = isPlotPrototype
  ? await import("./plot-engine-prototype/PlotEnginePrototype")
  : await import("./App");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider>
      <App />
    </Provider>
  </React.StrictMode>,
);
