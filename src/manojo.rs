use std::cmp::max;
use serde::{Deserialize, Serialize};
use crate::carta::*;
use crate::jugador::*;
use crate::mano::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Manojo {
  pub se_fue_al_mazo: bool,
  pub cartas:         [Carta; 3],
  pub tiradas:        [bool; 3],
  pub ultima_tirada:  isize,
  pub jugador:        Jugador,
}

impl Manojo {
  pub fn new(jugador: Jugador, cartas: [Carta; 3]) -> Manojo {
    Manojo{
      jugador: jugador,
      se_fue_al_mazo: false,
      ultima_tirada: -1,
      tiradas: [false; 3],
      cartas: cartas
    }
  }

  pub fn get_cant_cartas_tiradas(&self) -> usize {
    self.tiradas.into_iter().filter(|b| *b).count()
  }

  pub fn ya_tiro_carta(&self, mano:NumMano) -> bool {
    let cant_tiradas = self.get_cant_cartas_tiradas();
    match mano {
        NumMano::Primera => cant_tiradas == 1,
        NumMano::Segunda => cant_tiradas == 2,
        NumMano::Tercera => cant_tiradas == 3,
    }
  }

  pub fn get_carta_idx(&self, carta: &Carta) -> usize {
    self.cartas.iter().position(|c| c == carta).unwrap()
  }

  pub fn tiene_flor(&self, muestra: &Carta) -> (bool,isize) {
    // caso 1: al menos dos piezas
    let mut num_piezas = 0;
    let mut pieza_ix = 0;

    for (ix, c) in self.cartas.iter().enumerate() {
        if c.es_pieza(muestra) {
          num_piezas += 1;
          pieza_ix = ix;
        }
    }

    if num_piezas >= 2 {
      return (true, 1);
    }

    // caso 2: tres cartas del mismo palo
    let todas_mismo_palo = 2 == 
      self.cartas[1..]
        .iter()
        .filter(|c| c.palo == self.cartas[0].palo)
        .count();
    
    if todas_mismo_palo {
      return (true, 2)
    }

    // caso 3: una pieza y dos cartas del mismo palo
    // Y ESAS DOS DIFERENTES DE LA PIEZA (piezaIdx)!
    let tiene_dos_del_mismo_palo = num_piezas > 0 &&
      (self.cartas[0].palo == self.cartas[1].palo && pieza_ix == 2) ||
        (self.cartas[0].palo == self.cartas[2].palo && pieza_ix == 1) ||
        (self.cartas[1].palo == self.cartas[2].palo && pieza_ix == 0);

    if num_piezas == 1 && tiene_dos_del_mismo_palo {
      return (true, 3)
    }

    return (false, -1);
  }

  pub fn calc_flor(&self, muestra: &Carta) -> isize {
    let puntaje_flor: usize ;
    let (tiene_flor, tipo_flor) = self.tiene_flor(muestra);
    if !tiene_flor {
      return -1
    }

    let mut ptjs =
      self.cartas
        .iter()
        .map(|c| c.calc_puntaje(muestra))
        .collect::<Vec<usize>>();
    
    if tipo_flor == 1 {
      let (max_ix, max) = ptjs
        .iter()
        .enumerate()
        .fold((0, 0), |a, c| if c.1 > &a.1 {(c.0, *c.1)} else {a});
      
      // alternativa
      // use std::cmp::max;
      // let _max = max(ptjs[0], max(ptjs[1], ptjs[2]));
      // let max_ix = 
      //   ptjs
      //     .iter()
      //     .enumerate()
      //     .max_by(|(_, &a), (_, &b)| a.cmp(&b))
      //     .map(|(ix, _)| ix)
      //     .unwrap();

        ptjs.remove(max_ix);
        puntaje_flor = max + ptjs.iter().map(|p|p % 10).sum::<usize>();
    } else {
      puntaje_flor = ptjs.iter().sum::<usize>();
    }

    return puntaje_flor as isize;
  }

  // tiene2DelMismoPalo devuelve `true` si tiene dos cartas
  // del mismo palo, y ademas los indices de estas en el array manojo.Cartas
  pub fn tiene_2_del_mismo_palo(&self) -> (bool, (usize, usize)) {
    for i in 0..2 {
      for j in i+1..3 {
        let mismo_palo = self.cartas[i].palo == self.cartas[j].palo; 
        if mismo_palo {
          return (true, (i,j))
        }
      }
    }
    return (false, (0, 0))
  }

  // CalcularEnvido devuelve el puntaje correspondiente al envido del manojo
  // PRE: no tiene flor
  pub fn calcular_envido(&self, muestra: &Carta) -> usize {
    let (tiene_2_del_mismo_palo, ixs) = self.tiene_2_del_mismo_palo();
    if tiene_2_del_mismo_palo {
      let (x,y) = (
        self.cartas[ixs.0].calc_puntaje(&muestra),
        self.cartas[ixs.1].calc_puntaje(&muestra)
      );
      let no_tiene_niguna_pieza = max(x,y) < 27;
      return if no_tiene_niguna_pieza {x + y + 20} else {x + y};
    } else {
      // si no, entonces implemente suma las 2 de mayor valor
      let mut pts: Vec<usize> = 
        self.cartas
          .iter()
          .map(|c| c.calc_puntaje(muestra))
          .collect::<Vec<usize>>();
      pts.sort();
      pts.reverse();
      return pts.iter().take(2).sum();
    }
  }
  
}