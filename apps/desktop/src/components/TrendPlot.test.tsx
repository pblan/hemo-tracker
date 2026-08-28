import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Provider } from "./ui/provider";
import { TrendPlot } from "./TrendPlot";

describe("TrendPlot", () => {
  it("renders a plot with an equivalent table and missing/flagged values", () => {
    render(
      <Provider>
        <TrendPlot
          title="Hemoglobin"
          points={[
            {
              date: "2026-01-01",
              value: 13.7,
              unit: "g/dL",
              sourceValue: "13.7",
              sourceUnit: "g/dL",
              targetStatus: "in target",
            },
            { date: "2026-02-01", value: null, unit: "g/dL", flag: "missing" },
          ]}
        />
      </Provider>,
    );
    expect(
      screen.getByRole("img", { name: "Hemoglobin trend plot" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("table")).toHaveAccessibleName(
      "Accessible data table for Hemoglobin",
    );
    expect(screen.getByText("Missing")).toBeInTheDocument();
    expect(screen.getByText("missing")).toBeInTheDocument();
    expect(screen.getByText("in target")).toBeInTheDocument();
    expect(screen.getAllByText("13.7 g/dL")).toHaveLength(2);
  });

  it("links a table point to its source report", async () => {
    const onOpenReport = vi.fn();
    render(
      <Provider>
        <TrendPlot
          title="Linked trend"
          points={[
            { reportId: "report-1", date: "2026-01-01", value: 1, unit: "g/L" },
          ]}
          onOpenReport={onOpenReport}
        />
      </Provider>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Open report" }));
    expect(onOpenReport).toHaveBeenCalledWith("report-1");
  });

  it("renders a representative local series without failing", () => {
    const points = Array.from({ length: 1000 }, (_, index) => ({
      date: `2026-${String((index % 12) + 1).padStart(2, "0")}-01`,
      value: index % 17 === 0 ? null : index / 10,
      unit: "g/dL",
    }));
    render(
      <Provider>
        <TrendPlot title="Large local series" points={points} />
      </Provider>,
    );
    expect(
      screen.getByRole("img", { name: "Large local series trend plot" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("table")).toBeInTheDocument();
  });
});
