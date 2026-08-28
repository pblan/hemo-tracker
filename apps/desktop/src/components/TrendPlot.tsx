import { Box, Table, Text } from "@chakra-ui/react";

export type TrendPoint = {
  date: string;
  value: number | null;
  unit: string;
  flag?: string;
};

export function TrendPlot({
  title,
  points,
}: {
  title: string;
  points: TrendPoint[];
}) {
  const numeric = points.filter((point) => point.value !== null);
  const values = numeric.map((point) => point.value as number);
  const min = values.length ? Math.min(...values) : 0;
  const max = values.length ? Math.max(...values) : 1;
  const range = max - min || 1;
  const width = 640;
  const height = 180;
  const path = numeric
    .map((point, index) => {
      const x = (index / Math.max(numeric.length - 1, 1)) * (width - 24) + 12;
      const y =
        height - 12 - (((point.value as number) - min) / range) * (height - 24);
      return `${index ? "L" : "M"}${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(" ");
  return (
    <Box
      borderWidth="1px"
      borderColor="border"
      borderRadius="xl"
      p="5"
      bg="bg.panel"
    >
      <Text fontWeight="semibold" mb="3">
        {title}
      </Text>
      <svg
        role="img"
        aria-label={`${title} trend plot`}
        viewBox={`0 0 ${width} ${height}`}
        width="100%"
        height="180px"
        preserveAspectRatio="none"
      >
        <path
          d={path}
          fill="none"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <Table.Root size="sm" variant="outline" mt="4">
        <Table.Caption>Accessible data table for {title}</Table.Caption>
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeader>Date</Table.ColumnHeader>
            <Table.ColumnHeader>Value</Table.ColumnHeader>
            <Table.ColumnHeader>Flag</Table.ColumnHeader>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {points.map((point) => (
            <Table.Row key={`${point.date}-${point.value}`}>
              <Table.Cell>{point.date}</Table.Cell>
              <Table.Cell>
                {point.value === null
                  ? "Missing"
                  : `${point.value} ${point.unit}`}
              </Table.Cell>
              <Table.Cell>{point.flag || "—"}</Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}
