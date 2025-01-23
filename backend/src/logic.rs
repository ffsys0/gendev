use bit_set::BitSet;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::models::AppState;
struct Subset {
    packages: Vec<usize>,
    price: u32,
    count_packages: usize,
    start_idx: u32,
    covered: BitSet,
}

impl Eq for Subset {}

impl PartialEq for Subset {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.count_packages == other.count_packages
    }
}
impl PartialOrd for Subset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // We want (sum, len) to be in ascending order. BinaryHeap is a max-heap, so we invert:
        // If self.sum < other.sum, we want self to have HIGHER priority (so we return Ordering::Greater)
        match self.price.cmp(&other.price) {
            Ordering::Less => Some(Ordering::Greater),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Equal => {
                // If sums are equal, compare by length ascending
                match self.count_packages.cmp(&other.count_packages) {
                    Ordering::Less => Some(Ordering::Greater),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Equal => Some(Ordering::Equal),
                }
            }
        }
    }
}

impl Ord for Subset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
// does not remove package which is more expensive if it was added first in coverage_list?! just sort by price maybe lol?
pub fn discard_packages_which_cover_same_games(required_games: &BitSet, packages: &mut Vec<usize>, state: &AppState) {
    let mut coverage_list = Vec::new();
    packages.retain(|&pkg_idx| {
        let mut covered_req = state.packages_to_covered_games[pkg_idx].clone();
        covered_req.intersect_with(required_games);
        let price = state.packages.iter()
            .find(|p| p.id == pkg_idx)
            .unwrap()
            .monthly_price_yearly_subscription_in_cents // naja so halb richtig weil was wenn montly billing oder so?
            .unwrap();
        for (pkg_id, coverage) in coverage_list.iter() {
            if covered_req.is_subset(coverage) && price >= state.packages.iter().find(|p| p.id == *pkg_id).unwrap().monthly_price_yearly_subscription_in_cents.unwrap() {
                return false;
            }
        }
        coverage_list.push((pkg_idx, covered_req.clone()));
        true
    });
}


pub fn find_minimal_packages(
    required_games: &BitSet,
    state: &AppState,
    consider_packages: Vec<usize>, // already respects the monthly bound
    monthly: bool,
) -> Option<Vec<usize>> {
    let mut consider_packages = consider_packages;
    log::debug!("consider_packages: {}", consider_packages.len());
    // consider_packages.sort_by_key(|&pkg_idx| {
    //     state.packages.iter().find(|p| p.id == pkg_idx).unwrap().monthly_price_yearly_subscription_in_cents.unwrap()
    // });
    discard_packages_which_cover_same_games(required_games, &mut consider_packages, state);
    let mut result: Vec<usize> = Vec::new();
    let mut heap: BinaryHeap<Subset> = BinaryHeap::new();
    let my_copy: Vec<BitSet> = state
    .packages_to_covered_games
    .iter()
    .map(|set| BitSet::from_iter(set.iter()))
    .collect();
    log::debug!("consider_packages: {}", consider_packages.len());
    let threshhold = 4;
    let mut coverage_count = HashMap::new();
    for (pkg_idx, coverage) in state.packages_to_covered_games.iter().enumerate() {
        for game_id in coverage.iter() {
            if required_games.contains(game_id) {
                *coverage_count.entry(game_id).or_insert(0) += 1;
            }
        }
    }
    // what about edge case where already everything is covered??? is this possible? TODO
    for game_id in required_games.iter() {
        if coverage_count.get(&game_id).unwrap_or(&0) <= &threshhold {
            for (pkg_idx, coverage) in state.packages_to_covered_games.iter().enumerate() { // iters over all packages possibly not respecting monthly bound
                if coverage.contains(game_id) {
                    if monthly && state.packages.iter().find(|p| p.id == pkg_idx).unwrap().monthly_price_cents.is_none(){
                        continue;
                    }
                    let mut single_covered = BitSet::new();
                    single_covered.union_with(&my_copy[pkg_idx]);
                    let single_price = if monthly {
                        state.packages.iter().find(|p| p.id == pkg_idx).unwrap().monthly_price_cents.unwrap()
                    } else {
                        state.packages.iter().find(|p| p.id == pkg_idx).unwrap().monthly_price_yearly_subscription_in_cents.unwrap()
                    };
                    heap.push(Subset {
                        packages: vec![pkg_idx],
                        price: single_price,
                        count_packages: 1,
                        start_idx: 0,
                        covered: single_covered,
                    });
                }
            }
        }
    }
    // if no hard games just start with empty
    if heap.is_empty() {
        heap.push(Subset {
            packages: Vec::new(),
            price: 0,
            count_packages: 0,
            start_idx: 0,
            covered: BitSet::new(),
        });
    }

    let consider_packages = consider_packages;
    let my_requ = BitSet::from_iter(required_games.iter());


    let mut covered = BitSet::new();
    let mut visited_coverage: HashMap<BitSet, u32> = HashMap::new();

    // iterate over all powersets of considered_packages
    // ordered by price first and by amount packages second
    while !heap.is_empty() {
        let current = heap.pop().unwrap();
        for p in current.packages.iter() {
            covered.union_with(&my_copy[*p]);
        }
        if my_requ.is_subset(&current.covered) {
            result.extend(current.packages);
            return Some(result);
        }
        if let Some(&known_price) = visited_coverage.get(&current.covered) {
            if known_price <= current.price {
                // We already have as cheap or cheaper coverage. Skip.
                continue;
            }
        }
        visited_coverage.insert(current.covered.clone(), current.price);

        for i in (current.start_idx as usize)..consider_packages.len() {
            let elem = *consider_packages.get(i).unwrap();
            let mut new_packages = current.packages.clone();
            new_packages.push(elem);
            let mut tmp_covered = current.covered.clone();
            tmp_covered.union_with(&my_copy[elem]);
            let new_subset = Subset {
                price: current.price
                    + if monthly {
                        state.packages.iter().find(|p| p.id == elem).unwrap().monthly_price_cents.unwrap()
                    } else {
                        state.packages.iter().find(|p| p.id == elem).unwrap().monthly_price_yearly_subscription_in_cents.unwrap()
                    },
                count_packages: current.count_packages + 1,
                start_idx: (i + 1) as u32,
                packages: new_packages,
                covered: tmp_covered,
            };
            heap.push(new_subset);
        }
    }
    None
}
