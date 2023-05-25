use std::fmt;
use serde::{Deserialize, Serialize};

use crate::Manojo;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, Ord, 
  PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum EstadoEnvite {
  Deshabilitado,
  NoCantadoAun, // deberia ser "No tocado"
  Envido,
  RealEnvido,
  FaltaEnvido,
  Flor,
  ContraFlor,
  ContraFlorAlResto,
} 

impl fmt::Display for EstadoEnvite {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      EstadoEnvite::Deshabilitado     => write!(f, "Deshabilitado"),
      EstadoEnvite::NoCantadoAun      => write!(f, "NoCantadoAun"),
      EstadoEnvite::Envido            => write!(f, "Envido"),
      EstadoEnvite::RealEnvido        => write!(f, "RealEnvido"),
      EstadoEnvite::FaltaEnvido       => write!(f, "FaltaEnvido"),
      EstadoEnvite::Flor              => write!(f, "Flor"),
      EstadoEnvite::ContraFlor        => write!(f, "ContraFlor"),
      EstadoEnvite::ContraFlorAlResto => write!(f, "ContraFlorAlResto"),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Envite<'a> {
  pub estado: EstadoEnvite,
  pub puntaje: usize,
  pub cantado_por: String,
  #[serde(skip_deserializing, skip_serializing)]
  pub jugadores_con_flor: Vec<&'a Manojo>,
  // alternativa
  // pub jugadores_con_flor: Vec<String>,
  pub sin_cantar: Vec<String>,
}

impl Envite<'_> {
  pub fn no_canto_flor_aun(&self, j:&String) -> bool {
    self.sin_cantar.contains(j)
  }

  // Elimina a `j` de los jugadores que tienen pendiente cantar flor
  pub fn canto_flor(&mut self, j:&String) {
    // if j in self.sin_cantar:
    // self.sin_cantar.remove(j)
    if let Some(pos) = self.sin_cantar.iter().position(|x| *x == *j) {
      self.sin_cantar.remove(pos);
    }
  }
}