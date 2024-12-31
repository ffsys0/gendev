mod handlers;
mod logic;
mod models;
mod e2e_testing;

use crate::handlers::{get_games, get_streaming_packages, get_teams};
use crate::models::{AppState, Game, Offer, Package};
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, Result};
use bit_set::BitSet;
use handlers::get_tournaments;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;


fn read_csv<T: for<'de> Deserialize<'de>>(file_path: &str) -> Result<Vec<T>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: T = result?;
        records.push(record);
    }
    Ok(records)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // leaking data is ok because should live for the whole lifetime of program...
    let mut games = Box::new(
        read_csv::<Game>("data/bc_game.csv")
            .expect("Failed to read games.csv")
            .into_iter()
            .flat_map(|game| {
                [
                    game.clone(),
                    // highlight games are just the same game with id + 8876
                    // this allows us to easily use Bitsets
                    Game {
                        id: game.id + 8876,
                        team_home: game.team_home,
                        team_away: game.team_away,
                        starts_at: game.starts_at,
                        tournament_name: game.tournament_name,
                    },
                ]
            })
            .collect::<Vec<Game>>(),
    );
    let mut packages = Box::new(
        read_csv::<Package>("data/bc_streaming_package.csv").expect("Failed to read packages.csv"),
    );
    let mut offers = Box::new(
        read_csv::<Offer>("data/bc_streaming_offer.csv").expect("Failed to read offers.csv"),
    );
    games.sort_unstable_by_key(|v| v.id);
    packages.sort_unstable_by_key(|v| v.id);
    offers.sort_unstable_by_key(|v| v.game_id);

    let games: &Vec<Game> = Box::leak(games);
    let packages: &Vec<Package> = Box::leak(packages);
    let offers: &Vec<Offer> = Box::leak(offers);

    let teams = games
        .iter()
        .flat_map(|game| [&game.team_away, &game.team_home])
        .collect::<HashSet<&String>>()
        .into_iter()
        .collect::<Vec<&String>>();
    let teams: &Vec<&String> = Box::leak(Box::new(teams));

    let teams_to_games: &mut HashMap<&String, BitSet> = Box::leak(Box::new(HashMap::new()));
    for game in games.iter() {
        // HOME TEAM
        teams_to_games
            .entry(&game.team_home)
            .and_modify(|s: &mut BitSet| {
                s.insert(game.id);
            })
            .or_insert({
                let mut new_set = BitSet::new();
                new_set.insert(game.id);
                new_set
            });
        // AWAY TEAM
        teams_to_games
            .entry(&game.team_away)
            .and_modify(|s| {
                s.insert(game.id);
            })
            .or_insert({
                let mut new_set = BitSet::new();
                new_set.insert(game.id);
                new_set
            });
    }

    let tournaments = games
        .iter()
        .fold(HashSet::new(), |mut set, game| {
            set.insert(&game.tournament_name);
            set
        })
        .into_iter()
        .collect();
    let tournamets: &Vec<&String> = Box::leak(Box::new(tournaments));

    let tournament_to_games: HashMap<&String, BitSet> =
        games.iter().fold(HashMap::new(), |mut map, game| {
            map.entry(&game.tournament_name)
                .and_modify(|s| {
                    s.insert(game.id);
                })
                .or_insert({
                    let mut new_set = BitSet::new();
                    new_set.insert(game.id);
                    new_set
                });
            map
        });
    let tournament_to_games: &HashMap<&String, BitSet> = Box::leak(Box::new(tournament_to_games));

    let mut all_covered_games = BitSet::new();
    let packages_to_covered_games: &mut Vec<BitSet> = Box::leak(Box::new(vec![
        BitSet::new();
        packages.iter().map(|p| p.id).max().unwrap()
            as usize
            + 1
    ]));

    let mut all_games_covered_by_monthly_packages = BitSet::new();
    for offer in offers {
        let package_id = offer.streaming_package_id;
        let is_monthly = packages
            .iter()
            .find(|p| p.id == package_id)
            .unwrap()
            .monthly_price_cents
            .is_some();

        let game_id = offer.game_id as usize;
        if offer.live {
            packages_to_covered_games[package_id].insert(game_id);
            all_covered_games.insert(game_id);
            if is_monthly {
                all_games_covered_by_monthly_packages.insert(game_id);
            }
        }
        if offer.highlights {
            packages_to_covered_games[package_id].insert(game_id + 8876);
            all_covered_games.insert(game_id + 8876);
            if is_monthly {
                all_games_covered_by_monthly_packages.insert(game_id + 8876);
            }
        }
    }

    let mut game_to_packages: HashMap<usize, BitSet> = HashMap::new();
    for (index, package) in packages_to_covered_games.iter().enumerate() {
        package.iter().for_each(|game_id| {
            game_to_packages
                .entry(game_id)
                .and_modify(|set| {
                    set.insert(index);
                })
                .or_insert({
                    let mut new_set = BitSet::new();
                    new_set.insert(index);
                    new_set
                });
        });
    }

    let uniquely_covered_games_map: HashMap<usize, usize> = game_to_packages
        .into_iter()
        .filter(|(_, packages)| packages.len() == 1)
        .map(|(game, packages)| (game, packages.into_iter().next().unwrap()))
        .collect();
    let uniquely_covered_games_map: &HashMap<usize, usize> =
        Box::leak(Box::new(uniquely_covered_games_map));

    let mut uniquely_covered_games_set = BitSet::new();
    uniquely_covered_games_map.keys().for_each(|k| {
        uniquely_covered_games_set.insert(*k);
    });

    // rerefences have to be immutable in closure, so we avoid copy
    let teams_to_games: &HashMap<&String, BitSet> = teams_to_games;
    let packages_to_covered_games: &Vec<BitSet> = packages_to_covered_games;
    let all_games_covered: &BitSet = Box::leak(Box::new(all_covered_games));
    let all_games_covered_by_monthly_packages: &BitSet =
        Box::leak(Box::new(all_games_covered_by_monthly_packages));
    let uniquely_covered_games_set: &BitSet = Box::leak(Box::new(uniquely_covered_games_set));

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                games,
                offers,
                packages,
                teams,
                tournaments: tournamets,
                tournament_to_games,
                packages_to_covered_games,
                teams_to_games,
                uniquely_covered_games_map,
                uniquely_covered_games_set,
                all_games_covered,
                all_games_covered_by_monthly_packages,
            }))
            .service(get_streaming_packages)
            .service(get_teams)
            .service(get_games)
            .service(get_tournaments)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
