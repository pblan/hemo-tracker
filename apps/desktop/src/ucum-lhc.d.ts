declare module "@lhncbc/ucum-lhc" {
  export class UcumLhcUtils {
    static getInstance(): UcumLhcUtils;
    validateUnitString(
      unit: string,
      suggest?: boolean,
    ): {
      status: "valid" | "invalid" | "error";
      ucumCode: string | null;
      msg: string[];
    };
    convertUnitTo(
      fromUnit: string,
      value: number,
      toUnit: string,
      options?: Record<string, never>,
    ): {
      status: "succeeded" | "failed" | "error";
      toVal: number | null;
      msg: string[] | null;
    };
  }
}
