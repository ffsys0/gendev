import { useMemo } from 'react';
import {
  MaterialReactTable,
  useMaterialReactTable,
  type MRT_ColumnDef,
} from 'material-react-table';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import RemoveCircleIcon from '@mui/icons-material/RemoveCircle';
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked';

const getCoverageIcon = (coverage: string) => {
  switch (coverage) {
    case 'FULL':
      return <CheckCircleIcon sx={{ color: 'green' }} />;
    case 'PARTIAL':
      return <RemoveCircleIcon sx={{ color: 'orange' }} />;
    case 'NONE':
    default:
      return <RadioButtonUncheckedIcon sx={{ color: 'gray' }} />;
  }
};

interface Package {
  id: number;
  name: string;
  monthly_price_cents: number | null;
  monthly_price_yearly_subscription_in_cents: number;
}

interface RowEntry {
  key: string;
  subrows?: RowEntry[];
  [provider: string]: any;
}

interface ComparisonGridProps {
  data: {
    rows: RowEntry[];
    packages: Package[];
    result?: Package[];
  };
}

const ComparisonGrid = (props: ComparisonGridProps) => {
  const { rows, packages, result } = props.data;

  const columns = useMemo<MRT_ColumnDef<RowEntry>[]>(
    () => [
      {
        accessorKey: 'key',
        header: 'Competition',
      },
      ...packages.map((provider, index) => {
        const isHighlighted = result?.some((r) => r.name === provider.name);
        const backgroundColor = index % 2 === 0 ? '#242424' : '#2E2E2E';

        return {
          header: provider.name,
          Header: () => (
            <div>
              <span style={{ color: isHighlighted ? 'red' : 'inherit' }}>
                {provider.name}
              </span>
              <div style={{ fontSize: 'smaller' }}>
                $
                {(
                  provider.monthly_price_yearly_subscription_in_cents / 100
                ).toFixed(2)}
                /mo yearly
                {provider.monthly_price_cents && (
                  <div>
                    $
                    {(provider.monthly_price_cents / 100).toFixed(2)}
                    /mo monthly
                  </div>
                )}
              </div>
            </div>
          ),
          muiTableHeadCellProps: {
            sx: { backgroundColor },
          },
          columns: [
            {
              id: `${provider.name}-live-coverage`,
              accessorKey: `provider_coverage.${provider.name}`,
              header: 'Live Coverage',
              Cell: ({ cell }: any) => {
                const coverage = cell.getValue();
                return getCoverageIcon(coverage);
              },
              muiTableHeadCellProps: {
                sx: { backgroundColor },
              },
              muiTableBodyCellProps: {
                sx: { backgroundColor },
              },
            },
            {
              id: `${provider.name}-highlights`,
              accessorKey: `provider_coverage_highlights.${provider.name}`,
              header: 'Highlights',
              Cell: ({ cell }: any) => {
                const coverage = cell.getValue();
                return getCoverageIcon(coverage);
              },
              muiTableHeadCellProps: {
                sx: { backgroundColor },
              },
              muiTableBodyCellProps: {
                sx: { backgroundColor },
              },
            },
          ],
        };
      }),
    ],
    [packages, result],
  );

  const table = useMaterialReactTable({
    columns,
    data: rows,
    enableExpanding: true,
    getSubRows: (row) => row.sub_rows ?? undefined,
  });

  return <MaterialReactTable table={table} />;
};

export default ComparisonGrid;
