use bit_set::BitSet;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a single game row from CSV
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Game {
    pub(crate) id: usize, // for use with Bitset
    pub(crate) team_home: String,
    pub(crate) team_away: String,
    pub(crate) starts_at: String,
    pub(crate) tournament_name: String,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} vs {} ({})", self.team_home, self.team_away, self.tournament_name)
    }
}
/// Represents a single offer row from CSV
#[derive(Deserialize)]
pub struct Offer {
    pub(crate) game_id: usize, // for consistency usize as well instead of u32
    pub(crate) streaming_package_id: usize,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) live: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) highlights: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Package {
    pub(crate) id: usize, // usize for use with Bitset
    pub(crate) name: String,
    pub(crate) monthly_price_cents: Option<u32>,
    pub(crate) monthly_price_yearly_subscription_in_cents: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct GetResponse {
    pub(crate) packages: Vec<&'static Package>,
    pub(crate) rows: Vec<Row>,
    pub(crate) result: Vec<&'static Package>,
}

#[derive(Deserialize)]
pub(crate) struct GetQuery {
    pub(crate) games: String,
    pub(crate) teams: String,
    pub(crate) tournaments: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) all_games: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) only_monthly_billing: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) live: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub(crate) highlights: bool,
}

pub(crate) struct AppState {
    pub(crate) games: &'static Vec<Game>,
    pub(crate) offers: &'static Vec<Offer>,
    pub(crate) packages: &'static Vec<Package>,
    pub(crate) teams: &'static Vec<&'static String>,
    pub(crate) tournaments: &'static Vec<&'static String>,

    pub(crate) tournament_to_games: &'static HashMap<&'static String, BitSet>,
    pub(crate) packages_to_covered_games: &'static Vec<BitSet>,
    pub(crate) teams_to_games: &'static HashMap<&'static String, BitSet>,
    pub(crate) uniquely_covered_games_map: &'static HashMap<usize, usize>,
    pub(crate) uniquely_covered_games_set: &'static BitSet,
    pub(crate) all_games_covered: &'static BitSet,
    pub(crate) all_games_covered_by_monthly_packages: &'static BitSet,
}


#[derive(Serialize)]

pub(crate) enum Coverage {
    FULL,
    PARTIAL,
    NONE,
}



#[derive(Serialize)]
pub struct Row {
    pub key: String,
    pub provider_coverage: HashMap<String, Coverage>,
    pub provider_coverage_highlights: HashMap<String, Coverage>,
    pub sub_rows: Option<Vec<Row>>,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(de::Error::custom("Invalid value for bool")),
    }
}
