use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EstadoTruco {
  no_cantado,
  truco,
  truco_querido,
  re_truco,
  re_truco_querido,
  vale4,
  vale4_querido,
}

impl fmt::Display for EstadoTruco {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      EstadoTruco::no_cantado       => write!(f, "No cantado"),
      EstadoTruco::truco            => write!(f, "Truco"),
      EstadoTruco::truco_querido    => write!(f, "Truco querido"),
      EstadoTruco::re_truco         => write!(f, "Retruco"),
      EstadoTruco::re_truco_querido => write!(f, "Retruco querido"),
      EstadoTruco::vale4            => write!(f, "Vale4"),
      EstadoTruco::vale4_querido    => write!(f, "Vale4 querido"),
    }
  }
}

impl EstadoTruco {
  pub fn es_truco_respondible(&self, e: EstadoTruco) -> bool {
    [EstadoTruco::truco, EstadoTruco::re_truco, EstadoTruco::vale4].contains(e)
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Truco {
  pub cantado_por: String,
  pub estado: EstadoTruco,
}