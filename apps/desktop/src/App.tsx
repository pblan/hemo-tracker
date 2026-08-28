import {
  Alert,
  Box,
  Button,
  Field,
  Heading,
  Input,
  Stack,
  Text,
} from "@chakra-ui/react";
import { type FormEvent, useEffect, useState } from "react";

import {
  addAnalyteDefinition,
  addLabMeasurement,
  addPersonalTargetRange,
  archiveLabReport,
  chooseAndRestoreLocalVault,
  chooseAndExportPlaintextZip,
  completeLabReport,
  correctLabMeasurement,
  chooseAndBackupLocalVault,
  createLabReport,
  createLocalAccount,
  getVaultState,
  listAnalyteDefinitions,
  listLabReports,
  getLabReport,
  readSourceFile,
  type ReportSummary,
  lockVault,
  permanentlyDeleteLabReport,
  selectAndAttachSourceFile,
  unlockWithPassphrase,
  unlockWithRecovery,
  type VaultState,
} from "./vault-client";
import { parseMeasurementInput } from "./measurement-parser";
import { validateMeasurementRow } from "./measurement-validation";
import { validatePersonalTargetRange } from "./personal-target-range-validation";
import {
  normalizeMeasurement,
  resolveApplicableTargetRange,
} from "./measurement-normalization";
import { TrendPlot } from "./components/TrendPlot";

function App() {
  const [vaultState, setVaultState] = useState<VaultState | null>(null);
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void getVaultState()
      .then(setVaultState)
      .catch(() => {
        setError("Hemo Tracker could not read the local vault state.");
      });
  }, []);

  return (
    <Box
      as="main"
      minH="100vh"
      bg="bg"
      color="fg"
      px={{ base: "6", md: "10" }}
      py="12"
    >
      <Stack gap="8" maxW="3xl" mx="auto">
        <Stack gap="3">
          <Text
            color="teal.600"
            fontSize="sm"
            fontWeight="semibold"
            letterSpacing="wide"
            textTransform="uppercase"
          >
            Local laboratory tracking
          </Text>
          <Heading as="h1" size="4xl">
            Hemo Tracker
          </Heading>
          <Text color="fg.muted" fontSize="lg">
            Your laboratory data stays in an encrypted vault on this computer.
          </Text>
        </Stack>
        {error ? (
          <Alert.Root status="error">
            <Alert.Indicator />
            <Alert.Title>{error}</Alert.Title>
          </Alert.Root>
        ) : null}
        {!vaultState ? <Text>Read local vault…</Text> : null}
        {vaultState?.status === "missing" && !recoveryCode ? (
          <CreateVault
            onCreated={(code) => {
              setRecoveryCode(code);
              setVaultState({ accountExists: true, status: "unlocked" });
            }}
            onError={setError}
          />
        ) : null}
        {recoveryCode ? (
          <RecoveryKey
            code={recoveryCode}
            onConfirmed={() => setRecoveryCode(null)}
          />
        ) : null}
        {vaultState?.status === "locked" ? (
          <UnlockVault onUnlocked={setVaultState} onError={setError} />
        ) : null}
        {vaultState?.status === "unlocked" && !recoveryCode ? (
          <UnlockedVault onLocked={setVaultState} onError={setError} />
        ) : null}
      </Stack>
    </Box>
  );
}

function CreateVault({
  onCreated,
  onError,
}: {
  onCreated: (code: string) => void;
  onError: (message: string) => void;
}) {
  const [passphrase, setPassphrase] = useState("");
  const [confirmation, setConfirmation] = useState("");
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (passphrase !== confirmation)
      return onError("The passphrases do not match.");
    try {
      const created = await createLocalAccount(passphrase);
      setPassphrase("");
      setConfirmation("");
      onCreated(created.recoveryCode);
    } catch {
      onError("Hemo Tracker could not create the local vault.");
    }
  }
  return (
    <Box borderWidth="1px" borderRadius="xl" p={{ base: "5", md: "7" }}>
      <Stack as="form" gap="5" onSubmit={submit}>
        <Stack gap="2">
          <Heading as="h2" size="xl">
            Create your local vault
          </Heading>
          <Text color="fg.muted">
            Use a unique passphrase. Hemo Tracker cannot recover it for you.
          </Text>
        </Stack>
        <Field.Root required>
          <Field.Label>Passphrase</Field.Label>
          <Input
            type="password"
            autoComplete="new-password"
            value={passphrase}
            onChange={(event) => setPassphrase(event.target.value)}
          />
        </Field.Root>
        <Field.Root required>
          <Field.Label>Confirm passphrase</Field.Label>
          <Input
            type="password"
            autoComplete="new-password"
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
          />
        </Field.Root>
        <Button type="submit" alignSelf="start" colorPalette="teal">
          Create vault
        </Button>
      </Stack>
    </Box>
  );
}

function RecoveryKey({
  code,
  onConfirmed,
}: {
  code: string;
  onConfirmed: () => void;
}) {
  return (
    <Box borderWidth="1px" borderRadius="xl" p={{ base: "5", md: "7" }}>
      <Stack gap="5">
        <Heading as="h2" size="xl">
          Store your recovery key
        </Heading>
        <Text>
          Store this recovery key separately from this computer. You need it if
          you forget the passphrase.
        </Text>
        <Box
          as="code"
          overflowWrap="anywhere"
          bg="bg.muted"
          borderRadius="md"
          p="4"
        >
          {code}
        </Box>
        <Button alignSelf="start" onClick={onConfirmed}>
          I stored the recovery key
        </Button>
      </Stack>
    </Box>
  );
}

function UnlockVault({
  onUnlocked,
  onError,
}: {
  onUnlocked: (state: VaultState) => void;
  onError: (message: string) => void;
}) {
  const [passphrase, setPassphrase] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  async function withPassphrase(event: FormEvent) {
    event.preventDefault();
    onError("");
    try {
      onUnlocked(await unlockWithPassphrase(passphrase));
    } catch {
      onError("The passphrase or local vault is invalid.");
    } finally {
      setPassphrase("");
    }
  }
  async function withRecovery(event: FormEvent) {
    event.preventDefault();
    onError("");
    try {
      onUnlocked(await unlockWithRecovery(recoveryCode));
    } catch {
      onError("The recovery key or local vault is invalid.");
    } finally {
      setRecoveryCode("");
    }
  }
  return (
    <Stack gap="6">
      <Box
        as="form"
        borderWidth="1px"
        borderRadius="xl"
        p="6"
        onSubmit={withPassphrase}
      >
        <Stack gap="4">
          <Heading as="h2" size="xl">
            Unlock your vault
          </Heading>
          <Field.Root required>
            <Field.Label>Passphrase</Field.Label>
            <Input
              type="password"
              value={passphrase}
              onChange={(event) => setPassphrase(event.target.value)}
            />
          </Field.Root>
          <Button type="submit" alignSelf="start" colorPalette="teal">
            Unlock vault
          </Button>
        </Stack>
      </Box>
      <Box
        as="form"
        borderWidth="1px"
        borderRadius="xl"
        p="6"
        onSubmit={withRecovery}
      >
        <Stack gap="4">
          <Heading as="h2" size="lg">
            Use recovery key
          </Heading>
          <Field.Root required>
            <Field.Label>Recovery key</Field.Label>
            <Input
              value={recoveryCode}
              onChange={(event) => setRecoveryCode(event.target.value)}
            />
          </Field.Root>
          <Button type="submit" alignSelf="start" variant="outline">
            Unlock with recovery key
          </Button>
        </Stack>
      </Box>
    </Stack>
  );
}

function UnlockedVault({
  onLocked,
  onError,
}: {
  onLocked: (state: VaultState) => void;
  onError: (message: string) => void;
}) {
  const [collectionTime, setCollectionTime] = useState("");
  const [laboratory, setLaboratory] = useState("");
  const [sourceLabel, setSourceLabel] = useState("");
  const [sourceValue, setSourceValue] = useState("");
  const [valueHint, setValueHint] = useState<string | null>(null);
  const [formatConfirmed, setFormatConfirmed] = useState(false);
  const [sourceUnit, setSourceUnit] = useState("");
  const [sourceReferenceInterval, setSourceReferenceInterval] = useState("");
  const [sourceFlag, setSourceFlag] = useState("");
  const [extraMeasurement, setExtraMeasurement] = useState(false);
  const [secondLabel, setSecondLabel] = useState("");
  const [secondValue, setSecondValue] = useState("");
  const [secondUnit, setSecondUnit] = useState("");
  const [secondInterval, setSecondInterval] = useState("");
  const [secondFlag, setSecondFlag] = useState("");
  const [analyteName, setAnalyteName] = useState("");
  const [analyteComponent, setAnalyteComponent] = useState("");
  const [analyteProperty, setAnalyteProperty] = useState("");
  const [analytes, setAnalytes] = useState<
    Awaited<ReturnType<typeof listAnalyteDefinitions>>
  >([]);
  const [selectedAnalyteId, setSelectedAnalyteId] = useState("");
  const [sourceFilename, setSourceFilename] = useState<string | null>(null);
  const [sourceRole, setSourceRole] = useState("primary");
  const [saving, setSaving] = useState(false);
  const [dataVersion, setDataVersion] = useState(0);
  const [reports, setReports] = useState<ReportSummary[]>([]);
  const [reportSearch, setReportSearch] = useState("");
  const [expandedReportId, setExpandedReportId] = useState<string | null>(null);
  const [editingMeasurement, setEditingMeasurement] = useState<string | null>(
    null,
  );
  const [correctionValue, setCorrectionValue] = useState("");
  const [correctionAnalyteId, setCorrectionAnalyteId] = useState("");
  const [trendAnalyteId, setTrendAnalyteId] = useState("");
  const [compareAnalyteId, setCompareAnalyteId] = useState("");
  const [pinnedAnalyteIds, setPinnedAnalyteIds] = useState<string[]>(() => {
    try {
      const stored = localStorage.getItem("hemo-tracker:pinned-analytes");
      const parsed: unknown = stored ? JSON.parse(stored) : [];
      return Array.isArray(parsed) &&
        parsed.every((id) => typeof id === "string")
        ? parsed.slice(0, 6)
        : [];
    } catch {
      return [];
    }
  });
  const [rangeAnalyteId, setRangeAnalyteId] = useState("");
  const [rangeLower, setRangeLower] = useState("");
  const [rangeUpper, setRangeUpper] = useState("");
  const [rangeUnit, setRangeUnit] = useState("");
  const [rangeValidFrom, setRangeValidFrom] = useState("");
  const [rangeValidTo, setRangeValidTo] = useState("");
  const [rangeContext, setRangeContext] = useState("");
  const [rangeNotes, setRangeNotes] = useState("");
  const [rangeMessage, setRangeMessage] = useState("");
  const [restorePassphrase, setRestorePassphrase] = useState("");
  const [sourcePreview, setSourcePreview] = useState<{
    filename: string;
    mediaType: string;
    url: string;
  } | null>(null);
  const [addingMeasurementReportId, setAddingMeasurementReportId] = useState<
    string | null
  >(null);
  const [additionalLabel, setAdditionalLabel] = useState("");
  const [additionalValue, setAdditionalValue] = useState("");
  const [additionalUnit, setAdditionalUnit] = useState("");

  useEffect(() => {
    void listAnalyteDefinitions()
      .then(setAnalytes)
      .catch(() => undefined);
  }, []);
  useEffect(() => {
    void listLabReports()
      .then((ids) => Promise.all(ids.map((id) => getLabReport(id))))
      .then(setReports)
      .catch(() => undefined);
  }, [saving, dataVersion]);
  useEffect(() => {
    try {
      if (typeof localStorage?.setItem === "function")
        localStorage.setItem(
          "hemo-tracker:pinned-analytes",
          JSON.stringify(pinnedAnalyteIds),
        );
    } catch {
      // Local preferences are optional and must not block vault use.
    }
  }, [pinnedAnalyteIds]);

  async function saveReport(event: FormEvent) {
    event.preventDefault();
    setRangeMessage("");
    const validationErrors = validateMeasurementRow({
      sourceLabel,
      sourceValue,
      sourceUnit,
      sourceReferenceInterval,
      sourceFlag,
    });
    if (validationErrors.length) {
      onError(validationErrors[0] ?? "Complete the measurement fields.");
      return;
    }
    if (
      extraMeasurement &&
      validateMeasurementRow({
        sourceLabel: secondLabel,
        sourceValue: secondValue,
        sourceUnit: secondUnit,
        sourceReferenceInterval: secondInterval,
        sourceFlag: secondFlag,
      }).length
    ) {
      onError("Complete all fields in the second measurement row.");
      return;
    }
    if (valueHint?.startsWith("This value") && !formatConfirmed) {
      onError("Confirm the measurement format before saving.");
      return;
    }
    setSaving(true);
    onError("");
    try {
      const reportId = await createLabReport({
        collectionTime,
        laboratory: laboratory || undefined,
        tags: [],
      });
      const source = await selectAndAttachSourceFile(reportId, sourceRole);
      if (!source) throw new Error("source file not selected");
      setSourceFilename(source.originalFilename);
      const analyteId =
        selectedAnalyteId ||
        (await addAnalyteDefinition({
          name: analyteName || sourceLabel,
          component: analyteComponent || analyteName || sourceLabel,
          property: analyteProperty || "Result",
          specimen: "Blood",
          scale: "Quantitative",
          aliases: [],
          canonicalUnit: sourceUnit,
          personalTargetRanges: [],
        }));
      const parsedSourceValue = parseMeasurementInput(sourceValue, "de-DE");
      await addLabMeasurement(reportId, {
        sourceLabel,
        sourceValue,
        sourceUnit,
        sourceReferenceInterval,
        sourceFlag,
        parsedNumericValue:
          parsedSourceValue.kind === "number"
            ? parsedSourceValue.normalized
            : undefined,
        analyteId,
      });
      if (extraMeasurement) {
        const parsedSecondValue = parseMeasurementInput(secondValue, "de-DE");
        await addLabMeasurement(reportId, {
          sourceLabel: secondLabel,
          sourceValue: secondValue,
          sourceUnit: secondUnit,
          sourceReferenceInterval: secondInterval,
          sourceFlag: secondFlag,
          parsedNumericValue:
            parsedSecondValue.kind === "number"
              ? parsedSecondValue.normalized
              : undefined,
          analyteId,
        });
      }
      await completeLabReport(reportId);
      setAnalytes(await listAnalyteDefinitions());
      setCollectionTime("");
      setLaboratory("");
      setSourceLabel("");
      setSourceValue("");
      setSourceUnit("");
      setSourceReferenceInterval("");
      setSourceFlag("");
    } catch {
      onError("Hemo Tracker could not save the lab report.");
    } finally {
      setSaving(false);
    }
  }

  async function savePersonalTargetRange(event: FormEvent) {
    event.preventDefault();
    const validationErrors = validatePersonalTargetRange({
      analyteId: rangeAnalyteId,
      lowerBound: rangeLower,
      upperBound: rangeUpper,
      unit: rangeUnit,
      validFrom: rangeValidFrom,
      validTo: rangeValidTo,
    });
    if (validationErrors.length) {
      onError(validationErrors[0] ?? "Complete the personal target range.");
      return;
    }
    onError("");
    try {
      await addPersonalTargetRange(rangeAnalyteId, {
        lowerBound: rangeLower || undefined,
        upperBound: rangeUpper || undefined,
        unit: rangeUnit,
        validFrom: rangeValidFrom || undefined,
        validTo: rangeValidTo || undefined,
        context: rangeContext || undefined,
        notes: rangeNotes || undefined,
      });
      setAnalytes(await listAnalyteDefinitions());
      setRangeLower("");
      setRangeUpper("");
      setRangeUnit("");
      setRangeValidFrom("");
      setRangeValidTo("");
      setRangeContext("");
      setRangeNotes("");
      setRangeMessage("Personal target range added.");
    } catch {
      onError("Hemo Tracker could not save the personal target range.");
    }
  }

  async function backupVault() {
    try {
      await chooseAndBackupLocalVault();
    } catch {
      onError("Hemo Tracker could not create the encrypted backup.");
    }
  }

  async function restoreVault() {
    if (!restorePassphrase) {
      onError("Enter the backup passphrase before you restore.");
      return;
    }
    try {
      const restored = await chooseAndRestoreLocalVault(restorePassphrase);
      if (restored) onError("The encrypted backup was restored.");
      setRestorePassphrase("");
    } catch {
      onError("Hemo Tracker could not restore that encrypted backup.");
    }
  }

  async function exportPlaintext() {
    if (
      !window.confirm(
        "This creates a plaintext copy of your health data. Continue?",
      )
    )
      return;
    try {
      const exported = await chooseAndExportPlaintextZip();
      if (exported)
        onError(
          "The plaintext JSON export was saved. Protect or delete it when it is no longer needed.",
        );
    } catch {
      onError("Hemo Tracker could not create the plaintext export.");
    }
  }

  async function saveCorrection(
    measurement: ReportSummary["measurements"][number],
  ) {
    try {
      const parsedCorrection = parseMeasurementInput(correctionValue, "de-DE");
      await correctLabMeasurement(measurement.id, {
        sourceLabel: measurement.sourceLabel,
        sourceValue: correctionValue,
        sourceUnit: measurement.sourceUnit,
        sourceReferenceInterval: measurement.sourceReferenceInterval,
        sourceFlag: measurement.sourceFlag,
        parsedNumericValue:
          parsedCorrection.kind === "number"
            ? parsedCorrection.normalized
            : undefined,
        analyteId: correctionAnalyteId || measurement.analyteId,
      });
      setEditingMeasurement(null);
      setCorrectionAnalyteId("");
      const ids = await listLabReports();
      setReports(await Promise.all(ids.map((id) => getLabReport(id))));
    } catch {
      onError("Hemo Tracker could not save the correction.");
    }
  }

  async function lock() {
    try {
      onLocked(await lockVault());
    } catch {
      onError("Hemo Tracker could not lock the local vault.");
    }
  }
  async function previewSource(reportId: string, sourceFileId: string) {
    try {
      const content = await readSourceFile(reportId, sourceFileId);
      const url = URL.createObjectURL(
        new Blob([new Uint8Array(content.bytes)], { type: content.mediaType }),
      );
      setSourcePreview((current) => {
        if (current) URL.revokeObjectURL(current.url);
        return {
          filename: content.filename,
          mediaType: content.mediaType,
          url,
        };
      });
    } catch {
      onError("Hemo Tracker could not open the encrypted source file.");
    }
  }
  async function saveAdditionalMeasurement(reportId: string) {
    const validationErrors = validateMeasurementRow({
      sourceLabel: additionalLabel,
      sourceValue: additionalValue,
      sourceUnit: additionalUnit,
      sourceReferenceInterval: "",
      sourceFlag: "",
    });
    if (validationErrors.length) {
      onError(validationErrors[0] ?? "Complete the measurement fields.");
      return;
    }
    try {
      const parsed = parseMeasurementInput(additionalValue, "de-DE");
      await addLabMeasurement(reportId, {
        sourceLabel: additionalLabel,
        sourceValue: additionalValue,
        sourceUnit: additionalUnit,
        sourceReferenceInterval: "",
        sourceFlag: "",
        parsedNumericValue:
          parsed.kind === "number" ? parsed.normalized : undefined,
        analyteId: selectedAnalyteId || undefined,
      });
      setAddingMeasurementReportId(null);
      setAdditionalLabel("");
      setAdditionalValue("");
      setAdditionalUnit("");
      setDataVersion((value) => value + 1);
    } catch {
      onError("Hemo Tracker could not save the measurement.");
    }
  }
  async function archiveReport(reportId: string) {
    try {
      await archiveLabReport(reportId);
      setDataVersion((value) => value + 1);
    } catch {
      onError("Hemo Tracker could not archive the report.");
    }
  }
  async function permanentlyDeleteReport(reportId: string) {
    if (
      !window.confirm(
        "Permanently delete this archived report and its encrypted source files? This cannot be undone.",
      )
    )
      return;
    try {
      await permanentlyDeleteLabReport(reportId, true);
      setExpandedReportId(null);
      setDataVersion((value) => value + 1);
    } catch {
      onError("Hemo Tracker could not permanently delete the report.");
    }
  }
  const buildTrend = (analyteId: string) => {
    const analyte = analytes.find((item) => item.id === analyteId);
    const candidates = reports.flatMap((report) =>
      report.measurements
        .filter((measurement) => measurement.analyteId === analyteId)
        .map((measurement) => ({ report, measurement })),
    );
    const points = candidates.flatMap(({ report, measurement }) => {
      const normalized = normalizeMeasurement(measurement, analyte);
      return normalized.status === "normalized"
        ? [
            {
              id: measurement.id,
              reportId: report.id,
              date: report.collectionTime,
              value: normalized.value,
              unit: normalized.unit,
              sourceValue: measurement.sourceValue,
              sourceUnit: measurement.sourceUnit,
              flag: measurement.sourceFlag,
              targetStatus: (() => {
                if (!analyte) return undefined;
                const target = resolveApplicableTargetRange(
                  report.collectionTime,
                  analyte.personalTargetRanges,
                  analyte,
                );
                if (target.status !== "applicable") return undefined;
                if (
                  target.lowerBound !== undefined &&
                  normalized.value < target.lowerBound
                )
                  return "below target" as const;
                if (
                  target.upperBound !== undefined &&
                  normalized.value > target.upperBound
                )
                  return "above target" as const;
                return "in target" as const;
              })(),
            },
          ]
        : [];
    });
    return { points, excluded: candidates.length - points.length };
  };
  const trend = buildTrend(trendAnalyteId);
  const comparison = buildTrend(compareAnalyteId);
  const openReportFromTrend = (reportId: string) => {
    setExpandedReportId(reportId);
    requestAnimationFrame(() =>
      document.getElementById(`report-${reportId}`)?.scrollIntoView({
        behavior: "smooth",
        block: "center",
      }),
    );
  };
  const togglePinnedAnalyte = (analyteId: string) => {
    setPinnedAnalyteIds((current) => {
      if (current.includes(analyteId))
        return current.filter((id) => id !== analyteId);
      return current.length < 6 ? [...current, analyteId] : current;
    });
  };

  return (
    <Stack gap="6">
      <Box
        bg="bg.panel"
        borderWidth="1px"
        borderColor="border"
        borderRadius="2xl"
        p={{ base: "5", md: "6" }}
      >
        <Stack gap="3">
          <Heading as="h2" size="lg">
            Analyte trend
          </Heading>
          <select
            aria-label="Trend analyte"
            value={trendAnalyteId}
            onChange={(event) => setTrendAnalyteId(event.target.value)}
          >
            <option value="">Select an analyte</option>
            {analytes.map((analyte) => (
              <option key={analyte.id} value={analyte.id}>
                {analyte.name}
              </option>
            ))}
          </select>
          <Stack gap="2" aria-label="Pinned analytes">
            <Text fontWeight="semibold" fontSize="sm">
              Pin analytes for this overview (up to 6)
            </Text>
            <Stack
              direction={{ base: "column", sm: "row" }}
              wrap="wrap"
              gap="2"
            >
              {analytes.map((analyte) => (
                <label key={analyte.id}>
                  <input
                    type="checkbox"
                    checked={pinnedAnalyteIds.includes(analyte.id)}
                    onChange={() => togglePinnedAnalyte(analyte.id)}
                    disabled={
                      !pinnedAnalyteIds.includes(analyte.id) &&
                      pinnedAnalyteIds.length >= 6
                    }
                  />{" "}
                  {analyte.name}
                </label>
              ))}
            </Stack>
          </Stack>
          <select
            aria-label="Compare analyte"
            value={compareAnalyteId}
            onChange={(event) => setCompareAnalyteId(event.target.value)}
          >
            <option value="">Compare with another analyte (optional)</option>
            {analytes
              .filter((analyte) => analyte.id !== trendAnalyteId)
              .map((analyte) => (
                <option key={analyte.id} value={analyte.id}>
                  {analyte.name}
                </option>
              ))}
          </select>
          {trendAnalyteId || pinnedAnalyteIds.length ? (
            <Stack gap="4">
              {trendAnalyteId ? (
                <Stack gap="2">
                  <TrendPlot
                    title="Local analyte trend"
                    points={trend.points}
                    onOpenReport={openReportFromTrend}
                  />
                  {trend.excluded ? (
                    <Text color="orange.700" fontSize="sm" role="status">
                      {trend.excluded} result could not be normalized and is not
                      connected to this series.
                    </Text>
                  ) : null}
                </Stack>
              ) : null}
              {trendAnalyteId && compareAnalyteId ? (
                <Stack gap="2">
                  <TrendPlot
                    title="Compared analyte trend"
                    points={comparison.points}
                    onOpenReport={openReportFromTrend}
                  />
                  {comparison.excluded ? (
                    <Text color="orange.700" fontSize="sm" role="status">
                      {comparison.excluded} comparison result could not be
                      normalized and is not connected to this series.
                    </Text>
                  ) : null}
                </Stack>
              ) : null}
              {pinnedAnalyteIds.length ? (
                <Stack gap="4" aria-label="Pinned analyte overview">
                  <Heading as="h3" size="md">
                    Pinned analytes
                  </Heading>
                  {pinnedAnalyteIds.map((analyteId) => {
                    const pinned = analytes.find(
                      (item) => item.id === analyteId,
                    );
                    const pinnedTrend = buildTrend(analyteId);
                    return pinned ? (
                      <Stack key={analyteId} gap="2">
                        <TrendPlot
                          title={pinned.name}
                          points={pinnedTrend.points}
                          onOpenReport={openReportFromTrend}
                        />
                        {pinnedTrend.excluded ? (
                          <Text color="orange.700" fontSize="sm" role="status">
                            {pinnedTrend.excluded} result could not be
                            normalized for this pinned series.
                          </Text>
                        ) : null}
                      </Stack>
                    ) : null;
                  })}
                </Stack>
              ) : null}
            </Stack>
          ) : (
            <Text color="fg.muted" fontSize="sm">
              Select an analyte to view recorded numeric values.
            </Text>
          )}
        </Stack>
      </Box>
      <Box
        as="form"
        bg="bg.panel"
        borderWidth="1px"
        borderColor="border"
        borderRadius="2xl"
        p={{ base: "5", md: "6" }}
        onSubmit={savePersonalTargetRange}
      >
        <Stack gap="4">
          <Stack gap="1">
            <Heading as="h2" size="lg">
              Personal target ranges
            </Heading>
            <Text color="fg.muted" fontSize="sm">
              These ranges are informational. They do not replace the source
              laboratory interval or medical advice.
            </Text>
          </Stack>
          <Field.Root required>
            <Field.Label>Analyte</Field.Label>
            <select
              aria-label="Target range analyte"
              value={rangeAnalyteId}
              onChange={(event) => setRangeAnalyteId(event.target.value)}
            >
              <option value="">Select an analyte</option>
              {analytes.map((analyte) => (
                <option key={analyte.id} value={analyte.id}>
                  {analyte.name}
                </option>
              ))}
            </select>
          </Field.Root>
          <Stack direction={{ base: "column", sm: "row" }} gap="3">
            <Field.Root>
              <Field.Label>Lower limit</Field.Label>
              <Input
                inputMode="decimal"
                value={rangeLower}
                onChange={(event) => setRangeLower(event.target.value)}
              />
            </Field.Root>
            <Field.Root>
              <Field.Label>Upper limit</Field.Label>
              <Input
                inputMode="decimal"
                value={rangeUpper}
                onChange={(event) => setRangeUpper(event.target.value)}
              />
            </Field.Root>
            <Field.Root required>
              <Field.Label>Unit</Field.Label>
              <Input
                value={rangeUnit}
                onChange={(event) => setRangeUnit(event.target.value)}
              />
            </Field.Root>
          </Stack>
          <Stack direction={{ base: "column", sm: "row" }} gap="3">
            <Field.Root>
              <Field.Label>Valid from</Field.Label>
              <Input
                type="date"
                value={rangeValidFrom}
                onChange={(event) => setRangeValidFrom(event.target.value)}
              />
            </Field.Root>
            <Field.Root>
              <Field.Label>Valid to</Field.Label>
              <Input
                type="date"
                value={rangeValidTo}
                onChange={(event) => setRangeValidTo(event.target.value)}
              />
            </Field.Root>
          </Stack>
          <Stack direction={{ base: "column", sm: "row" }} gap="3">
            <Field.Root>
              <Field.Label>Applicability note</Field.Label>
              <Input
                placeholder="For example, fasting"
                value={rangeContext}
                onChange={(event) => setRangeContext(event.target.value)}
              />
            </Field.Root>
            <Field.Root>
              <Field.Label>Personal note</Field.Label>
              <Input
                value={rangeNotes}
                onChange={(event) => setRangeNotes(event.target.value)}
              />
            </Field.Root>
          </Stack>
          <Button type="submit" alignSelf="start" colorPalette="teal">
            Add personal target range
          </Button>
          {rangeMessage ? <Text role="status">{rangeMessage}</Text> : null}
          {rangeAnalyteId ? (
            <Stack gap="2" aria-label="Saved personal target ranges">
              {analytes
                .find((analyte) => analyte.id === rangeAnalyteId)
                ?.personalTargetRanges.map((range) => (
                  <Text key={range.id} fontSize="sm">
                    {range.lowerBound || "No lower limit"} to{" "}
                    {range.upperBound || "no upper limit"} {range.unit}
                    {range.validFrom ? ` from ${range.validFrom}` : ""}
                    {range.validTo ? ` to ${range.validTo}` : ""}
                    {range.context ? ` · ${range.context}` : ""}
                    {range.notes ? ` · ${range.notes}` : ""}
                  </Text>
                ))}
            </Stack>
          ) : null}
        </Stack>
      </Box>
      <Box
        bg="bg.panel"
        borderWidth="1px"
        borderColor="border"
        borderRadius="2xl"
        p={{ base: "5", md: "6" }}
        shadow="sm"
      >
        <Stack gap="4">
          <Stack
            direction={{ base: "column", sm: "row" }}
            justify="space-between"
            align={{ base: "stretch", sm: "center" }}
          >
            <Stack gap="0">
              <Heading as="h2" size="lg">
                Report history
              </Heading>
              <Text color="fg.muted" fontSize="sm">
                Stored only in this encrypted vault.
              </Text>
            </Stack>
            <Input
              aria-label="Search reports"
              placeholder="Search laboratory"
              maxW="sm"
              value={reportSearch}
              onChange={(event) => setReportSearch(event.target.value)}
            />
          </Stack>
          {reports
            .filter(
              (report) =>
                !reportSearch ||
                report.laboratory
                  ?.toLowerCase()
                  .includes(reportSearch.toLowerCase()),
            )
            .slice(0, 5)
            .map((report) => (
              <Box
                key={report.id}
                id={`report-${report.id}`}
                borderWidth="1px"
                borderColor="border"
                borderRadius="lg"
                px="4"
                py="3"
                cursor="pointer"
                onClick={() =>
                  setExpandedReportId((current) =>
                    current === report.id ? null : report.id,
                  )
                }
              >
                <Stack direction="row" justify="space-between" align="center">
                  <Stack gap="0">
                    <Text fontWeight="semibold">
                      {report.laboratory || "Laboratory report"}
                    </Text>
                    <Text color="fg.muted" fontSize="sm">
                      {report.collectionTime} · {report.measurementCount}{" "}
                      measurements · {report.sourceFileCount} source files
                    </Text>
                  </Stack>
                  <Text fontSize="sm" textTransform="capitalize">
                    {report.status}
                  </Text>
                </Stack>
                {expandedReportId === report.id ? (
                  <Stack
                    gap="2"
                    mt="3"
                    pt="3"
                    borderTopWidth="1px"
                    borderColor="border"
                  >
                    <Stack direction={{ base: "column", sm: "row" }} gap="2">
                      <select
                        aria-label={`Source file role for ${report.laboratory || "report"}`}
                        value={sourceRole}
                        onChange={(event) => setSourceRole(event.target.value)}
                      >
                        <option value="supplement">Supplement</option>
                        <option value="correction">Correction</option>
                        <option value="primary">Primary</option>
                      </select>
                      <Button
                        size="xs"
                        variant="outline"
                        onClick={(event) => {
                          event.stopPropagation();
                          void selectAndAttachSourceFile(
                            report.id,
                            sourceRole,
                          ).then((source) => {
                            if (source) setDataVersion((value) => value + 1);
                          });
                        }}
                      >
                        Attach source file
                      </Button>
                      <Button
                        size="xs"
                        variant="outline"
                        onClick={(event) => {
                          event.stopPropagation();
                          setAddingMeasurementReportId(report.id);
                        }}
                      >
                        Add measurement
                      </Button>
                      {report.status === "complete" ? (
                        <Button
                          size="xs"
                          variant="ghost"
                          onClick={(event) => {
                            event.stopPropagation();
                            void archiveReport(report.id);
                          }}
                        >
                          Archive report
                        </Button>
                      ) : null}
                      {report.status === "archived" ? (
                        <Button
                          size="xs"
                          variant="ghost"
                          colorPalette="red"
                          onClick={(event) => {
                            event.stopPropagation();
                            void permanentlyDeleteReport(report.id);
                          }}
                        >
                          Permanently delete
                        </Button>
                      ) : null}
                    </Stack>
                    {addingMeasurementReportId === report.id ? (
                      <Stack direction={{ base: "column", sm: "row" }} gap="2">
                        <Input
                          aria-label="Additional measurement label"
                          placeholder="Analyte label"
                          value={additionalLabel}
                          onChange={(event) =>
                            setAdditionalLabel(event.target.value)
                          }
                        />
                        <Input
                          aria-label="Additional measurement value"
                          placeholder="Value"
                          value={additionalValue}
                          onChange={(event) =>
                            setAdditionalValue(event.target.value)
                          }
                        />
                        <Input
                          aria-label="Additional measurement unit"
                          placeholder="Unit"
                          value={additionalUnit}
                          onChange={(event) =>
                            setAdditionalUnit(event.target.value)
                          }
                        />
                        <Button
                          size="xs"
                          onClick={() =>
                            void saveAdditionalMeasurement(report.id)
                          }
                        >
                          Save
                        </Button>
                      </Stack>
                    ) : null}
                    {report.sourceFiles.map((source) => (
                      <Button
                        key={source.id}
                        size="xs"
                        variant="outline"
                        alignSelf="start"
                        onClick={(event) => {
                          event.stopPropagation();
                          void previewSource(report.id, source.id);
                        }}
                      >
                        Preview {source.filename}
                      </Button>
                    ))}
                    {report.measurements.map((measurement) => (
                      <Stack key={measurement.id} gap="1">
                        {editingMeasurement === measurement.id ? (
                          <Stack direction="row" gap="2">
                            <Input
                              aria-label={`Correct ${measurement.sourceLabel}`}
                              value={correctionValue}
                              onChange={(event) =>
                                setCorrectionValue(event.target.value)
                              }
                            />
                            <select
                              aria-label={`Correct analyte for ${measurement.sourceLabel}`}
                              value={correctionAnalyteId}
                              onChange={(event) =>
                                setCorrectionAnalyteId(event.target.value)
                              }
                            >
                              <option value="">Keep current analyte</option>
                              {analytes.map((analyte) => (
                                <option key={analyte.id} value={analyte.id}>
                                  {analyte.name}
                                </option>
                              ))}
                            </select>
                            <Button
                              size="sm"
                              onClick={() => void saveCorrection(measurement)}
                            >
                              Save
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => setEditingMeasurement(null)}
                            >
                              Cancel
                            </Button>
                          </Stack>
                        ) : (
                          <Stack
                            direction="row"
                            justify="space-between"
                            align="center"
                          >
                            <Text fontSize="sm">
                              {measurement.sourceLabel}:{" "}
                              {measurement.sourceValue} {measurement.sourceUnit}
                              {measurement.sourceFlag
                                ? ` (${measurement.sourceFlag})`
                                : ""}
                            </Text>
                            {measurement.updatedAt ? (
                              <Text fontSize="xs" color="fg.muted">
                                Corrected {measurement.updatedAt} by{" "}
                                {measurement.updatedBy}
                              </Text>
                            ) : null}
                            <Button
                              size="xs"
                              variant="ghost"
                              onClick={(event) => {
                                event.stopPropagation();
                                setEditingMeasurement(measurement.id);
                                setCorrectionValue(measurement.sourceValue);
                                setCorrectionAnalyteId(
                                  measurement.analyteId || "",
                                );
                              }}
                            >
                              Correct
                            </Button>
                          </Stack>
                        )}
                      </Stack>
                    ))}
                    {!report.measurements.length ? (
                      <Text fontSize="sm" color="fg.muted">
                        No measurements recorded.
                      </Text>
                    ) : null}
                  </Stack>
                ) : null}
              </Box>
            ))}
          {!reports.length ? (
            <Text color="fg.muted" fontSize="sm">
              No reports recorded yet.
            </Text>
          ) : null}
          {sourcePreview ? (
            <Box borderWidth="1px" borderRadius="lg" p="3">
              <Text fontWeight="semibold" mb="2">
                Source preview: {sourcePreview.filename}
              </Text>
              {sourcePreview.mediaType === "application/pdf" ? (
                <iframe
                  title={`Preview of ${sourcePreview.filename}`}
                  src={sourcePreview.url}
                  width="100%"
                  height="480"
                />
              ) : sourcePreview.mediaType.startsWith("image/") ? (
                <img
                  src={sourcePreview.url}
                  alt={`Preview of ${sourcePreview.filename}`}
                  style={{ maxWidth: "100%", maxHeight: 480 }}
                />
              ) : (
                <Text color="fg.muted" fontSize="sm">
                  This file is encrypted and stored safely, but this webview
                  cannot render its format. Use the original file in the
                  report&apos;s source list.
                </Text>
              )}
            </Box>
          ) : null}
        </Stack>
      </Box>
      <Box
        bgGradient="to-r"
        gradientFrom="teal.700"
        gradientTo="cyan.600"
        color="white"
        borderRadius="2xl"
        px={{ base: "6", md: "8" }}
        py={{ base: "6", md: "8" }}
        shadow="lg"
      >
        <Stack gap="2">
          <Text
            fontSize="sm"
            opacity="0.85"
            fontWeight="semibold"
            textTransform="uppercase"
            letterSpacing="wide"
          >
            Your private health record
          </Text>
          <Heading size="xl">Record a lab report</Heading>
          <Text opacity="0.9">
            Keep the original document and record values exactly as printed.
          </Text>
        </Stack>
      </Box>
      <Box
        as="form"
        bg="bg.panel"
        borderWidth="1px"
        borderColor="border"
        borderRadius="2xl"
        p={{ base: "5", md: "8" }}
        shadow="sm"
        onSubmit={saveReport}
      >
        <Stack gap="5">
          <SectionHeading
            number="1"
            title="Report details"
            description="When and where was the sample collected?"
          />
          <Stack gap="3" bg="bg.subtle" borderRadius="lg" p="4">
            <Text fontWeight="semibold" fontSize="sm">
              Analyte identity (optional)
            </Text>
            <Text color="fg.muted" fontSize="sm">
              Use a definition to group this result with later reports.
            </Text>
            <Field.Root>
              <Field.Label>Use a saved analyte</Field.Label>
              <select
                aria-label="Use a saved analyte"
                value={selectedAnalyteId}
                onChange={(event) => setSelectedAnalyteId(event.target.value)}
              >
                <option value="">Create a new definition</option>
                {analytes.map((analyte) => (
                  <option key={analyte.id} value={analyte.id}>
                    {analyte.name} — {analyte.component} ({analyte.property})
                  </option>
                ))}
              </select>
            </Field.Root>
            <Input
              placeholder="Analyte name, for example Hemoglobin"
              value={analyteName}
              onChange={(event) => setAnalyteName(event.target.value)}
            />
            <Input
              placeholder="Component"
              value={analyteComponent}
              onChange={(event) => setAnalyteComponent(event.target.value)}
            />
            <Input
              placeholder="Property, for example concentration"
              value={analyteProperty}
              onChange={(event) => setAnalyteProperty(event.target.value)}
            />
          </Stack>
          <Field.Root required>
            <Field.Label>Collection date and time</Field.Label>
            <Input
              type="datetime-local"
              value={collectionTime}
              onChange={(event) => setCollectionTime(event.target.value)}
            />
          </Field.Root>
          {!extraMeasurement ? (
            <Button
              type="button"
              variant="outline"
              alignSelf="start"
              onClick={() => setExtraMeasurement(true)}
            >
              + Add another result
            </Button>
          ) : (
            <Stack gap="4" borderTopWidth="1px" borderColor="border" pt="5">
              <Heading as="h4" size="sm">
                Second result
              </Heading>
              <Input
                placeholder="Source label"
                value={secondLabel}
                onChange={(event) => setSecondLabel(event.target.value)}
                required
              />
              <Input
                placeholder="Source value"
                value={secondValue}
                onChange={(event) => setSecondValue(event.target.value)}
                required
              />
              <Input
                placeholder="Unit"
                value={secondUnit}
                onChange={(event) => setSecondUnit(event.target.value)}
                required
              />
              <Input
                placeholder="Reference interval"
                value={secondInterval}
                onChange={(event) => setSecondInterval(event.target.value)}
                required
              />
              <Input
                placeholder="Flag"
                value={secondFlag}
                onChange={(event) => setSecondFlag(event.target.value)}
                required
              />
            </Stack>
          )}
          <Field.Root>
            <Field.Label>Laboratory</Field.Label>
            <Input
              value={laboratory}
              onChange={(event) => setLaboratory(event.target.value)}
            />
          </Field.Root>
          <SectionHeading
            number="2"
            title="First measurement"
            description="Enter the result as shown on the source document."
          />
          <Field.Root required>
            <Field.Label>Source label</Field.Label>
            <Input
              value={sourceLabel}
              onChange={(event) => setSourceLabel(event.target.value)}
            />
          </Field.Root>
          <Field.Root required>
            <Field.Label>Source value</Field.Label>
            <Input
              value={sourceValue}
              onChange={(event) => {
                const next = event.target.value;
                setSourceValue(next);
                setFormatConfirmed(false);
                const parsed = parseMeasurementInput(next, "de-DE");
                setValueHint(
                  parsed.kind === "ambiguous"
                    ? "This value can have more than one meaning. Confirm the intended format."
                    : parsed.kind === "number" || parsed.kind === "date"
                      ? `Normalized preview: ${parsed.normalized}`
                      : null,
                );
              }}
            />
          </Field.Root>
          {valueHint ? (
            <Text
              fontSize="sm"
              color={valueHint.startsWith("This") ? "orange.600" : "fg.muted"}
            >
              {valueHint}
            </Text>
          ) : null}
          {valueHint?.startsWith("This value") && !formatConfirmed ? (
            <Button
              type="button"
              size="sm"
              variant="outline"
              alignSelf="start"
              onClick={() => setFormatConfirmed(true)}
            >
              Confirm this format
            </Button>
          ) : null}
          <Field.Root required>
            <Field.Label>Source unit</Field.Label>
            <Input
              value={sourceUnit}
              onChange={(event) => setSourceUnit(event.target.value)}
            />
          </Field.Root>
          <Field.Root required>
            <Field.Label>Source reference interval</Field.Label>
            <Input
              value={sourceReferenceInterval}
              onChange={(event) =>
                setSourceReferenceInterval(event.target.value)
              }
            />
          </Field.Root>
          <Field.Root required>
            <Field.Label>Source flag</Field.Label>
            <Input
              value={sourceFlag}
              onChange={(event) => setSourceFlag(event.target.value)}
            />
          </Field.Root>
          {sourceFilename ? (
            <Box
              bg="teal.50"
              _dark={{ bg: "teal.950" }}
              borderRadius="lg"
              px="4"
              py="3"
            >
              <Text
                fontSize="sm"
                color="teal.800"
                _dark={{ color: "teal.100" }}
              >
                Attached source file: {sourceFilename}
              </Text>
            </Box>
          ) : null}
          <Field.Root>
            <Field.Label>Source file role</Field.Label>
            <select
              aria-label="Source file role"
              value={sourceRole}
              onChange={(event) => setSourceRole(event.target.value)}
            >
              <option value="primary">Primary report</option>
              <option value="supplement">Supplement</option>
              <option value="correction">Correction</option>
            </select>
          </Field.Root>
          <Button
            type="submit"
            alignSelf="start"
            colorPalette="teal"
            loading={saving}
          >
            Choose source file and save report
          </Button>
        </Stack>
      </Box>
      <Stack direction={{ base: "column", sm: "row" }} gap="3">
        <Button
          alignSelf="start"
          variant="outline"
          onClick={() => void backupVault()}
        >
          Save encrypted backup
        </Button>
        <Button alignSelf="start" variant="outline" onClick={() => void lock()}>
          Lock vault
        </Button>
      </Stack>
      <Box borderWidth="1px" borderColor="border" borderRadius="xl" p="4">
        <Stack gap="3">
          <Text fontWeight="semibold">Restore an encrypted backup</Text>
          <Text color="fg.muted" fontSize="sm">
            This replaces the current vault after integrity checks. Keep the
            current vault until you confirm the restored data.
          </Text>
          <Input
            type="password"
            aria-label="Backup passphrase"
            placeholder="Backup passphrase"
            value={restorePassphrase}
            onChange={(event) => setRestorePassphrase(event.target.value)}
          />
          <Button
            alignSelf="start"
            variant="outline"
            onClick={() => void restoreVault()}
          >
            Choose backup and restore
          </Button>
        </Stack>
      </Box>
      <Button
        alignSelf="start"
        variant="outline"
        onClick={() => void exportPlaintext()}
      >
        Export plaintext ZIP (review warning)
      </Button>
    </Stack>
  );
}

function SectionHeading({
  number,
  title,
  description,
}: {
  number: string;
  title: string;
  description: string;
}) {
  return (
    <Stack direction="row" gap="3" align="start" pt="2">
      <Box
        borderRadius="full"
        bg="teal.600"
        color="white"
        minW="8"
        h="8"
        display="grid"
        placeItems="center"
        fontWeight="bold"
        fontSize="sm"
      >
        {number}
      </Box>
      <Stack gap="0">
        <Heading as="h3" size="md">
          {title}
        </Heading>
        <Text color="fg.muted" fontSize="sm">
          {description}
        </Text>
      </Stack>
    </Stack>
  );
}

export default App;
