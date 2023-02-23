mod utils;

use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate fixedbitset;
extern  crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use fixedbitset::FixedBitSet;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
// Entirety of the board in Conway's game of life.
// It wraps around the edges, and is in practice represented as a single vector
// of cells, and not a matrix, to ease integration into Wasm.
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;

        let size = (width * self.height) as usize;
        self.cells.grow(size);

        for i in 0..size {
            self.cells.set(i, false);
        }
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;

        let size = (self.width * height) as usize;
        self.cells.grow(size);

        for i in 0..size {
            self.cells.set(i, false);
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                /*
                log!(
                     "cell[{}, {}] is initially {:?} and has {} live neighbors",
                     row,
                     col,
                     cell,
                     live_neighbors
                 );
                */

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (true, x) if x < 2 => false,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (true, x) if x > 3 => false,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                //log!("    it becomes {:?}", next_cell);

                next.set(idx, next_cell);

                if self.cells[idx] != next[idx] {
                    log!(
                        "cell[{}, {}] is initially {:?} and became {}",
                        row,
                        col,
                        self.cells[idx],
                        next[idx]
                    );
                }
            }
        }

        self.cells = next;
    }
}

use core::panic;
use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let index = self.get_index(row, col);
                let cell = self.cells[index];
                let symbol = if cell { '◼' } else { '◻' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

/// Public methods, exported to JavaScript.
/// This `impl` block is mostly for constructors.
#[wasm_bindgen]
impl Universe {
    // Deterministic universe with random cell states, 64 by 64.
    pub fn hardcoded_64_by_64() -> Universe {

        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, i % 2 == 0 || i % 7 == 0);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    // Create a universe with a random initial position, in a non-deterministic
    // manner unlike the method above.
    // The grid's dimensions are passed as an argument.
    pub fn new(width : u32, height : u32) -> Universe {
        utils::set_panic_hook();

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0 .. size {
                if js_sys::Math::random() < 0.5 {
                    cells.set(i, true);
                } else {
                    cells.set(i, false);
                }
            };

        /*
        Exercise 2 of the debugging chapter.
        Uncomment this, comment the marked line in Cargo.toml, and run `wasm-pack build --release`.

        panic!("Just for testing purposes");
        */

        Universe {
            width,
            height,
            cells,
        }
    }

    // Create one instance of Gosper's glider at the center of
    // the board.
    pub fn new_glider_at(&mut self, row: u32, column: u32) {
        let mut i = 0;
        let neighbourhood = [
                false, true, true,
                true, false, true,
                false, false, true,
            ];

        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                self.cells.set(idx, neighbourhood[i]);
                i += 1;
            }
        }
    }

    // Create a new, empty universe with the given size, and a glider
    // at the center of the board.
    pub fn new_with_spaceship(width : u32, height : u32) -> Universe {
        utils::set_panic_hook();

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0 .. (width * height) as usize {
            cells.set(i, false)
        };

        let mut u = Universe {
            width,
            height,
            cells,
        };

        u.new_glider_at(width / 2, height / 2);
        u
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }

}

#[test]
fn test_display() {
    let universe = Universe::new_with_spaceship(16, 16);
    print!("{}", universe.to_string());
}