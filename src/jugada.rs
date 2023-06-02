use std::fmt::Debug;
use crate::partida::{Partida};
use crate::enco;

pub enum IJugadaId {
  JIdTirarCarta = 0,
  JIdEnvido = 1,
  JIdRealEnvido = 2,
  JIdFaltaEnvido = 3,
  JIdFlor = 4,
  JIdContraFlor = 5,
  JIdContraFlorAlResto = 6,
  JIdTruco = 7,
  JIdReTruco = 8,
  JIdVale4 = 9,
  JIdQuiero = 10,
  JIdNoQuiero = 11,
  JIdMazo = 12,
}

pub trait IJugada: Debug {
  fn id() -> IJugadaId;
  fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool);
  fn hacer(&self, p:Partida) -> Vec<enco::Packet>;
}

pub fn jugadear(p: &Partida) -> bool {
  p.puntuacion > 0 
}