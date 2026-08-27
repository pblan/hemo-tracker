export type PlotPoint = {
  date: number;
  value: number;
  low: number;
  high: number;
  flagged: boolean;
  comparable: boolean;
};

export type AnalyteSeries = {
  id: string;
  name: string;
  unit: string;
  targetLow: number;
  targetHigh: number;
  points: PlotPoint[];
};

export type StressFixture = {
  reportCount: number;
  measurementCount: number;
  analyteCount: number;
  allMeasurements: Float64Array;
  visible: AnalyteSeries[];
};

export function createStressFixture(): StressFixture {
  const reportCount = 1_000;
  const analyteCount = 250;
  const measurementsPerReport = 100;
  const visibleCount = 20;
  const allMeasurements = Float64Array.from(
    { length: reportCount * measurementsPerReport },
    (_, index) => 10 + (index % analyteCount) * 0.1 + Math.sin(index / 31),
  );
  const visible = Array.from({ length: visibleCount }, (_, analyteIndex) => {
    const base = 15 + analyteIndex * 1.7;
    const points = Array.from({ length: reportCount }, (_, reportIndex) => {
      const date =
        Date.UTC(2010, 0, 1) / 1_000 +
        reportIndex * 86400 * (2 + (reportIndex % 5));
      const intervalShift = reportIndex >= 500 ? base * 0.08 : 0;
      const low = base * 0.75 + intervalShift;
      const high = base * 1.25 + intervalShift;
      const value =
        base + Math.sin(reportIndex / 17 + analyteIndex) * base * 0.22;
      return {
        date,
        value,
        low,
        high,
        flagged: reportIndex % 113 === analyteIndex % 17,
        comparable: reportIndex !== 500,
      };
    });

    return {
      id: `analyte-${analyteIndex + 1}`,
      name: `Analyte ${analyteIndex + 1}`,
      unit: analyteIndex % 2 === 0 ? "mg/dL" : "mmol/L",
      targetLow: base * 0.85,
      targetHigh: base * 1.15,
      points,
    };
  });

  return {
    reportCount,
    measurementCount: reportCount * measurementsPerReport,
    analyteCount,
    allMeasurements,
    visible,
  };
}
