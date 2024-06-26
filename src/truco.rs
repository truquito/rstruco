use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, Ord, 
  PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum EstadoTruco {
  NoCantado,
  Truco,
  TrucoQuerido,
  ReTruco,
  ReTrucoQuerido,
  Vale4,
  Vale4Querido,
}

impl fmt::Display for EstadoTruco {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      EstadoTruco::NoCantado       => write!(f, "No cantado"),
      EstadoTruco::Truco           => write!(f, "Truco"),
      EstadoTruco::TrucoQuerido    => write!(f, "Truco querido"),
      EstadoTruco::ReTruco         => write!(f, "Retruco"),
      EstadoTruco::ReTrucoQuerido  => write!(f, "Retruco querido"),
      EstadoTruco::Vale4           => write!(f, "Vale4"),
      EstadoTruco::Vale4Querido    => write!(f, "Vale4 querido"),
    }
  }
}

impl EstadoTruco {
  pub fn es_truco_respondible(&self) -> bool {
    [EstadoTruco::Truco, EstadoTruco::ReTruco, EstadoTruco::Vale4].contains(&self)
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Truco {
  pub cantado_por: String,
  pub estado: EstadoTruco,
}

impl Truco {
  pub fn new() -> Truco {
    Truco{
      cantado_por: String::from(""),
      estado: EstadoTruco::NoCantado,
    }
  }
  pub fn reset(&mut self) {
    self.cantado_por = String::from("");
    self.estado = EstadoTruco::NoCantado;
  }
}