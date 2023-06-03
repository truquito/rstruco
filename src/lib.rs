// imports
mod equipo;
mod jugador;
mod carta;
mod manojo;
mod mano;
mod truco;
mod envite;
mod ronda;
mod partida;
mod jugada;
pub mod enco;
pub mod chi;

// `use` ~ import without namespace
// `pub` ~ export

// declaralos aca como pub hace que sea posible usarlos en /test
pub use self::equipo::{*};
pub use self::jugador::{*};
pub use self::carta::{*};
pub use self::manojo::{*};
pub use self::mano::{*};
pub use self::truco::{*};
pub use self::envite::{*};
pub use self::ronda::{*};
pub use self::partida::{*};
pub use self::jugada::{*};
pub use self::enco::{*};
pub use self::chi::{*};