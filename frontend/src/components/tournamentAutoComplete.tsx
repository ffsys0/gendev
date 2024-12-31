import * as React from 'react';
import Autocomplete, { autocompleteClasses, createFilterOptions } from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import { useTheme, styled } from '@mui/material/styles';
import useMediaQuery from '@mui/material/useMediaQuery';
import { VariableSizeList, ListChildComponentProps } from 'react-window';
import Popper from '@mui/material/Popper';

const LISTBOX_PADDING = 8; // px

function renderRow(props: ListChildComponentProps) {
    const { data, index, style } = props;
    const dataSet = data[index];
    const inlineStyle = {
        ...style,
        top: (style.top as number) + LISTBOX_PADDING,
    };

    const { key, ...optionProps } = dataSet[0];

    return (
        <Typography key={key} component="li" {...optionProps} noWrap style={inlineStyle}>
            {`${dataSet[1]}`} <br /> {dataSet[1].tournament_name}
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
            item: React.ReactElement<unknown>
        ) => {
            itemData.push(item);
        },
    );

    const itemCount = itemData.length;
    const itemSize = 48;


    const getHeight = () => {
        if (itemCount > 8) {
            return 8 * itemSize;
        }
        return itemData.map(() => itemSize).reduce((a, b) => a + b, 0);
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
                    itemSize={(index) => itemSize}
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

export default function TournamentAutoComplete(props: { setSelectedTournaments: React.Dispatch<React.SetStateAction<string[]>>, disabled: boolean }) {
    const [tournament, setTournaments] = React.useState<string[]>([]);

    React.useEffect(() => {
        const fetchGames = async () => {
            try {
                const response = await fetch('http://localhost:8080/tournaments');
                const data = await response.json();
                setTournaments(data);
            } catch (error) {
                console.error('Error fetching games:', error);
            }
        };

        fetchGames();
    }, []);


    const handleSelectionChange = (event: React.ChangeEvent<{}>, value: string[]) => {
        props.setSelectedTournaments(value);
    };


    return (
        <Autocomplete
            multiple
            disableListWrap
            options={tournament}
            disabled={props.disabled}
            renderInput={(params) => <TextField {...params} label="Select Tournamets" />}
            renderOption={(props, option, state) =>
                [props, option, state.index] as React.ReactNode
            }

            getOptionLabel={(option) => option}
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

