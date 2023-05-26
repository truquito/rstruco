use serde::{Deserialize, Serialize};
use crate::carta::{Carta};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, Ord, 
  PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum NumMano {
  Primera,
  Segunda,
  Tercera
}

impl NumMano {
  pub fn to_int(&self) -> usize {
    (*self as usize) + 1
  }

  pub fn inc(&self) -> NumMano {
    match self {
      NumMano::Primera => NumMano::Segunda,
      NumMano::Segunda => NumMano::Tercera,
      _ => unreachable!()
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Resultado {
  Indeterminado,
  GanoRojo,
  GanoAzul,
  Empardada
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartaTirada {
  pub jugador: String,
  pub carta: Carta,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Mano {
  pub resultado: Resultado,
  pub ganador: String,
  pub cartas_tiradas: Vec<CartaTirada>,
}

impl Mano {
  pub fn agregar_tirada(&mut self, carta_tirada: CartaTirada) {
    self.cartas_tiradas.push(carta_tirada);
  }
}