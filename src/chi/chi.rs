use rand::Rng;

// use std::fmt;
// use serde::{Deserialize, Serialize};
use crate::{Partida, IJugada, Manojo};
// use crate::{Packet};
use crate::{enco};
use crate::jugada::{*};
use rand::seq::SliceRandom;

pub fn is_done(pkts: &Vec<enco::Packet>) -> bool {
  pkts
    .iter()
    .any(|pkt| match pkt.message.0 {
        crate::Content::NuevaPartida |
        crate::Content::NuevaRonda => true,
        crate::Content::RondaGanada{ autor: _, razon: _ } => true,
        _ => false
    })
}

pub fn random_action_chi(chi: &Vec<Box<dyn IJugada>>) -> usize {
  rand::thread_rng().gen_range(0..chi.len())
}

// en el primer parametro de salida retorna un indice de manojo random
// en el segundo retorna una jugada random de este manojo
pub fn random_action_chis(chis: &Vec<Vec<Box<dyn IJugada>>>) -> (usize,usize) {
  let habilitados: Vec<usize>  =
    chis
      .iter()
      .enumerate()
      .filter(|(_ix,chi)| chi.len() > 0)
      .map(|(ix, _chi)| ix)
      .collect();

  let rmix = *habilitados.choose(&mut rand::thread_rng()).unwrap();
  let raix = rand::thread_rng().gen_range(0..chis[rmix].len());

  (rmix, raix)
}

// Retorna todas las acciones posibles para un jugador `m` dado
pub fn chi(p:&Partida, m:&Manojo, allow_mazo:bool) -> Vec<Box<dyn IJugada>> {
  let mut res: Vec<Box<dyn IJugada>> = vec![
    // cartas
    Box::new(TirarCarta{jid: m.jugador.id.clone(), carta: m.cartas[0].clone()}),
    Box::new(TirarCarta{jid: m.jugador.id.clone(), carta: m.cartas[1].clone()}),
    Box::new(TirarCarta{jid: m.jugador.id.clone(), carta: m.cartas[2].clone()}),
    // toques
    Box::new(TocarEnvido{jid: m.jugador.id.clone()}),
    Box::new(TocarRealEnvido{jid: m.jugador.id.clone()}),
    Box::new(TocarFaltaEnvido{jid: m.jugador.id.clone()}),
    // cantos
    Box::new(CantarFlor{jid: m.jugador.id.clone()}),
    Box::new(CantarContraFlor{jid: m.jugador.id.clone()}),
    Box::new(CantarContraFlorAlResto{jid: m.jugador.id.clone()}),
    // gritos
    Box::new(GritarTruco{jid: m.jugador.id.clone()}),
    Box::new(GritarReTruco{jid: m.jugador.id.clone()}),
    Box::new(GritarVale4{jid: m.jugador.id.clone()}),
    // respuestas
    Box::new(ResponderQuiero{jid: m.jugador.id.clone()}),
    Box::new(ResponderNoQuiero{jid: m.jugador.id.clone()}),
  ];

  res = res.into_iter().filter(|j| j.ok(p).1).collect();

  let mazo = Box::new(IrseAlMazo{jid: m.jugador.id.clone()});
  if allow_mazo && mazo.ok(p).1 {
    res.push(mazo);
  }
 
  res
}

pub fn chis(p:&Partida, allow_mazo:bool) -> Vec<Vec<Box<dyn IJugada>>> {
  p.ronda.manojos
    .iter()
    .map(|m| chi(p, m, allow_mazo))
    .collect()
}

pub fn random_action(p:&Partida, allow_mazo:bool) -> Box<dyn IJugada> {
  let mut chis = chis(p, allow_mazo);
  let (rmix, raix) = random_action_chis(&chis);
  chis.remove(rmix).remove(raix)
}