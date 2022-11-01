use std::collections::{BTreeSet, HashSet};

use crate::wfc::{State, Tile, WfcRules, WfcView};

mod wfc;

pub struct SudokuRules;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SudokuNum {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl SudokuNum {
    pub fn full_set() -> BTreeSet<Self> {
        BTreeSet::from([
            Self::One,
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
        ])
    }
}

type View<'a> = WfcView<'a, SudokuNum, SudokuRules>;

// impl SudokuRules {
//     fn check_column(view: &View) -> HashSet<SudokuNum> {
//         let mut missing = SudokuNum::full_set();
//         for item in view.col().row_iter() {
//             match item {
//                 Tile::Definite(state) => missing.remove(state),
//                 Tile::Indefinite(states) => {
//                     missing = missing
//                         .intersection(HashSet::from_iter(states.iter().cloned()))
//                         .collect::<HashSet<_>>();
//                 }
//             };
//         }
//         missing
//     }
// }

impl State for SudokuNum {}

impl WfcRules<SudokuNum> for SudokuRules {
    fn get_states(&self, map: View<'_>) -> BTreeSet<SudokuNum> {
        fn states<'a, I: Iterator<Item=&'a Tile<SudokuNum>>>(i: I) -> BTreeSet<&'a SudokuNum> {
            i.filter_map(|tile| match tile {
                Tile::Definite(s) => Some(s),
                Tile::Indefinite(_) => None
            }).collect()
        }

        let row = map.row();
        let col = map.col();
        let (x, y) = map.pos();
        let block = map.section_at(3, 3, *x, *y);

        let row_states = states(row.row_iter());
        let col_states = states(col.row_iter());
        let block_states = states(block.row_iter());
        let mut possible = SudokuNum::full_set();
        for states in [row_states, col_states, block_states] {
            for state in states {
                possible.remove(state);
            }
        }
        possible
    }
}

fn main() {
    println!("Hello, world!");
}