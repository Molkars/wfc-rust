#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::mem::replace;

use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

pub use view::*;

mod view;

pub trait State: Clone + PartialOrd + Ord {}
impl State for i32 {}
// impl<T: State + PartialOrd + Ord> State for T {}

/// The generic tile class for the WFC algorithm
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tile<T: State> {
    Definite(T),
    Indefinite(BTreeSet<T>),
}

/// A controller for dictating rules of the WFC algorithm
pub trait WfcRules<T: State>: Sized {
    /// Returns the valid states that are possible in [map.pos()]
    fn get_states(&self, map: WfcView<'_, T, Self>) -> BTreeSet<T>;

    fn entropy(&self, _tile: &Tile<T>) -> f64 {
        0.0
    }
}

/// The main structure for the WFC algorithm
#[derive(Debug)]
pub struct Wfc<T: State, R: WfcRules<T>> {
    width: usize,
    height: usize,
    rules: R,
    map: Vec<Tile<T>>,
}

impl<T: State, R: WfcRules<T>> Wfc<T, R> {
    /// Creates a new WFC using
    pub fn new(width: usize, height: usize, tiles: Vec<Tile<T>>, rules: R) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert_eq!(tiles.len(), width * height, "Tiles.len() must be w*h");

        Self {
            map: tiles,
            width,
            height,
            rules,
        }
    }

    /// Returns the width of the map
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the map
    #[inline(always)]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a new view centered at [x], [y]
    pub fn view(&self, idx: usize) -> WfcView<T, R> {
        assert!(idx < self.width * self.height, "x & y must be inside wfc map");
        WfcView {
            pos: (idx % self.width, idx / self.width),
            wfc: self,
        }
    }

    /// Converts an xy-pair into two (x, y) coordinates
    pub fn xy_pair(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

impl<T: State> Tile<T> {
    pub fn as_definite(&self) -> &T {
        match self {
            Tile::Definite(s) => s,
            _ => panic!("as_definite called on variant that was not Tile::Definite"),
        }
    }

    pub fn into_definite(self) -> T {
        match self {
            Tile::Definite(s) => s,
            _ => panic!("as_definite called on variant that was not Tile::Definite"),
        }
    }

    pub fn as_indefinite(&self) -> &BTreeSet<T> {
        match self {
            Tile::Indefinite(states) => states,
            _ => panic!("as_indefinite called on variant that was not Tile::Definite"),
        }
    }

    pub fn into_indefinite(self) -> BTreeSet<T> {
        match self {
            Tile::Indefinite(states) => states,
            _ => panic!("as_indefinite called on variant that was not Tile::Definite"),
        }
    }
}

impl<T: State, R: WfcRules<T>> Wfc<T, R> {
    pub fn step(&mut self) -> Option<()> {
        let entropy_map = {
            let mut map = self.map
                .iter()
                .filter(|tile| matches!(tile, Tile::Indefinite(_)))
                .map(|tile| self.rules.entropy(tile))
                .enumerate()
                .collect::<Vec<_>>();
            map.sort_by(|(_, a), (_, b)|
                a.partial_cmp(b).expect("Unable to compare tiles!"));
            map
        };

        if entropy_map.is_empty() {
            return None; // This means the filter removed everything so every state is definite
        }

        let next_highest = {
            let mut iter = entropy_map.iter();
            let (_, highest_entropy) = iter.next().unwrap();
            iter.position(|(_, e)| e.ne(highest_entropy))
        };
        let selected = match next_highest {
            Some(next_highest) => {
                // collapse random tile in 0..next_highest
                let mut rng = thread_rng();
                let tiles = &entropy_map[0..next_highest];
                tiles.choose(&mut rng)
            }
            None => {
                // collapse random tile
                let mut rng = thread_rng();
                entropy_map.choose(&mut rng)
            }
        }.expect("No states left!")
            .0;

        let old = {
            let states = self.map[selected].as_indefinite();
            let mut rng = thread_rng();
            let idx = rng.gen_range(0..states.len());
            let state = states.iter().nth(idx).unwrap().clone();

            let mut states = replace(&mut self.map[selected], Tile::Definite(state))
                .into_indefinite();
            states.remove(&state);
            states
        };

        if entropy_map.is_empty() {
            return None;
        }

        let mut valid = true;
        let mut states = Vec::with_capacity(entropy_map.len() - 1);
        for (idx, _) in entropy_map {
            if idx == selected {
                continue; // This is the collapsed state;
            }
            let view = self.view(idx);
            let collapsed = self.rules.get_states(view);
            match collapsed.len() {
                0 => {
                    valid = false;
                    break;
                }
                _ => states.push((idx, collapsed)),
            };
        }

        if !valid {
            if old.is_empty() {
                return None; // No alternatives for the selected tile; Todo: work on history
            }
            // Since we removed the randomly chosen state from the old vec,
            // The next iteration will not make the same mistake
            self.map[selected] = Tile::Indefinite(old);
        }

        for (idx, states) in states {
            let tile = match states.len() {
                0 => unreachable!(),
                1 => Tile::Definite(states.into_iter().next().unwrap()),
                _ => Tile::Indefinite(states),
            };
            self.map[idx] = tile;
        }

        Some(())
    }
}