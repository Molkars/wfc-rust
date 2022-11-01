#![allow(dead_code)]

use std::fmt::Debug;

mod view;
pub use view::*;

/// The generic tile class for the WFC algorithm
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tile<T> {
    Definite(T),
    Indefinite(Vec<T>),
}

/// A controller for dictating rules of the WFC algorithm
pub trait WfcRules<T>: Sized {
    /// Returns the valid states that are possible in [map.pos()]
    fn get_states(&self, map: &WfcView<'_, T, Self>) -> Vec<T>;
}

/// The main structure for the WFC algorithm
#[derive(Debug)]
pub struct Wfc<T, R: WfcRules<T>> {
    width: usize,
    height: usize,
    rules: R,
    map: Vec<Tile<T>>,
}

impl<T, R: WfcRules<T>> Wfc<T, R> {
    /// Creates a new WFC using
    pub fn new(width: usize, height: usize, tiles: Vec<Tile<T>>, rules: R) -> Self {
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
    pub fn view(&self, x: usize, y: usize) -> WfcView<T, R> {
        assert!(x < self.width, "x must be inside wfc map");
        assert!(y < self.height, "y must be inside wfc map");
        WfcView {
            pos: (x, y),
            wfc: self,
        }
    }
}


impl<T, R: WfcRules<T>> Wfc<T, R> {

}