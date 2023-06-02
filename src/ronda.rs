use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use serde::{Deserialize, Serialize};

use crate::enco;
use crate::mano::{NumMano, Mano, Resultado};
use crate::jugador::{Jugador};
use crate::equipo::{Equipo};
use crate::envite::{Envite};
use crate::truco::{Truco};
use crate::manojo::{Manojo};
use crate::carta::{Carta, get_cartas_random};

#[derive(Debug, Deserialize, Serialize, Clone)]
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
  
  pub manos: [Mano; 3],
  
  // otros
  #[serde(skip_deserializing, skip_serializing)]
  pub mixs: HashMap<String, usize>,
}

// cambio de variable
fn cv(x: usize, mano: usize, cant_jugadores: usize) -> usize {
  if x >= mano {
    x - mano
  } else {
    x + (cant_jugadores - mano)
  }
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
        manos: Default::default(),
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

  pub fn get_la_flor_mas_alta(&self) -> &Manojo {
    self.manojos
      .iter()
      .map(|m| (m, m.tiene_flor(&self.muestra).1))
      // .collect::<Vec<(&Manojo, isize)>>()
      // .into_iter()
      .max_by(|a, b| a.1.cmp(&b.1))
      .unwrap()
      .0
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

  pub fn get_el_mano(&self) -> &Manojo {
    &self.manojos[self.el_mano]
  }

  pub fn get_sig_el_mano(&self) -> usize {
    let m = self.get_el_mano();
    let s = self.get_siguiente(m);
    self.mixs[&s.jugador.id]
  }

  pub fn get_el_turno(&self) -> &Manojo {
    &self.manojos[self.turno]
  }

  pub fn get_mano_anterior(&self) -> &Mano {
    &self.manos[self.mano_en_juego as usize - 1]
  }

  pub fn get_mano_actual(&mut self) -> &mut Mano {
    &mut self.manos[self.mano_en_juego as usize]
  }

  pub fn get_sig(&self, j:usize) -> usize {
    let cant_jugadores = self.manojos.len();
    let es_el_ultimo = j == cant_jugadores - 1;
    if es_el_ultimo {0} else {j + 1}
  }

  pub fn get_siguiente(&self, m: &Manojo) -> &Manojo {
    let idx = self.mixs[&m.jugador.id];
    let cant_jugadores = self.manojos.len();
    let es_el_ultimo = idx == cant_jugadores - 1;
    match es_el_ultimo {
      true => &self.manojos[0],
      false => &self.manojos[idx+1],
    }
  }

  pub fn get_siguiente_habilitado<'a>(&'a self, m: &'a Manojo) -> Option<&Manojo> {
    let mut sig = m;
    let n = self.manojos.len();
    for _ in 0..n {
      sig = self.get_siguiente(sig);
      let no_se_fue_al_mazo = !sig.se_fue_al_mazo;
      let ya_tiro_carta_en_esta_mano = sig.ya_tiro_carta(self.mano_en_juego);
      let no_es_el = sig.jugador.id != m.jugador.id;
      let ok = no_se_fue_al_mazo && (!ya_tiro_carta_en_esta_mano) && no_es_el;
      if ok {
        break
      }
    }
    if sig.jugador.id != m.jugador.id {Some(sig)} else {None}
  }

  pub fn manojo(&self, jid: &str) -> &Manojo {
    &self.manojos[self.mixs[jid]]
  }

  /* PREDICADOS */

  pub fn le_gana_de_mano(&self, i: usize, j: usize) -> bool {
    let cant_jugadores = self.manojos.len();
    let p = cv(i, self.el_mano, cant_jugadores);
	  let q = cv(j, self.el_mano, cant_jugadores);
    p < q
  }

  pub fn hay_equipo_sin_cantar(&self, equipo: Equipo) -> bool {
    self.envite.sin_cantar
      .iter()
      .any(|jid| self.manojo(jid).jugador.equipo == equipo)
  }

  // setters

  pub fn set_next_turno(&mut self) {
    let manojo_turno_actual = &self.manojos[self.turno];
    let manojo_sig_turno = self.get_siguiente_habilitado(manojo_turno_actual).unwrap();
    self.turno = self.mixs[&manojo_sig_turno.jugador.id]
  }

  pub fn set_next_turno_pos_mano(&mut self) {

    let sanity_check = |r: &mut Ronda| {
      let candidato = &r.manojos[r.turno];
      if candidato.se_fue_al_mazo {
        let n = r.manojos.len();
        let start_from = r.el_mano;
        for i in 0..n {
          let ix = (start_from + i) % n;
          let m = &r.manojos[ix];
          let mismo_equipo = m.jugador.equipo == candidato.jugador.equipo;
          if mismo_equipo && !m.se_fue_al_mazo {
            r.turno = r.mixs[&m.jugador.id];
            break
          }
        }
      }
    };

    if self.mano_en_juego == NumMano::Primera {
      self.turno = self.el_mano;
      sanity_check(self);
    } else {
      if self.get_mano_anterior().resultado != Resultado::Empardada {
        let ganador_anterior = &self.get_mano_anterior().ganador;
        self.turno = self.mixs[&self.manojo(ganador_anterior).jugador.id];
        sanity_check(self);
      } else {
        let max_tirada = 
          self.get_mano_anterior().cartas_tiradas
            .iter()
            .map(|t| (t, t.carta.calc_poder(&self.muestra)))
            .max_by(|a, b| a.1.cmp(&b.1))
            .unwrap()
            .0;
        if !self.manojo(&max_tirada.jugador).se_fue_al_mazo {
          self.turno = self.mixs[&max_tirada.jugador];
          sanity_check(self);
          return;
        }

        let m = self.get_siguiente_habilitado(self.get_el_mano()).unwrap();
        self.turno = self.mixs[&m.jugador.id];
        sanity_check(self);
      }
    }
  }

  pub fn set_manojos(&mut self, manojos: Vec<Manojo>) {
    self.manojos = manojos;
    self.cachear_flores(true);
  }

  pub fn set_muestra(&mut self, muestra: Carta) {
    self.muestra = muestra;
    self.cachear_flores(true);
  }

  pub fn exec_el_envido(&mut self, verbose: bool) -> (usize, usize, Vec<enco::Packet>) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let cant_jugadores = self.manojos.len();
    let envidos =
      self.manojos
        .iter()
        .map(|m| m.calcular_envido(&self.muestra))
        .collect::<Vec<_>>();
    let mut ya_dijeron = vec![false; cant_jugadores];
    let mut jidx = self.el_mano;
    while self.manojos[jidx].se_fue_al_mazo {
      jidx = (jidx + 1) % cant_jugadores
    }
    ya_dijeron[jidx] = true;

    if verbose {
      pkts.push(enco::Packet{
        destination: vec![String::from("ALL")],
        message: enco::Message(
          enco::Content::DiceTengo {
            autor: self.manojos[jidx].jugador.id.clone(),
            valor: envidos[jidx]
          }
        )
      });
    }

    let mut todavia_no_dijeron_son_mejores = true;
    let mut i = if self.el_mano != cant_jugadores - 1 {self.el_mano + 1}  else {0};

    while i != self.el_mano {
      let se_fue_al_mazo = self.manojos[i].se_fue_al_mazo;
      let todavia_es_tenido_en_cuenta = (!ya_dijeron[i]) && (!se_fue_al_mazo);
      if todavia_es_tenido_en_cuenta {
        let es_de_equipo_contrario: bool = self.manojos[i].jugador.equipo != self.manojos[jidx].jugador.equipo;
        let tiene_envido_mas_alto: bool = envidos[i] > envidos[jidx];
        let tiene_envido_igual: bool = envidos[i] == envidos[jidx];
        let le_gana_de_mano: bool = self.le_gana_de_mano(i, jidx);
        let son_mejores: bool = tiene_envido_mas_alto || (tiene_envido_igual && le_gana_de_mano);

        if son_mejores {
          if es_de_equipo_contrario {
            if verbose {
              pkts.push(enco::Packet{
                destination: vec![String::from("ALL")],
                message: enco::Message(
                  enco::Content::DiceSonMejores {
                    autor: self.manojos[i].jugador.id.clone(),
                    valor: envidos[i]
                  }
                )
              });
              jidx = i;
              ya_dijeron[i] = true;
              todavia_no_dijeron_son_mejores = false;
              // se "resetea" el bucle
              i = self.get_sig(self.el_mano);
            }
          } else {
            i = self.get_sig(i);
          }
        } else {
          if es_de_equipo_contrario {
            if todavia_no_dijeron_son_mejores {
              if verbose {
                pkts.push(enco::Packet{
                  destination: vec![String::from("ALL")],
                  message: enco::Message(
                    enco::Content::DiceSonBuenas {
                      autor: self.manojos[i].jugador.id.clone()
                    }
                  )
                });
              }
              ya_dijeron[i] = true;
            }
            i = self.get_sig(i);
          } else {
            ya_dijeron[i] = true;
            i = self.get_sig(i);
          }
        }

      } else {
        i = self.get_sig(i);
      }
    }

    let max_envido = envidos[jidx];
    return (jidx, max_envido, pkts);
  }

  pub fn exec_las_flores(&mut self, a_partir_de:usize, verbose: bool) -> 
    (&Manojo, usize, Vec<enco::Packet>) {
    
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let equipo = self.manojo(&self.envite.jugadores_con_flor[0]).jugador.equipo.clone();
    let solo_un_equipo_tiene_flores = 
      self.envite.jugadores_con_flor[1..]
      .iter()
      .map(|jid| self.manojo(jid))
      .any(|m| m.jugador.equipo != equipo);
    if solo_un_equipo_tiene_flores {
      return (&self.manojo(&self.envite.jugadores_con_flor[0]), 0, pkts);
    }

    let flores = 
      self.manojos
        .iter()
        .map(|m| m.calc_flor(&self.muestra))
        .collect::<Vec<_>>();
    
    let mut ya_dijeron =
      self.manojos
        .iter()
        .enumerate()
        .map(|(i,m)| !(flores[i]>0 && !m.se_fue_al_mazo))
        .collect::<Vec<_>>();

    if flores[a_partir_de] > 0 {
      ya_dijeron[a_partir_de] = true;
      if verbose {
        pkts.push(enco::Packet{
          destination: vec![String::from("ALL")],
          message: enco::Message(
            enco::Content::DiceTengo {
              autor: self.manojos[a_partir_de].jugador.id.clone(),
              valor: flores[a_partir_de] as usize
            }
          )
        });
      }
    }

    let mut todavia_no_dijeron_son_mejores = true;
    let mut jidx = a_partir_de;
    let mut i = self.get_sig(a_partir_de);

    while i != a_partir_de {
      let todavia_es_tenido_en_cuenta = !ya_dijeron[i];
      if todavia_es_tenido_en_cuenta {

        let es_de_equipo_contrario = self.manojos[i].jugador.equipo != self.manojos[jidx].jugador.equipo;
        let tiene_envido_mas_alto = flores[i] > flores[jidx];
        let tiene_envido_igual = flores[i] == flores[jidx];
        let le_gana_de_mano = self.le_gana_de_mano(i, jidx);
        let son_mejores = tiene_envido_mas_alto || (tiene_envido_igual && le_gana_de_mano);

        if son_mejores {
          if es_de_equipo_contrario {
            if verbose {
              pkts.push(enco::Packet{
                destination: vec![String::from("ALL")],
                message: enco::Message(
                  enco::Content::DiceSonMejores {
                    autor: self.manojos[i].jugador.id.clone(),
                    valor: flores[a_partir_de] as usize
                  }
                )
              });
            }
            
            jidx = i;
            ya_dijeron[i] = true;
            todavia_no_dijeron_son_mejores = false;
            // se "resetea" el bucle
            i = self.get_sig(a_partir_de);

          } else {
            i = self.get_sig(i);
          }
        } else {
          if es_de_equipo_contrario {
            if todavia_no_dijeron_son_mejores {
              if verbose {
                pkts.push(enco::Packet{
                  destination: vec![String::from("ALL")],
                  message: enco::Message(
                    enco::Content::DiceSonBuenas {
                      autor: self.manojos[i].jugador.id.clone()
                    }
                  )
                });
              }
              ya_dijeron[i] = true;
            }
            i = self.get_sig(i);
          } else {
            ya_dijeron[i] = true;
            i = self.get_sig(i);
          }
        }

      } else {
        i = self.get_sig(i);
      }
    }

    let max_flor = flores[jidx];
    return (&self.manojos[jidx], max_flor as usize, pkts);
  }  

}