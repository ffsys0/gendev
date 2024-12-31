import * as React from 'react';
import Autocomplete, { autocompleteClasses, createFilterOptions } from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import { useTheme, styled } from '@mui/material/styles';
import useMediaQuery from '@mui/material/useMediaQuery';
import { VariableSizeList, ListChildComponentProps } from 'react-window';
import Popper from '@mui/material/Popper';
import ListSubheader from '@mui/material/ListSubheader';

export interface Game {
    id: number,
    team_home: string,
    team_away: string,
    starts_at: string,
    tournament_name: string,
}
const LISTBOX_PADDING = 8; // px

function renderRow(props: ListChildComponentProps) {
    const { data, index, style } = props;
    const dataSet = data[index];
    const inlineStyle = {
        ...style,
        top: (style.top as number) + LISTBOX_PADDING,
    };

    if (dataSet.hasOwnProperty('group')) {
        return (
            <ListSubheader key={dataSet.key} component="div" style={inlineStyle}>
                {dataSet.group}
            </ListSubheader>
        );
    }

    const { key, ...optionProps } = dataSet[0];

    return (
        <Typography key={key} component="li" {...optionProps} noWrap style={inlineStyle}>
            {`${dataSet[1].team_home} - ${dataSet[1].team_away}`} <br /> {dataSet[1].tournament_name}
        </Typography>
    );
}

const OuterElementContext = React.createContext({});

const OuterElementType = React.forwardRef<HTMLDivElement>((props, ref) => {
    const outerProps = React.useContext(OuterElementContext);
    return <div ref={ref} {...props} {...outerProps} />;
});

function useResetCache(data: any) {
    const ref = React.useRef<VariableSizeList>(null);
    React.useEffect(() => {
        if (ref.current != null) {
            ref.current.resetAfterIndex(0, true);
        }
    }, [data]);
    return ref;
}

// Adapter for react-window
const ListboxComponent = React.forwardRef<
    HTMLDivElement,
    React.HTMLAttributes<HTMLElement>
>(function ListboxComponent(props, ref) {
    const { children, ...other } = props;
    const itemData: React.ReactElement<unknown>[] = [];
    (children as React.ReactElement<unknown>[]).forEach(
        (
            item: React.ReactElement<unknown> & {
                children?: React.ReactElement<unknown>[];
            },
        ) => {
            itemData.push(item);
            itemData.push(...(item.children || []));
        },
    );

    const theme = useTheme();
    const smUp = useMediaQuery(theme.breakpoints.up('sm'), {
        noSsr: true,
    });
    const itemCount = itemData.length;
    const itemSize = 48;

    const getChildSize = (child: React.ReactElement<unknown>) => {
        if (child.hasOwnProperty('group')) {
            return 48;
        }

        return itemSize;
    };

    const getHeight = () => {
        if (itemCount > 8) {
            return 8 * itemSize;
        }
        return itemData.map(getChildSize).reduce((a, b) => a + b, 0);
    };

    const gridRef = useResetCache(itemCount);

    return (
        <div ref={ref}>
            <OuterElementContext.Provider value={other}>
                <VariableSizeList
                    itemData={itemData}
                    height={getHeight() + 2 * LISTBOX_PADDING}
                    width="100%"
                    ref={gridRef}
                    outerElementType={OuterElementType}
                    innerElementType="ul"
                    itemSize={(index) => getChildSize(itemData[index])}
                    overscanCount={5}
                    itemCount={itemCount}
                >
                    {renderRow}
                </VariableSizeList>
            </OuterElementContext.Provider>
        </div>
    );
});

const StyledPopper = styled(Popper)({
    [`& .${autocompleteClasses.listbox}`]: {
        boxSizing: 'border-box',
        '& ul': {
            padding: 0,
            margin: 0,
        },
    },
});

export default function GameAutoComplete(props: { setSelectedGames: React.Dispatch<React.SetStateAction<Game[]>>, disabled: boolean }) {
    const [games, setGames] = React.useState<Game[]>([]);

    React.useEffect(() => {
        const fetchGames = async () => {
            try {
                const response = await fetch('http://localhost:8080/games');
                const data = await response.json();
                setGames(data);
            } catch (error) {
                console.error('Error fetching games:', error);
            }
        };

        fetchGames();
    }, []);


    const filterOptions = createFilterOptions({
        stringify: (option: Game) => `${option.team_home}${option.team_away}${option.tournament_name}`,
    });

    const handleSelectionChange = (event: React.ChangeEvent<{}>, value: Game[]) => {
        props.setSelectedGames(value);
    };
    return (
        <Autocomplete
            multiple
            filterOptions={filterOptions}
            disableListWrap
            disabled={props.disabled}
            options={games}
            renderInput={(params) => <TextField {...params} label="Select Games" />}
            renderOption={(props, option, state) =>
                [props, option, state.index] as React.ReactNode
            }

            getOptionLabel={(option) => `${option.team_home} - ${option.team_away}`}
            renderGroup={(params) => params as any}
            slots={{
                popper: StyledPopper,
            }}
            slotProps={{
                listbox: {
                    component: ListboxComponent,
                },
            }}
            onChange={handleSelectionChange}
        />

    );
}

