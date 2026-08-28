import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { Provider } from "./components/ui/provider";
import * as vaultClient from "./vault-client";

vi.mock("./vault-client", () => ({
  createLocalAccount: vi.fn(),
  getVaultState: vi.fn(),
  lockVault: vi.fn(),
  addPersonalTargetRange: vi.fn(),
  listAnalyteDefinitions: vi.fn(),
  listLabReports: vi.fn(),
  getLabReport: vi.fn(),
  unlockWithPassphrase: vi.fn(),
  unlockWithRecovery: vi.fn(),
}));

describe("desktop application shell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(vaultClient.getVaultState).mockResolvedValue({
      accountExists: false,
      status: "missing",
    });
    vi.mocked(vaultClient.listAnalyteDefinitions).mockResolvedValue([]);
    vi.mocked(vaultClient.listLabReports).mockResolvedValue([]);
  });

  it("identifies the private local-first application", async () => {
    render(
      <Provider>
        <App />
      </Provider>,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Hemo Tracker" }),
    ).toBeInTheDocument();
    expect(await screen.findByText(/create your local vault/i)).toBeVisible();
  });

  it("creates a local vault and presents its recovery key", async () => {
    vi.mocked(vaultClient.createLocalAccount).mockResolvedValue({
      recoveryCode: "HTRK1-example-recovery-code",
    });
    const user = userEvent.setup();
    render(
      <Provider>
        <App />
      </Provider>,
    );

    await user.type(
      await screen.findByLabelText("Passphrase"),
      "correct horse battery staple",
    );
    await user.type(
      screen.getByLabelText("Confirm passphrase"),
      "correct horse battery staple",
    );
    await user.click(screen.getByRole("button", { name: "Create vault" }));

    expect(vaultClient.createLocalAccount).toHaveBeenCalledWith(
      "correct horse battery staple",
    );
    expect(
      await screen.findByText("HTRK1-example-recovery-code"),
    ).toBeVisible();
    expect(screen.getByText(/store this recovery key/i)).toBeVisible();
  });

  it("unlocks an existing vault with its passphrase", async () => {
    vi.mocked(vaultClient.getVaultState).mockResolvedValue({
      accountExists: true,
      status: "locked",
    });
    vi.mocked(vaultClient.unlockWithPassphrase).mockResolvedValue({
      accountExists: true,
      status: "unlocked",
    });
    vi.mocked(vaultClient.listLabReports).mockResolvedValue(["report-1"]);
    vi.mocked(vaultClient.getLabReport).mockResolvedValue({
      id: "report-1",
      collectionTime: "2026-08-20T08:30:00+02:00",
      laboratory: "Fictional Central Laboratory",
      status: "complete",
      sourceFileCount: 1,
      measurementCount: 2,
      sourceFiles: [],
      measurements: [],
    });
    const user = userEvent.setup();
    render(
      <Provider>
        <App />
      </Provider>,
    );

    await user.type(
      await screen.findByLabelText("Passphrase"),
      "valid passphrase",
    );
    await user.click(screen.getByRole("button", { name: "Unlock vault" }));

    expect(vaultClient.unlockWithPassphrase).toHaveBeenCalledWith(
      "valid passphrase",
    );
    expect(
      await screen.findByRole("heading", { name: "Record a lab report" }),
    ).toBeVisible();
    expect(
      await screen.findByText("Fictional Central Laboratory"),
    ).toBeVisible();
    await user.type(
      screen.getByRole("textbox", { name: "Search reports" }),
      "other",
    );
    expect(
      screen.queryByText("Fictional Central Laboratory"),
    ).not.toBeInTheDocument();
  });

  it("clears a rejected passphrase before the user retries", async () => {
    vi.mocked(vaultClient.getVaultState).mockResolvedValue({
      accountExists: true,
      status: "locked",
    });
    vi.mocked(vaultClient.unlockWithPassphrase).mockRejectedValue(
      new Error("invalid"),
    );
    const user = userEvent.setup();
    render(
      <Provider>
        <App />
      </Provider>,
    );

    const passphrase = await screen.findByLabelText("Passphrase");
    await user.type(passphrase, "wrong passphrase");
    await user.click(screen.getByRole("button", { name: "Unlock vault" }));

    expect(
      await screen.findByText("The passphrase or local vault is invalid."),
    ).toBeVisible();
    expect(passphrase).toHaveValue("");
  });

  it("adds a dated personal target range for a saved analyte", async () => {
    vi.mocked(vaultClient.getVaultState).mockResolvedValue({
      accountExists: true,
      status: "unlocked",
    });
    vi.mocked(vaultClient.listAnalyteDefinitions).mockResolvedValue([
      {
        id: "hemoglobin",
        name: "Hemoglobin",
        component: "Hemoglobin",
        property: "MCnc",
        specimen: "Blood",
        scale: "Quantitative",
        aliases: [],
        personalTargetRanges: [],
      },
    ]);
    vi.mocked(vaultClient.addPersonalTargetRange).mockResolvedValue("range-1");
    const user = userEvent.setup();
    render(
      <Provider>
        <App />
      </Provider>,
    );

    const analyteSelect = await screen.findByLabelText("Target range analyte");
    await within(analyteSelect).findByRole("option", { name: "Hemoglobin" });
    await user.selectOptions(analyteSelect, "hemoglobin");
    await user.type(screen.getByLabelText("Lower limit"), "120");
    await user.type(screen.getByLabelText("Upper limit"), "180");
    await user.type(screen.getByLabelText("Unit"), "g/L");
    await user.type(screen.getByLabelText("Valid from"), "2026-01-01");
    await user.click(
      screen.getByRole("button", { name: "Add personal target range" }),
    );

    expect(vaultClient.addPersonalTargetRange).toHaveBeenCalledWith(
      "hemoglobin",
      expect.objectContaining({
        lowerBound: "120",
        upperBound: "180",
        unit: "g/L",
        validFrom: "2026-01-01",
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Personal target range added.",
    );
  });
});
