pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// imports
mod equipo;
mod jugador;
mod carta;
mod manojo;
mod mano;
mod truco;
// mod envite;

// `use` ~ import without namespace
// `pub` ~ export
pub use equipo::{*};
pub use jugador::{*};
pub use carta::{*};
pub use manojo::{*};
pub use mano::{*};
pub use truco::{*};
// pub use envite::{*};