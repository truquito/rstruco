use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use serde::{Deserialize, Serialize};

use crate::get_cartas_random;
use crate::mano::{NumMano};
use crate::jugador::{Jugador};
use crate::equipo::{Equipo};
use crate::envite::{Envite, EstadoEnvite};
use crate::truco::{Truco, EstadoTruco};
use crate::manojo::{Manojo};
use crate::carta::{Carta};

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
  pub fn new(azules: Vec<String>, rojos: Vec<String>) -> Result<Ronda, &'static str> {
    let cant_jugadores_por_equipo = azules.len();

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
    
    let sin_cantar = con_flor.clone();     

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
        envite: Envite {
          estado: EstadoEnvite::NoCantadoAun,
          puntaje: 0,
          cantado_por: String::from(""),
          jugadores_con_flor: con_flor,
          sin_cantar: sin_cantar,
        },
        truco: Truco{
          cantado_por: String::from(""),
          estado: EstadoTruco::NoCantado,
        },
      }
    )
  }
}