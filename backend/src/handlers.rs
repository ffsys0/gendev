use std::collections::HashMap;
// TODO why old_required games??!
use crate::{logic::find_minimal_packages, models::*};
use actix_web::{get, web, HttpResponse, Responder, Result};
use bit_set::BitSet;
use itertools::Itertools;

#[get("/teams")]
async fn get_teams(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=600"))
        .json(&state.teams)
}

#[get("/games")]
async fn get_games(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=600"))
        .json(&state.games)
}

#[get("/tournaments")]
async fn get_tournaments(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=600"))
        .json(&state.tournaments)
}

#[get("/")]
async fn get_streaming_packages(
    query: web::Query<GetQuery>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    if !query.live && !query.highlights{
        return Err(actix_web::error::ErrorBadRequest("At least one of 'live' or 'highlights' must be true"));
    }

    let mut requested_games: Vec<usize> = serde_json::from_str(&query.games)
        .map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("Invalid JSON in 'games': {}", e))
        })?;
    let requested_teams: Vec<String> = serde_json::from_str(&query.teams).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid JSON in 'teams': {}", e))
    })?;
    let requested_tournaments: Vec<String> = serde_json::from_str(&query.tournaments)
        .map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("Invalid JSON in 'tournaments': {}", e))
        })?;

    let mut remaining_needed_games: BitSet = BitSet::new();
    
    if query.all_games {
        requested_games = state.games.iter().map(|game| game.id).collect();
    }

    requested_games.iter().for_each(|v| {
        remaining_needed_games.insert(*v);
    });
    requested_teams.iter().try_for_each(|team| {
        state
            .teams_to_games
            .get(&team)
            .map(|games| remaining_needed_games.union_with(games))
            .ok_or_else(|| {
                actix_web::error::ErrorNotFound(format!("Team '{}' not found in state", team))
            })
    })?;
    requested_tournaments.iter().try_for_each(|tournament| {
        state
            .tournament_to_games
            .get(&tournament)
            .map(|games| remaining_needed_games.union_with(games))
            .ok_or_else(|| {
                actix_web::error::ErrorNotFound(format!(
                    "Tournament '{}' not found in state",
                    tournament
                ))
            })
    })?;

    if query.highlights {
        let highlight_games = remaining_needed_games.iter().map(|live_game| live_game+8876).collect::<BitSet>();
        if query.live {
            remaining_needed_games.union_with(&highlight_games);
        } else {
            remaining_needed_games = highlight_games;
        }   
    }// else only live games, one must be selected ensured by if condition above

    let covered_games = if query.only_monthly_billing {
            &state.all_games_covered_by_monthly_packages
    } else {
            &state.all_games_covered
    };
    // discard impossible games
    remaining_needed_games.intersect_with(covered_games);

    let mut consider_packages: Vec<usize> = Vec::new();
    let mut result_packages: Vec<usize> = Vec::new();

    let uniquely_covered = state
        .uniquely_covered_games_set
        .intersection(&remaining_needed_games);
    // if a game is only covered by a single package select that
    for game in uniquely_covered {
        let package = state.uniquely_covered_games_map.get(&game).ok_or_else(|| {
            actix_web::error::ErrorInternalServerError(format!("Could not find game {}", game))
        })?;

        if query.only_monthly_billing
            && state
                .packages
                .iter()
                .find(|p| p.id == *package) // packages are only of about 40 elements so this is fine
                .unwrap()
                .monthly_price_cents
                .is_some()
            && !result_packages.contains(package) // result packages is small so this is fine
        {
                result_packages.push(*package);
        }
        
    }
    let all_needed_games = remaining_needed_games.clone();


    // remove games which are covered by the uniquely covered packages
    result_packages.iter().for_each(|package_id| {
        remaining_needed_games.difference_with(&state.packages_to_covered_games[*package_id]);
    });
    // generate an iterator over the packages either all or just packages which have monthly billing
    let packages_iter: Box<dyn Iterator<Item = &Package>> = match query.only_monthly_billing {
        false => Box::new(state.packages.iter()),
        true => Box::new(
            state
                .packages
                .iter()
                .filter(|package| package.monthly_price_cents.is_some()),
        ),
    };
    // consider all packages which cover at least one required game
    for package in packages_iter {
        let covered_games = &state.packages_to_covered_games[package.id];
        if remaining_needed_games.intersection(covered_games).next().is_some() {
            consider_packages.push(package.id);
        }
    }

    let result = find_minimal_packages(&remaining_needed_games, &state, consider_packages, query.only_monthly_billing);
    log::debug!("done with min calcluation");
    return match result {
        Some(packages) => {
                let packages_refs: Vec<&Package> = state.packages.iter().collect();
                result_packages.extend(packages);
            
                //--------------------------------------------------------------------
                // Build the Rows
                //--------------------------------------------------------------------
                // Helper to build sub-rows for individual games
                let build_game_sub_rows = |game_ids: &BitSet, packages: &Vec<&Package>| {
                    let mut rows = Vec::new();
                    for game_id in game_ids.iter().map(|id| if id <= 8876 {id} else {id-8876}).unique() {
                        if let Ok(pos) = state.games.binary_search_by(|g| g.id.cmp(&game_id)) {
                            let game_obj = &state.games[pos];
                            let mut game_row = Row {
                                key: game_obj.to_string(),
                                provider_coverage: HashMap::new(),
                                provider_coverage_highlights: HashMap::new(),
                                sub_rows: None,
                            };
            
                            // Check coverage for each package
                            for package in packages {
                                let covered = state.packages_to_covered_games[package.id].contains(game_id);
                                let coverage = if covered { Coverage::FULL } else { Coverage::NONE };
                                game_row
                                    .provider_coverage
                                    .insert(package.name.clone(), coverage);

                                let coverage_highlights = if state.packages_to_covered_games[package.id].contains(game_id + 8876) {
                                    Coverage::FULL
                                } else {
                                    Coverage::NONE
                                };
                                game_row
                                    .provider_coverage_highlights
                                    .insert(package.name.clone(), coverage_highlights);
                            }
                            rows.push(game_row);
                        }
                    }
                    rows
                };
            
                // A helper that determines FULL / PARTIAL / NONE coverage for a collection of games
                let coverage_for_game_ids = |game_ids: &BitSet, coverage_set: &BitSet| {
                    let total_needed = game_ids.len();
                    let intersection_count = game_ids.intersection(coverage_set).count();
            
                    if intersection_count == 0 {
                        Coverage::NONE
                    } else if intersection_count == total_needed {
                        Coverage::FULL
                    } else {
                        Coverage::PARTIAL
                    }
                };
            
                let mut all_rows: Vec<Row> = Vec::new();
            
                //--------------------------------------------------------------------
                // 1) Build rows for requested TEAMS
                //--------------------------------------------------------------------
                for team_name in requested_teams.iter() {
                    let team_game_ids = match state.teams_to_games.get(team_name) {
                        Some(bitset) => bitset,
                        None => {
                            continue;
                        }
                    };
                    let mut team_game_ids = team_game_ids.iter().flat_map(|id| [id,id+8876]).collect::<BitSet>();
                    team_game_ids.intersect_with(&all_needed_games);
            
                    let sub_rows = build_game_sub_rows(&team_game_ids, &packages_refs);
            
                    let mut team_row = Row {
                        key: team_name.to_string(),
                        provider_coverage: HashMap::new(),
                        provider_coverage_highlights: HashMap::new(),
                        sub_rows: Some(sub_rows),
                    };
            
                    for package in state.packages {
                        let coverage_set = &state.packages_to_covered_games[package.id].intersection(&all_needed_games).collect();
                        let coverage = coverage_for_game_ids(&team_game_ids.iter().filter(|id| *id<=8876).collect(), coverage_set);
                        team_row
                            .provider_coverage
                            .insert(package.name.clone(), coverage);

                        let coverage_highlights = coverage_for_game_ids(&team_game_ids.iter().filter(|id| *id>8876).collect(), coverage_set);
                        team_row
                            .provider_coverage_highlights
                            .insert(package.name.clone(), coverage_highlights);
                    }
            
                    all_rows.push(team_row);
                }
            
                //--------------------------------------------------------------------
                // 2) Build rows for all TOURNAMENTS
                //--------------------------------------------------------------------
                for tournament_name in requested_tournaments.iter() {
                    let tournament_game_ids = match state.tournament_to_games.get(tournament_name) {
                        Some(bitset) => bitset,
                        None => {
                            continue;
                        }
                    };
                    let mut tournament_game_ids = tournament_game_ids.iter().flat_map(|id| [id,id+8876]).collect::<BitSet>();
                    tournament_game_ids.intersect_with(&all_needed_games);

            
                    let sub_rows = build_game_sub_rows(&tournament_game_ids, &packages_refs);
            
                    let mut tournament_row = Row {
                        key: tournament_name.to_string(),
                        provider_coverage: HashMap::new(),
                        provider_coverage_highlights: HashMap::new(),
                        sub_rows: Some(sub_rows),
                    };
            
                    // Evaluate coverage across all games in the tournament
                    for package in state.packages {
                        let coverage_set = &state.packages_to_covered_games[package.id].intersection(&all_needed_games).collect();
                        let coverage = coverage_for_game_ids(&tournament_game_ids.iter().filter(|id|*id<=8876 ).collect(), coverage_set);
                        tournament_row
                            .provider_coverage
                            .insert(package.name.clone(), coverage);

                        let coverage_highlights = coverage_for_game_ids(&tournament_game_ids.iter().filter(|id| *id>8876).collect(), coverage_set);
                        tournament_row
                            .provider_coverage_highlights
                            .insert(package.name.clone(), coverage_highlights);
                    }
            
                    all_rows.push(tournament_row);
                }
            
                //--------------------------------------------------------------------
                // 3) Build rows for each GAME
                //--------------------------------------------------------------------
                for game in requested_games.iter() {
                    let mut game_row = Row {
                        key: game.to_string(),
                        provider_coverage: HashMap::new(),
                        provider_coverage_highlights: HashMap::new(),
                        sub_rows: None,
                    };
                    for package in state.packages {
                        let coverage_set = &state.packages_to_covered_games[package.id];
                        let coverage = if coverage_set.contains(*game) {
                            Coverage::FULL
                        } else {
                            Coverage::NONE
                        };
                        game_row
                            .provider_coverage
                            .insert(package.name.clone(), coverage);

                        let coverage_highlights = if state.packages_to_covered_games[package.id].contains(game + 8876) {
                            Coverage::FULL
                        } else {
                            Coverage::NONE
                        };
                        game_row
                            .provider_coverage_highlights
                            .insert(package.name.clone(), coverage_highlights);
                    }
                    all_rows.push(game_row);
                }
            
                let sorted_packages_by_covered_games_per_euro = state.packages
                    .iter()
                    .map(|p| (p, (state.packages_to_covered_games[p.id].intersection(&all_needed_games).count() as f32) / (p.monthly_price_yearly_subscription_in_cents.unwrap() as f32)))
                    .map(|(p, weigth)| (p, if weigth.is_nan() { 0.0 as f32 } else { weigth })) // if the package was free and div by zero resulted in NaN
                    .sorted_by(|(p1, covered_game_per_euro1), (p2, covered_game_per_euro2)| {
                        // Check if either package is in result_packages
                        let p1_in_result = result_packages.contains(&p1.id);
                        let p2_in_result = result_packages.contains(&p2.id);
                        
                        match (p1_in_result, p2_in_result) {
                            // If both or neither are in result_packages, sort by covered games
                            (true, true) | (false, false) => covered_game_per_euro2.partial_cmp(covered_game_per_euro1).unwrap_or(std::cmp::Ordering::Equal),
                            // If only p1 is in result_packages, it comes first
                            (true, false) => std::cmp::Ordering::Less,
                            // If only p2 is in result_packages, it comes first
                            (false, true) => std::cmp::Ordering::Greater,
                        }
                    })
                    .map(|(p, _)| p)
                    .collect::<Vec<&Package>>();

                return Ok(HttpResponse::Ok().json(GetResponse {
                    packages: sorted_packages_by_covered_games_per_euro,
                    rows: all_rows,
                    result: result_packages
                    .iter()
                    .map(|id| state.packages.iter().find(|p| p.id == *id).unwrap())
                    .collect(),
                }));
            }
        None => Err(actix_web::error::ErrorInternalServerError(
            "Could not find a solution",
        )),
    };
}

