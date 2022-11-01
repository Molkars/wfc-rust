use std::fmt::Debug;
use std::ops::Range;

use crate::wfc::{Tile, Wfc, WfcRules};

/// A view of the WFC map
///
/// Comes with utility methods to inspect parts of the map in order to determine
/// valid states in the WfcRules
#[derive(Debug)]
pub struct WfcView<'wfc, T: Sized, R: WfcRules<T>> where Self: 'wfc {
    pub(super) wfc: &'wfc Wfc<T, R>,
    pub(super) pos: (usize, usize),
}

impl<'wfc, T, R: WfcRules<T>> WfcView<'wfc, T, R> where Self: 'wfc {
    /// Returns the width of the map
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.wfc.width()
    }

    /// Returns the height of the map
    #[inline(always)]
    pub fn height(&self) -> usize {
        self.wfc.height()
    }

    /// Returns the position associated with this view
    #[inline(always)]
    pub fn pos(&self) -> &(usize, usize) {
        &self.pos
    }

    /// Returns a span of the elements in the row at [row]
    /// 
    /// # Panics
    /// * If row >= self.height()
    pub fn row(&self, row: usize) -> Span<'wfc, T> {
        let height = self.height();
        assert!(row < height, "row must be inside of the map's height");
        let idx = row * self.width();
        Span(vec![&self.wfc.map[idx..idx + height]])
    }

    /// Returns a span of the the elements in the column at [col]
    ///
    /// # Panics
    /// * If col >= self.width()
    pub fn col(&self, col: usize) -> Span<'wfc, T> {
        let width = self.width();
        assert!(col < width, "column must be inside of the map's width");
        Span(self.wfc.map.as_slice()
            .chunks(width)
            .map(|chunk| &chunk[col..col + 1])
            .collect())
    }

    /// Returns a span of the elements in the rectangle formed by the area of [x] and [y]
    /// 
    /// # Panics
    /// * If [x].len() == 0
    /// * If [x].len() == 0
    /// * If [x].end >= self.width()
    /// * If [y].end >= self.height()
    pub fn span(&self, x: Range<usize>, y: Range<usize>) -> Span<'wfc, T> {
        assert_ne!(x.len(), 0, "x-range cannot be zero-width");
        assert_ne!(y.len(), 0, "y-range cannot be zero-height");
        let width = self.width();

        assert!(x.end < width, "x-range must be inside of the map's width");
        assert!(y.end < self.height(), "y-range must be inside of the map's height");

        Span(self.wfc.map.as_slice()
            .chunks(width)
            .take(y.end)
            .skip(y.start)
            .map(move |chunk| &chunk[x.clone()])
            .collect())
    }

    /// Returns the span in [x] from the row at [row]
    ///
    /// # Panics
    /// * If [row] >= self.height()
    /// * If [x].len() == 0
    /// * If [x].end >= self.width()
    pub fn row_span(&self, row: usize, x: Range<usize>) -> Span<'wfc, T> {
        let width = self.width();
        assert!(row < self.height(), "row must be inside of the map's height");
        assert_ne!(x.len(), 0, "x-range cannot be zero-width");
        assert!(x.end < width, "x-range must be inside of the map's width");

        let y = row * width;
        let y0 = y + x.start;
        let y1 = y + x.end;
        Span(vec![&self.wfc.map.as_slice()[y0..y1]])
    }

    /// Returns the span in [y] from the column at [col]
    ///
    /// # Panics
    /// * If [col] >= self.width()
    /// * If [y].len() == 0
    /// * If [y].end >= self.height()
    pub fn col_span(&self, col: usize, y: Range<usize>) -> Span<'wfc, T> {
        let width = self.width();
        assert!(col < self.width(), "col must be inside of the map's width");
        assert_ne!(y.len(), 0, "y-range cannot be zero-height");
        assert!(y.end < self.height(), "y-range must be inside of the map's width");

        Span(self.wfc.map
            .chunks(width)
            .take(y.end)
            .skip(y.start)
            .map(move |chunk| &chunk[col..col + 1])
            .collect())
    }

    /// Returns the tile at the xy pair: [col], [row]
    pub fn get_at(&self, row: usize, col: usize) -> &'wfc Tile<T> {
        &self.wfc.map[row * self.width() + col]
    }

    #[inline(always)]
    /// Returns the tile at self.pos()
    pub fn get(&self) -> &'wfc Tile<T> {
        let (row, col) = self.pos;
        self.get_at(row, col)
    }
}

#[derive(Debug)]
pub struct Span<'wfc, T>(Vec<&'wfc [Tile<T>]>) where Self: 'wfc;

impl<'wfc, T> Span<'wfc, T> where Self: 'wfc {
    /// Returns the length of each row in this span
    pub fn width(&self) -> usize {
        self.0.get(0)
            .map(|slice| slice.len())
            .unwrap_or(0)
    }

    /// Returns the length of the columns in this span
    pub fn height(&self) -> usize {
        self.0.len()
    }

    /// Returns a row-iterator for this span
    pub fn row_iter<'a>(&'a self) -> RowIter<'a, 'wfc, T> {
        RowIter {
            span: self,
            y: 0,
            x: 0,
        }
    }

    /// Returns a new column-iterator for this span
    pub fn col_iter<'a>(&'a self) -> ColIter<'a, 'wfc, T> {
        ColIter {
            span: self,
            x_idx: 0,
            y_idx: 0,
        }
    }
}


/// An iterator for the rows in a [Span]
pub struct RowIter<'span, 'wfc, T> where 'span: 'wfc {
    span: &'span Span<'wfc, T>,
    y: usize,
    x: usize,
}

impl<'span, 'wfc, T> Iterator for RowIter<'span, 'wfc, T> where 'span: 'wfc {
    type Item = &'wfc Tile<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < self.span.0.len() {
            let out = self.span.0[self.y];
            if self.x < out.len() {
                let out = &out[self.x];
                self.x += 1;
                Some(out)
            } else {
                self.y += 1;
                self.x = 1;
                if self.y < self.span.0.len() {
                    Some(&self.span.0[self.y][0])
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}

/// An iterator for the columns in a [Span]
pub struct ColIter<'span, 'wfc, T> {
    span: &'span Span<'wfc, T>,
    y_idx: usize,
    x_idx: usize,
}

impl<'span, 'wfc, T> Iterator for ColIter<'span, 'wfc, T> {
    type Item = &'wfc Tile<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.x_idx, self.y_idx) {
            (x, y) if x == self.span.width() && y == self.span.height() => None,
            (_, y) if y == self.span.height() => {
                self.y_idx = 1;
                self.x_idx += 1;
                Some(self.x_idx)
                    .filter(|x| *x != self.span.width())
                    .map(|x| &self.span.0[0][x])
            }
            (_, y) => {
                self.y_idx += 1;
                Some(&self.span.0[y][self.x_idx])
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::wfc::{Tile, Wfc, WfcRules, WfcView};

    #[derive(Debug)]
    struct S;

    impl WfcRules<i32> for S {
        fn get_states(&self, _: &WfcView<'_, i32, Self>) -> Vec<i32> {
            vec![0]
        }
    }

    fn wfc() -> Wfc<i32, S> {
        Wfc::new(4, 4, (0..16).map(Tile::Definite).collect(), S)
    }

    #[test]
    fn row_iter() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", wfc);

        let span = view.span(1..3, 0..3);
        println!("{:?}", span);

        println!("{:?}", span.row_iter().collect::<Vec<_>>());
        let mut iter = span.row_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(1)));
        assert_eq!(iter.next(), Some(&Tile::Definite(2)));
        assert_eq!(iter.next(), Some(&Tile::Definite(5)));
        assert_eq!(iter.next(), Some(&Tile::Definite(6)));
        assert_eq!(iter.next(), Some(&Tile::Definite(9)));
        assert_eq!(iter.next(), Some(&Tile::Definite(10)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col_iter() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", view);

        let span = view.span(1..3, 0..3);
        println!("{:?}", span);

        println!("{:?}", span.col_iter().collect::<Vec<_>>());
        let mut iter = span.col_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(1)));
        assert_eq!(iter.next(), Some(&Tile::Definite(5)));
        assert_eq!(iter.next(), Some(&Tile::Definite(9)));
        assert_eq!(iter.next(), Some(&Tile::Definite(2)));
        assert_eq!(iter.next(), Some(&Tile::Definite(6)));
        assert_eq!(iter.next(), Some(&Tile::Definite(10)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", view);

        let row = view.row(0);
        let mut iter = row.row_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(0)));
        assert_eq!(iter.next(), Some(&Tile::Definite(1)));
        assert_eq!(iter.next(), Some(&Tile::Definite(2)));
        assert_eq!(iter.next(), Some(&Tile::Definite(3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", view);

        let col = view.col(0);
        println!("{:?}", col);

        let mut iter = col.row_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(0)));
        assert_eq!(iter.next(), Some(&Tile::Definite(4)));
        assert_eq!(iter.next(), Some(&Tile::Definite(8)));
        assert_eq!(iter.next(), Some(&Tile::Definite(12)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row_span() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", view);

        let span = view.row_span(0, 1..3);
        let mut iter = span.row_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(1)));
        assert_eq!(iter.next(), Some(&Tile::Definite(2)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col_span() {
        let wfc = wfc();
        let view = wfc.view(0, 0);
        println!("{:?}", view);

        let span = view.col_span(0, 1..3);
        let mut iter = span.row_iter();
        assert_eq!(iter.next(), Some(&Tile::Definite(4)));
        assert_eq!(iter.next(), Some(&Tile::Definite(8)));
        assert_eq!(iter.next(), None);
    }
}