use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Equipo {
  Azul,
  Rojo,
}

impl Equipo {
  pub fn equipo_contrario(&self) -> Equipo {
    match self {
      Equipo::Azul => Equipo::Rojo,
      Equipo::Rojo => Equipo::Azul,
    }
  }
}

impl fmt::Display for Equipo {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     match self {
      Equipo::Azul => write!(f, "Azul"),
      Equipo::Rojo => write!(f, "Rojo"),
     }
  }
}
