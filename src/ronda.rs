use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use serde::{Deserialize, Serialize};

use crate::mano::{NumMano};
use crate::jugador::{Jugador};
use crate::equipo::{Equipo};
use crate::envite::{Envite};
use crate::truco::{Truco};
use crate::manojo::{Manojo};
use crate::carta::{Carta, get_cartas_random};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
// pub struct Ronda<'a> { // <- si se usa `jugadores_con_flor` con referencias
  pub struct Ronda {
  pub mano_en_juego: NumMano,
  pub cant_jugadores_en_juego: HashMap<Equipo, usize>,
  // Indices
	pub el_mano: usize,
	pub turno:  usize,
  // gritos y toques/cantos
  // pub envite: Envite<'a>,
  pub envite: Envite,
	pub truco: Truco,

  // cartas
  pub manojos: Vec<Manojo>,
  pub muestra: Carta,
  
  // pub manos: [Mano; 3],
  
  // otros
  #[serde(skip_deserializing, skip_serializing)]
  pub mixs: HashMap<String, usize>,
}



// impl<'a> Ronda<'a> { // <- si se usa `jugadores_con_flor` con referencias
impl Ronda {

  fn check_inputs(azules: &[String], rojos: &[String]) -> Result<(), &'static str> {
    // checkeo que no hayan repetidos
    let uniques = [&azules[..], &rojos[..]]
      .concat()
      .iter()
      .map(|x| x.as_str())
      .collect::<HashSet<&str>>()
      .len();

    let ok = azules.len() + rojos.len() == uniques;
    if !ok {
      return Err("Hay repetidos");
    }

    // checkeo que la cantidad de jugadores sea correcta
    let ok = [2,4,6].contains(&uniques) && azules.len() == rojos.len();
    if !ok {
      return Err("La cantidad de jugadores es inválida");
    }

    Ok(())
  }

  pub fn new(azules: Vec<String>, rojos: Vec<String>) -> Result<Ronda, &'static str> {
    if let Err(msg) = Ronda::check_inputs(&azules[..], &rojos[..]) {
      return Err(msg);
    }
    
    let cant_jugadores_por_equipo = azules.len();

    // paso a crear los jugadores + manojos(+cartas)
    let mut cartas = get_cartas_random(cant_jugadores_por_equipo * 2 * 3 + 1);
    let muestra = cartas.pop().unwrap();

    let jugadores = 
      azules
        .into_iter()
        .zip(rojos)
        .flat_map(|(x, y)| iter::once(x).chain(iter::once(y)))
        .enumerate()
        .map(|(ix, jid)| Jugador{
          id: jid,
          equipo: if ix%2 == 0 {Equipo::Azul} else {Equipo::Rojo}  
        })
        .collect::<Vec<_>>();
    
    let mixs: HashMap<String, usize> =
      jugadores
        .iter()
        .enumerate()
        .map(|(ix,j)| (j.id.clone(), ix))
        .collect();

    let manojos = 
      jugadores
        .into_iter()
        .map(|j| Manojo::new(
          j,
          [
            cartas.pop().unwrap(),
            cartas.pop().unwrap(),
            cartas.pop().unwrap(),
          ]
        ))
        .collect::<Vec<Manojo>>();
    
    let con_flor = 
      manojos
        .iter()
        .filter(|m| m.tiene_flor(&muestra).0)
        .map(|m| m.jugador.id.clone())
        .collect::<Vec<String>>();
    
    Ok(
      Ronda{
        mano_en_juego: NumMano::Primera,
        cant_jugadores_en_juego: HashMap::from([
          (Equipo::Azul, cant_jugadores_por_equipo),
          (Equipo::Rojo, cant_jugadores_por_equipo)
        ]),
        el_mano: 0,
        turno:  0,
        manojos: manojos,
        muestra: muestra,
        mixs: mixs,
        envite: Envite::new(con_flor),
        truco: Truco::new(),
      }
    )
  }

  pub fn indexar_manojos(&mut self) {
    self.mixs =
      self.manojos
        .iter()
        .map(|m| &m.jugador)
        .enumerate()
        .map(|(ix,j)| (j.id.clone(), ix))
        .collect();
  }

  pub fn get_flores(&self) -> (bool, Vec<&Manojo>) {
    let manojos_con_flor = 
      self.manojos
        .iter()
        .filter(|m| m.tiene_flor(&self.muestra).0)
        .collect::<Vec<&Manojo>>();

    (manojos_con_flor.len() > 0, manojos_con_flor)
  }

  pub fn cachear_flores(&mut self, reset: bool) {
    self.envite.jugadores_con_flor =
      self
        .get_flores()
        .1
        .iter()
        .map(|m| m.jugador.id.clone())
        .collect::<Vec<String>>();
    if reset {
      self.envite.sin_cantar = self.envite.jugadores_con_flor.clone();
    }
  }

  // reparte 3 cartas a cada jugador
  // ademas reparte una muestra
  // resetea las `tiradas` y el `se_fue_al_mazo` de cada manojo
  pub fn repartir_cartas(&mut self) {
    let cant_jugadores = self.manojos.len();
    let num_jugs_por_equipo = cant_jugadores / 2;
    let mut cartas = get_cartas_random(num_jugs_por_equipo * 2 * 3 + 1);
    self.muestra = cartas.pop().unwrap();
    for m in &mut self.manojos {
      m.se_fue_al_mazo = false;
      // m.tiradas = [false;3];
      m.tiradas.iter_mut().for_each(|i| *i = false);
      for i in 0..3 {
        m.cartas[i] = cartas.pop().unwrap()
      }
    }
  }

  // el "reset" de `ronda`
  pub fn nueva_ronda(&mut self, el_mano: usize) {
    let cant_jugadores = self.manojos.len();
    let num_jugs_por_equipo = cant_jugadores / 2;

    self.mano_en_juego = NumMano::Primera;
    self.cant_jugadores_en_juego.insert(Equipo::Azul, num_jugs_por_equipo);
    self.cant_jugadores_en_juego.insert(Equipo::Rojo, num_jugs_por_equipo);
    self.el_mano = el_mano;
    self.turno = el_mano;
    self.repartir_cartas();
    self.envite.reset();
    self.cachear_flores(true);
    self.truco.reset();
    // self.manos = make([]Mano, 3)
  }
}