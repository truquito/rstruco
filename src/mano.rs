use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, Ord, 
  PartialOrd)]
// #[serde(rename_all = "lowercase")]
pub enum NumMano {
  Primera = 0,
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