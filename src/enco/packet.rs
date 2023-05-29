use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Packet {
  pub destination: Vec<String>,
  pub message: Message,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Razon {
  EnvidoGanado,
  RealEnvidoGanado,
  FaltaEnvidoGanado,
  EnviteNoQuerido,
  FlorAchicada,
  LaUnicaFlor,
  LasFlores,
  LaFlorMasAlta,
  ContraFlorGanada,
  ContraFlorAlRestoGanada,
  TrucoNoQuerido,
  TrucoQuerido,
  SeFueronAlMazo,
}

impl fmt::Display for Razon {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Razon::EnvidoGanado => write!(f, "Envido Ganado"),
      Razon::RealEnvidoGanado => write!(f, "Real Envido Ganado"),
      Razon::FaltaEnvidoGanado => write!(f, "Falta Envido Ganado"),
      Razon::EnviteNoQuerido => write!(f, "Envite No Querido"),
      Razon::FlorAchicada => write!(f, "Flor Achicada"),
      Razon::LaUnicaFlor => write!(f, "La Unica Flor"),
      Razon::LasFlores => write!(f, "Las Flores"),
      Razon::LaFlorMasAlta => write!(f, "La Flor Mas Alta"),
      Razon::ContraFlorGanada => write!(f, "Contra Flor Ganada"),
      Razon::ContraFlorAlRestoGanada => write!(f, "Contra Flor Al Resto Ganada"),
      Razon::TrucoNoQuerido => write!(f, "Truco No Querido"),
      Razon::TrucoQuerido => write!(f, "Truco Querido"),
      Razon::SeFueronAlMazo => write!(f, "Se Fueron Al Mazo"),
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Content {
  // sin nada
  LaManoResultaParda,
  // string msg
  Error{msg: String},
  ByeBye{msg: String},
  // who?
  DiceSonBuenas{autor: String},
  CantarFlor{autor: String},
  CantarContraFlor{autor: String},
  CantarContraFlorAlResto{autor: String},
  TocarEnvido{autor: String},
  TocarRealEnvido{autor: String},
  TocarFaltaEnvido{autor: String},
  GritarTruco{autor: String},
  GritarReTruco{autor: String},
  GritarVale4{autor: String},
  NoQuiero{autor: String},
  ConFlorMeAchico{autor: String},
  QuieroTruco{autor: String},
  QuieroEnvite{autor: String},
  Mazo{autor: String},
  ElEnvidoEstaPrimero{autor: String},
  Abandono{autor: String},
  // (pos:int)
  SigTurno{pos: usize},
  SigTurnoPosMano{pos: usize},
  // (autor:string, valor:int)
  DiceTengo{autor: String, valor: usize},
  DiceSonMejores{autor: String, valor: usize},
  ManoGanada{autor: String, valor: usize},
  // (autor:string, razon:string)
  RondaGanada{autor: String, razon: Razon},
  // (partida:partida)
  NuevaPartida,
  NuevaRonda,
  // (autor:string, palo:palo, valor:valor)
  TirarCarta{autor: String, palo: String, valor: usize},
  // (autor:string, razon:string, pts:int)
  SumaPts{autor:String, razon:Razon, pts:usize},
}

impl Content {
  fn cod(&self) -> String {
    match self {
      Content::LaManoResultaParda{} => String::from("LaManoResultaParda"),
      Content::Error{msg: _} => String::from("Error"),
      Content::ByeBye{msg: _} => String::from("ByeBye"),
      Content::DiceSonBuenas{autor: _} => String::from("DiceSonBuenas"),
      Content::CantarFlor{autor: _} => String::from("CantarFlor"),
      Content::CantarContraFlor{autor: _} => String::from("CantarContraFlor"),
      Content::CantarContraFlorAlResto{autor: _} => String::from("CantarContraFlorAlResto"),
      Content::TocarEnvido{autor: _} => String::from("TocarEnvido"),
      Content::TocarRealEnvido{autor: _} => String::from("TocarRealEnvido"),
      Content::TocarFaltaEnvido{autor: _} => String::from("TocarFaltaEnvido"),
      Content::GritarTruco{autor: _} => String::from("GritarTruco"),
      Content::GritarReTruco{autor: _} => String::from("GritarReTruco"),
      Content::GritarVale4{autor: _} => String::from("GritarVale4"),
      Content::NoQuiero{autor: _} => String::from("NoQuiero"),
      Content::ConFlorMeAchico{autor: _} => String::from("ConFlorMeAchico"),
      Content::QuieroTruco{autor: _} => String::from("QuieroTruco"),
      Content::QuieroEnvite{autor: _} => String::from("QuieroEnvite"),
      Content::Mazo{autor: _} => String::from("Mazo"),
      Content::ElEnvidoEstaPrimero{autor: _} => String::from("ElEnvidoEstaPrimero"),
      Content::Abandono{autor: _} => String::from("Abandono"),
      Content::SigTurno{pos: _} => String::from("SigTurno"),
      Content::SigTurnoPosMano{pos: _} => String::from("SigTurnoPosMano"),
      Content::DiceTengo{autor: _, valor: _} => String::from("DiceTengo"),
      Content::DiceSonMejores{autor: _, valor: _} => String::from("DiceSonMejores"),
      Content::ManoGanada{autor: _, valor: _} => String::from("ManoGanada"),
      Content::RondaGanada{autor: _, razon: _} => String::from("RondaGanada"),
      Content::NuevaPartida{} => String::from("NuevaPartida"),
      Content::NuevaRonda{} => String::from("NuevaRonda"),
      Content::TirarCarta{autor: _, palo: _, valor: _} => String::from("TirarCarta"),
      Content::SumaPts{autor:_, razon:_, pts:_} => String::from("SumaPts"),
    }
  }
}

use serde::ser::{Serializer, SerializeStruct};

#[derive(Debug)]
pub struct Message(pub Content);

impl Serialize for Message {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    let mut is_simple = false; 
    if let Content::LaManoResultaParda = self.0 {
      is_simple = true;
    };

    let num_fields = if is_simple {1} else {2};

    // 3 is the number of fields in the struct.
    let mut state = 
      serializer.serialize_struct("Message", num_fields)?;
    state.serialize_field("cod", &self.0.cod())?;
    if !is_simple {
      state.serialize_field("cont", &self.0)?;
    }
    state.end()
  }
}