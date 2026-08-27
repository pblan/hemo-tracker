import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import App from "./App";
import { Provider } from "./components/ui/provider";

describe("desktop application shell", () => {
  it("identifies the private local-first application", () => {
    render(
      <Provider>
        <App />
      </Provider>,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Hemo Tracker" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/local-first laboratory results/i)).toBeVisible();
  });
});
