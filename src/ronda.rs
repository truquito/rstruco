use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::mano::{NumMano};
use crate::equipo::{Equipo};
use crate::envite::{Envite};
use crate::truco::{Truco};
use crate::manojo::{Manojo};
use crate::carta::{Carta};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Ronda<'a> {
  pub mano_en_juego: NumMano,
  pub cant_jugadores_en_juego: HashMap<Equipo, usize>,
  // Indices
	pub el_mano: usize,
	pub turno:  usize,
  // gritos y toques/cantos
  pub envite: Envite<'a>,
	pub truco: Truco,

  // cartas
  pub manojos: Vec<Manojo>,
  pub muestra: Carta,
  
  // pub manos: [Mano; 3],
  
  // otros
  #[serde(skip_deserializing, skip_serializing)]
  pub mixs: HashMap<String, usize>,
}