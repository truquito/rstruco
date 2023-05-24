use serde::{Deserialize, Serialize};
// use crate::equipo;
use crate::equipo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jugador {
  pub id: String,
  pub equipo: equipo::Equipo,
}