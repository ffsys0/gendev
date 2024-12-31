'use client'
import React, { useState } from 'react'
import GameAutoComplete, { Game } from './gameAutoComplete'
import CssBaseline from '@mui/material/CssBaseline';
import Container from '@mui/material/Container';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Paper from '@mui/material/Paper';
import FormControlLabel from '@mui/material/FormControlLabel';
import Checkbox from '@mui/material/Checkbox';
import TournamentAutoComplete from './tournamentAutoComplete';
import TeamsAutoComplete from './teamsAutoComplete';
import ComparisonTable from './ComparisonGrid';
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';
import { Typography } from '@mui/material';
import Alert from '@mui/material/Alert';

interface Package {
  id: number;
  name: string;
  monthly_price_cents: number;
  monthly_price_yearly_subscription_in_cents: number;
}

interface GameResponse {
  id: number;
  team_home: string;
  team_away: string;
  starts_at: string;
  tournament_name: string;
}

interface Response {
  packages: Package[];
  games_not_covered: GameResponse[];
  rows: any[];
}

function home() {
  const [selectedGames, setSelectedGames] = useState<Game[]>([])
  const [selectedTeams, setSelectedTeams] = useState<string[]>([])
  const [selectedTournaments, setSelectedTournaments] = useState<string[]>([])
  const [onlyMonthlyBilling, setOnlyMonthlyBilling] = useState(false)
  const [response, setResponse] = useState<Response | null>(null);
  const [allGames, setAllGames] = useState(false)
  const [formats, setFormats] = React.useState(() => ['live']);
  const [error, setError] = useState<string | null>(null)

  const handleFormat = (
    event: React.MouseEvent<HTMLElement>,
    newFormats: string[],
  ) => {
    setFormats(newFormats);
  };

  const sum = React.useMemo(() => {
    if (!response) {
      return 0;
    }
    return response.packages.reduce((acc, pkg) => acc + (onlyMonthlyBilling ? pkg.monthly_price_cents : pkg.monthly_price_yearly_subscription_in_cents), 0)
  }, [response]);

  const handleSubmit = async () => {
    try {
      const ids = selectedGames.map((game) => game.id)
      const queryString = new URLSearchParams({
        games: JSON.stringify(ids),
        teams: JSON.stringify(selectedTeams),
        tournaments: JSON.stringify(selectedTournaments),
        all_games: allGames ? "1" : "0",
        only_monthly_billing: onlyMonthlyBilling ? "1" : "0",
        live: formats.includes('live') ? "1" : "0",
        highlights: formats.includes('highlights') ? "1" : "0",
      }).toString();
      const response = await fetch(`http://localhost:8080/?${queryString}`, {
        method: 'GET',
      });

      if (!response.ok) {
        setError('We could not find a solution for the selected games, because no packages are covering any game');
        throw new Error('Network response was not ok');
      }

      const data = await response.json();
      setResponse(data); // Update packages state with the response JSON
      console.log('Fetched data:', data);
    } catch (error) {
      console.error('Error during fetch:', error);
    }
  };

  const handleAllGamesCheckbox = (event: React.ChangeEvent<HTMLInputElement>) => {
    setAllGames(event.target.checked);
  }

  const handleBillingIntervalChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setOnlyMonthlyBilling(event.target.checked);
  }
  return (
    <React.Fragment>
      <CssBaseline />

      {response ?
        (<>
          <ComparisonTable data={response} />
          <Button onClick={() => {
            setResponse(null)
            setSelectedGames([])
            setSelectedTeams([])
            setSelectedTournaments([])
            setAllGames(false)
            setFormats(['live'])
          }} variant="contained" fullWidth>New Comparsion</Button></>)
        :
        <Container sx={{ height: '100vh', padding: 2, minHeight: "100%" }} maxWidth="lg">
          <Paper elevation={3} sx={{ padding: 2 }}>
            <Stack spacing={2}>
              {!response &&
                (<>
                  <GameAutoComplete disabled={allGames} setSelectedGames={setSelectedGames} />
                  <TournamentAutoComplete disabled={allGames} setSelectedTournaments={setSelectedTournaments} />
                  <TeamsAutoComplete disabled={allGames} setSelectedTeams={setSelectedTeams} />
                  <FormControlLabel control={<Checkbox onChange={handleBillingIntervalChange} />} label="Only Monthly Billing" />
                  <FormControlLabel control={<Checkbox onChange={handleAllGamesCheckbox} />} label="Select all Games" />
                  <ToggleButtonGroup
                    value={formats}
                    onChange={handleFormat}
                    aria-label="text formatting"
                    fullWidth
                  >
                    <ToggleButton value="live" aria-label="live" fullWidth>
                      <Typography>LIVE</Typography>
                    </ToggleButton>
                    <ToggleButton value="highlights" aria-label="highlights" fullWidth>
                      <Typography>HIGHLIGHTS</Typography>
                    </ToggleButton>
                  </ToggleButtonGroup>
                  <Button onClick={handleSubmit} variant="contained" fullWidth>Submit</Button>
                </>)
              }
              {error && <Alert severity="error">{error}</Alert>}
            </Stack>
          </Paper>
        </Container>
      }

    </React.Fragment>
  )
}

export default home