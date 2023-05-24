pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// imports
mod equipo;
mod jugador;
mod carta;

// `use` ~ import without namespace
// `pub` ~ export
pub use equipo::{*};
pub use jugador::{*};
pub use carta::{*};