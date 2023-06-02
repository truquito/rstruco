use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::equipo::{Equipo};
use crate::{EstadoEnvite, EstadoTruco, Palo};
use crate::ronda::{Ronda};
use crate::enco;
use crate::mano::{NumMano, Resultado};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Partida{
  pub puntuacion: usize,
  pub puntajes: HashMap<Equipo, usize>,
  pub ronda: Ronda,
  #[serde(skip_deserializing, skip_serializing)]
  pub verbose: bool,
}

impl Partida {
  pub fn new(
    puntuacion:usize,
    azules: Vec<String>,
    rojos: Vec<String>,
    verbose: bool,
  ) -> Result<Partida, &'static str> {

    if ![20,30,40].contains(&puntuacion) {
      return Err("la puntuacion de la partida no es valida");
    }

    let misma_cant_de_jugadores = azules.len() == rojos.len();
    let cant_jugadores = azules.len() + rojos.len();
    let cant_correcta = [2,4,6].contains(&cant_jugadores);
    let ok = misma_cant_de_jugadores && cant_correcta;
    if !ok {
      return Err("la cantidad de jguadores no es correcta");
    }

    Ok(Partida{
      puntuacion: puntuacion,
      puntajes: HashMap::from([(Equipo::Azul, 0), (Equipo::Rojo, 0)]),
      verbose: verbose,
      ronda: Ronda::new(azules, rojos).unwrap()
    })
  }

  /* GETTERs */
  pub fn get_max_puntaje(&self) -> usize {
    *self.puntajes
      .iter()
      .max_by_key(|&(_k,v) | v)
      .unwrap()
      .1
  }

  pub fn el_que_va_ganando(&self) -> Equipo {
    let va_ganando_rojo = 
      self.puntajes[&Equipo::Rojo] > self.puntajes[&Equipo::Azul];
    if va_ganando_rojo {Equipo::Rojo} else {Equipo::Azul}
  }

  pub fn get_puntuacion_malas(&self) -> usize{
    self.puntuacion / 2
  }

  pub fn es_mano_a_mano(&self) -> bool{
    self.ronda.manojos.len() == 2
  }

  pub fn terminada(&self) -> bool{
    self.get_max_puntaje() >= self.puntuacion
  }

  pub fn el_chico(&self) -> usize{
    self.puntuacion / 2
  }

  pub fn esta_en_malas(&self, e:Equipo) -> bool{
    self.puntajes[&e] < self.el_chico()
  }

  pub fn calc_pts_falta(&self) -> usize{
    self.puntuacion - self.puntajes[&self.el_que_va_ganando()]
  }

  pub fn calc_pts_falta_envido(&self, ganador_del_envite:Equipo) -> usize{
    if self.esta_en_malas(self.el_que_va_ganando()){
      let lo_que_le_falta_al_ganador_para_ganar_el_chico = 
        self.el_chico() - self.puntajes[&ganador_del_envite];
      return lo_que_le_falta_al_ganador_para_ganar_el_chico;
    }
    let lo_que_le_falta_al_que_va_ganando_para_ganar_el_chico = 
      self.calc_pts_falta();
    lo_que_le_falta_al_que_va_ganando_para_ganar_el_chico
  }

  pub fn calc_pts_contraflor_al_resto(&self, ganador_del_envite:Equipo) -> usize{
    self.calc_pts_falta_envido(ganador_del_envite)
  }

  pub fn suma_puntos(&mut self, e:Equipo, total_pts:usize) -> bool{
    self.puntajes.entry(e).and_modify(|v| *v += total_pts);
    self.terminada()
  }

  pub fn tocar_envido(&mut self, jid: &str) {
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    // 2 opciones: o bien no se jugo aun
	  // o bien ya estabamos en envido
    let ya_se_habia_cantado_el_envido = 
      self.ronda.envite.estado == EstadoEnvite::Envido;
    if ya_se_habia_cantado_el_envido {
      // se aumenta el puntaje del envido en +2
      self.ronda.envite.puntaje += 2;
      self.ronda.envite.cantado_por = m.jugador.id.clone();
    } else { // no se habia jugado aun
      self.ronda.envite.cantado_por = m.jugador.id.clone();
      self.ronda.envite.estado = EstadoEnvite::Envido;
      self.ronda.envite.puntaje = 2;
    }    
  }

  pub fn tocar_real_envido(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.envite.cantado_por = m.jugador.id.clone();
    // 2 opciones:
    // o bien el envido no se jugo aun,
    // o bien ya estabamos en envido
    let no_se_jugo_aun = 
      self.ronda.envite.estado == EstadoEnvite::NoCantadoAun;
    if no_se_jugo_aun { // no se habia jugado aun
      self.ronda.envite.puntaje = 3;
    } else { // ya se habia cantado ENVIDO x cantidad de veces
      self.ronda.envite.puntaje += 3;
    }
    self.ronda.envite.estado = EstadoEnvite::RealEnvido;
  }

  pub fn tocar_falta_envido(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.envite.estado = EstadoEnvite::FaltaEnvido;
    self.ronda.envite.cantado_por = m.jugador.id.clone();
  }

  pub fn cantar_flor(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    let ya_estabamos_en_flor = self.ronda.envite.estado >= EstadoEnvite::Flor;
    if ya_estabamos_en_flor{
      self.ronda.envite.puntaje += 3;
      // si estabamos en algo mas grande que `FLOR` -> no lo aumenta
      if self.ronda.envite.estado == EstadoEnvite::Flor {
        self.ronda.envite.cantado_por = m.jugador.id.clone();
        self.ronda.envite.estado = EstadoEnvite::Flor;
      }
    } else {
      // se usa por si dicen "no quiero" -> se obtiene el equipo
      // al que pertenece el que la canto en un principio para
      // poder sumarle los puntos correspondientes
      self.ronda.envite.puntaje = 3;
      self.ronda.envite.cantado_por = m.jugador.id.clone();
      self.ronda.envite.estado = EstadoEnvite::Flor;
    }
  }

  pub fn cantar_contra_flor(&mut self, jid: &str){
    self.ronda.envite.estado = EstadoEnvite::ContraFlor;
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.envite.cantado_por = m.jugador.id.clone();
    // ahora la flor pasa a jugarse por 4 puntos
    self.ronda.envite.puntaje = 4;
  }

  pub fn cantar_contra_flor_al_resto(&mut self, jid: &str){
    self.ronda.envite.estado = EstadoEnvite::ContraFlorAlResto;
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.envite.cantado_por = m.jugador.id.clone();
    // ahora la flor pasa a jugarse por 4 puntos
    self.ronda.envite.puntaje = 4; // <- eso es al pedo, es independiente
  }

  pub fn gritar_truco(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.truco.cantado_por = m.jugador.id.clone();
    self.ronda.truco.estado = EstadoTruco::Truco;
  }

  pub fn querer_truco(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.truco.cantado_por = m.jugador.id.clone();
    self.ronda.truco.estado = match self.ronda.truco.estado {
      EstadoTruco::Truco => {EstadoTruco::TrucoQuerido},
      EstadoTruco::ReTruco => {EstadoTruco::ReTrucoQuerido}
      EstadoTruco::Vale4 => {EstadoTruco::Vale4Querido}
      _ => unreachable!()
    };
  }

  pub fn gritar_retruco(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.truco.cantado_por = m.jugador.id.clone();
    self.ronda.truco.estado = EstadoTruco::ReTruco;
  }

  pub fn gritar_vale4(&mut self, jid: &str){
    let m = &self.ronda.manojos[self.ronda.mixs[jid]];
    self.ronda.truco.cantado_por = m.jugador.id.clone();
    self.ronda.truco.estado = EstadoTruco::Vale4;
  }

  pub fn ir_al_mazo(&mut self, jid: &str){
    let m = &mut self.ronda.manojos[self.ronda.mixs[jid]];
    m.se_fue_al_mazo = true;
    let equipo_del_jugador = m.jugador.equipo;
    self.ronda.cant_jugadores_en_juego
      .entry(equipo_del_jugador)
      .and_modify(|v| *v -= 1);
    // lo elimino de los jugadores que tenian flor (si es que tenia)
    if self.ronda.envite.sin_cantar.contains(&m.jugador.id) {
      // self.ronda.envite.sin_cantar.remove(m.jugador.id)
      self.ronda.envite.sin_cantar
        .remove(
          self.ronda.envite.sin_cantar
            .iter()
            .position(|x| *x == m.jugador.id)
            .expect("not found")
        );
    }
  }

  // canonical
  // pub fn tirar_carta(&mut self, m: &mut Manojo, idx:usize){
  //   m.tiradas[idx] = true;
  //   m.ultima_tirada = idx as isize;
  //   let carta = m.cartas[idx].clone();
  //   let tirada = CartaTirada{
  //     jugador: m.jugador.id.clone(),
  //     carta: carta
  //   };

  //   self.ronda.get_mano_actual().agregar_tirada(tirada);
  // }

  pub fn tirar_carta(&mut self, jid: &str, idx:usize) {
    let tirada = self.ronda.manojos[self.ronda.mixs[jid]].tirar_carta(idx);
    self.ronda.get_mano_actual().agregar_tirada(tirada);
  }

  pub fn abandono(&mut self, jid:&str) -> Vec<enco::Packet> {
    // encuentra al jugador
    let manojo = self.ronda.manojo(jid);
    // doy por ganador al equipo contrario
    let equipo_contrario = manojo.jugador.equipo.equipo_contrario();
    let pts_faltantes = self.puntuacion - self.puntajes[&equipo_contrario];
    self.suma_puntos(equipo_contrario, pts_faltantes);

    let mut pkts: Vec<enco::Packet> = Vec::new();
    if self.verbose {
      pkts.push(enco::Packet{
        destination: vec![String::from("ALL")],
        message: enco::Message(
          enco::Content::Abandono {
            autor: jid.to_string(),
          }
        )
      });
    }

    pkts
  }

  pub fn evaluar_ronda(&mut self) -> (bool, Vec<enco::Packet>) {
    let mut pkts: Vec<enco::Packet> = Vec::new();

    // A MENOS QUE SE HAYAN IDO TODOS EN LA PRIMERA MANO!!!
    let hay_jugadores_rojo = self.ronda.cant_jugadores_en_juego[&Equipo::Rojo] > 0;
    let hay_jugadores_azul = self.ronda.cant_jugadores_en_juego[&Equipo::Azul] > 0;
    let hay_jugadores_en_ambos = hay_jugadores_rojo && hay_jugadores_azul;
    let primera_mano = self.ronda.mano_en_juego == NumMano::Primera;

    // o bien que en la primera mano hayan cantado truco y uno no lo quizo
    let mano_actual = self.ronda.mano_en_juego as usize;
    let el_truco_no_tuvo_respuesta = self.ronda.truco.estado.es_truco_respondible();
    let no_fue_parda = self.ronda.manos[mano_actual].resultado != Resultado::Empardada;
    let esta_mano_ya_tiene_ganador = no_fue_parda && self.ronda.manos[mano_actual].ganador != "";
    let el_truco_fue_no_querido = el_truco_no_tuvo_respuesta && esta_mano_ya_tiene_ganador;
    let el_truco_fue_querido = !el_truco_fue_no_querido;

    let no_se_acabo = primera_mano && hay_jugadores_en_ambos && el_truco_fue_querido;
    if no_se_acabo {
      return (false, pkts);
    }

    // de aca en mas ya se que hay al menos 2 manos jugadas
    // (excepto el caso en que un equipo haya abandonado)
    // asi que es seguro acceder a los indices 0 y 1 en:
    // self.ronda.manos[0] & self.ronda.manos[1]

    let cant_manos_ganadas: HashMap<Equipo, usize> =
      self.ronda.manos[..mano_actual+1]
        .iter()
        .fold(HashMap::new(), |mut map, c| {
            if c.resultado != Resultado::Empardada {
              let e = self.ronda.manojo(&c.ganador).jugador.equipo;
              *map.entry(e).or_insert(0) += 1;
            }
            map
        });

    let hay_empate = cant_manos_ganadas[&Equipo::Rojo] == cant_manos_ganadas[&Equipo::Azul];
    let parda_primera = self.ronda.manos[0].resultado == Resultado::Empardada;
    let parda_segunda = self.ronda.manos[1].resultado == Resultado::Empardada;
    let parda_tercera = self.ronda.manos[2].resultado == Resultado::Empardada;
    let se_esta_jugando_la_segunda = self.ronda.mano_en_juego == NumMano::Segunda;

    let no_se_acabo_aun = se_esta_jugando_la_segunda && hay_empate &&
      hay_jugadores_en_ambos && !el_truco_fue_no_querido;

    if no_se_acabo_aun {
      return (false, pkts);
    }

    // caso particular:
    // no puedo definir quien gano si la seguna mano no tiene definido un resultado
    let no_esta_empardada = self.ronda.manos[NumMano::Segunda as usize].resultado != Resultado::Empardada;
    let no_tiene_ganador = self.ronda.manos[NumMano::Segunda as usize].ganador == "";
    let segunda_mano_indefinida = no_esta_empardada && no_tiene_ganador;
    // tengo que diferenciar si vengo de: TirarCarta o si vengo de un no quiero:
    // si viniera de un TirarCarta -> en la mano actual (o la anterior)? la ultima carta tirada pertenece al turno actual
    let ix_mano_en_juego = self.ronda.mano_en_juego as usize;
    let n = self.ronda.manos[ix_mano_en_juego].cartas_tiradas.len();
    let actual = self.ronda.get_el_turno().jugador.id.clone();
    let mix = self.ronda.mano_en_juego as usize;
    let ultima_carta_tirada_pertenece_al_turno_actual = n > 0 &&
      self.ronda.manos[mix].cartas_tiradas[n-1].jugador == actual;
    let vengo_de_tirar_carta = ultima_carta_tirada_pertenece_al_turno_actual;

    if segunda_mano_indefinida && hay_jugadores_en_ambos && vengo_de_tirar_carta {
      return (false, pkts);
    }

    // hay ganador -> ya se que al final voy a retornar un true
    let mut ganador = "".to_string();

    if !hay_jugadores_en_ambos {
      // enonces como antes paso por evaluar mano
      // y seteo a ganador de la ultima mano jugada (la "actual")
      // al equipo que no abandono -> lo sacao de ahi
      // caso particular: la mano resulto "empardada pero uno abandono"
      if no_fue_parda && esta_mano_ya_tiene_ganador {
        ganador = self.ronda.get_mano_actual().ganador.clone()
      } else {
        // el ganador es el primer jugador que no se haya ido al mazo del equipo
        // que sigue en pie
        let equipo_ganador = if hay_jugadores_azul {Equipo::Azul} else {Equipo::Rojo};
        for m in &self.ronda.manojos {
          if (!m.se_fue_al_mazo) && m.jugador.equipo == equipo_ganador {
            ganador = m.jugador.id.clone();
            break
          }
        }
      }
    } else if cant_manos_ganadas[&Equipo::Rojo] >= 2 {
      // agarro cualquier manojo de los rojos
      // o bien es la Primera o bien la Segunda
      ganador = 
        match self.ronda.manojo(&self.ronda.manos[0].ganador).jugador.equipo {
          Equipo::Rojo => { self.ronda.manos[0].ganador.clone() },
          Equipo::Azul => { self.ronda.manos[1].ganador.clone() },
        }

    } else if cant_manos_ganadas[&Equipo::Azul] >= 2 {
      ganador = 
        match self.ronda.manojo(&self.ronda.manos[0].ganador).jugador.equipo {
          Equipo::Azul => { self.ronda.manos[0].ganador.clone() },
          Equipo::Rojo => { self.ronda.manos[1].ganador.clone() },
        }
    } else {
        ganador =
          match (parda_primera, parda_segunda, parda_tercera) {
            (true, false, false) => {self.ronda.manos[NumMano::Segunda as usize].ganador.clone()},
            (false, true, false) => {self.ronda.manos[NumMano::Primera as usize].ganador.clone()},
            (false, false, false) => {self.ronda.manos[NumMano::Primera as usize].ganador.clone()},
            (true, true, false) => {self.ronda.manos[NumMano::Tercera as usize].ganador.clone()},
            // (true, true, false) => {self.ronda.get_el_mano().jugador.id.clone()},
            (_, _, _) => unreachable!()
          }
    }

    let total_pts = match self.ronda.truco.estado {
      EstadoTruco::NoCantado | EstadoTruco::Truco => 1,
      EstadoTruco::TrucoQuerido | EstadoTruco::ReTruco => 2,
      EstadoTruco::ReTrucoQuerido | EstadoTruco::Vale4 => 3,
      EstadoTruco::Vale4Querido => 4,
    };

    if hay_jugadores_en_ambos {
      if self.verbose {
        pkts.push(enco::Packet{
          destination: vec![String::from("ALL")],
          message: enco::Message(
            enco::Content::RondaGanada {
              autor: ganador.clone(),
              razon: enco::Razon::SeFueronAlMazo
            }
          )
        });
      }
    } else if el_truco_no_tuvo_respuesta {
      ganador = self.ronda.truco.cantado_por.clone();
      let razon = match self.ronda.truco.estado {
        EstadoTruco::Truco => enco::Razon::TrucoNoQuerido,
        EstadoTruco::ReTruco => enco::Razon::TrucoNoQuerido,
        EstadoTruco::Vale4 => enco::Razon::TrucoNoQuerido,
        _ => unreachable!(),
      };
      if self.verbose {
        pkts.push(enco::Packet{
          destination: vec![String::from("ALL")],
          message: enco::Message(
            enco::Content::RondaGanada {
              autor: ganador.clone(),
              razon: razon
            }
          )
        });
      }
    } else {
      let razon = match self.ronda.truco.estado {
        EstadoTruco::Truco => enco::Razon::TrucoQuerido,
        EstadoTruco::ReTruco => enco::Razon::TrucoQuerido,
        EstadoTruco::Vale4 => enco::Razon::TrucoQuerido,
        _ => unreachable!(),
      };
      if self.verbose {
        pkts.push(enco::Packet{
          destination: vec![String::from("ALL")],
          message: enco::Message(
            enco::Content::RondaGanada {
              autor: ganador.clone(),
              razon: razon
            }
          )
        });
      }
    }

    self.suma_puntos(self.ronda.manojo(&ganador).jugador.equipo, total_pts);

    if self.verbose {
      pkts.push(enco::Packet{
        destination: vec![String::from("ALL")],
        message: enco::Message(
          enco::Content::SumaPts {
            autor: ganador,
            razon: enco::Razon::TrucoQuerido,
            pts: total_pts,
          }
        )
      });
    }

    (true, pkts)
  }

  pub fn evaluar_mano(&mut self) -> (bool, Vec<enco::Packet>) {
    let mut pkts: Vec<enco::Packet> = Vec::new();

    let max_poder: HashMap<Equipo, (usize, String)> = HashMap::from([
      (Equipo::Rojo, (0, String::new())),
      (Equipo::Azul, (0, String::new())),
    ]);

    // let mano = self.ronda.get_mano_actual();
    // let mano = self.ronda.get_mano_actual_nm();
    // self.ronda.manos[self.ronda.mano_en_juego as usize]

    // mano en juego index
    let mej_ix = self.ronda.mano_en_juego as usize;
    
    let max_poder: HashMap<Equipo, (usize, String)> =
      self.ronda.manos[mej_ix].cartas_tiradas
        .iter()
        .fold(max_poder, |mut map, c| {
            let e = self.ronda.manojo(&c.jugador).jugador.equipo;
            let p = c.carta.calc_poder(&self.ronda.muestra);
            let x = map.entry(e).or_default();
            if p > x.0 {
              *x = (p,c.jugador.clone())
            }
            map
        });

    let es_parda = 
      max_poder.get(&Equipo::Rojo) == max_poder.get(&Equipo::Azul);

    let no_se_llego_a_tirar_ninguna_carta = 
      self.ronda.manos[mej_ix].cartas_tiradas.len() == 0;
    let se_fueron_todos = 
      self.ronda.cant_jugadores_en_juego[&Equipo::Rojo] == 0 ||
      self.ronda.cant_jugadores_en_juego[&Equipo::Azul] == 0;

      if no_se_llego_a_tirar_ninguna_carta || se_fueron_todos {
        let equipo_ganador: Equipo;
        let quedan_jugadores_del_rojo = 
          self.ronda.cant_jugadores_en_juego[&Equipo::Rojo] > 0;
        if quedan_jugadores_del_rojo {
          equipo_ganador = Equipo::Rojo;
          self.ronda.get_mano_actual().resultado = Resultado::GanoRojo;
        } else {
          equipo_ganador = Equipo::Azul;
          self.ronda.get_mano_actual().resultado = Resultado::GanoAzul;
        }
        if self.ronda.manojos[0].jugador.equipo == equipo_ganador {
          self.ronda.manos[mej_ix].ganador = 
            self.ronda.manojos[0].jugador.id.clone();
        } else {
          self.ronda.manos[mej_ix].ganador = 
            self.ronda.manojos[1].jugador.id.clone();
        }
      } else if es_parda {
        self.ronda.manos[mej_ix].resultado = Resultado::Empardada;
        self.ronda.manos[mej_ix].ganador = String::from("");
        if self.verbose {
          pkts.push(enco::Packet{
            destination: vec![String::from("ALL")],
            message: enco::Message(
              enco::Content::LaManoResultaParda {}
            )
          });
        }
      } else {
        if max_poder[&Equipo::Rojo].0 > max_poder[&Equipo::Azul].0 {
          self.ronda.manos[mej_ix].ganador = max_poder[&Equipo::Rojo].1.clone();
          self.ronda.manos[mej_ix].resultado = Resultado::GanoRojo;
        } else {
          self.ronda.manos[mej_ix].ganador = max_poder[&Equipo::Azul].1.clone();
          self.ronda.manos[mej_ix].resultado = Resultado::GanoAzul;
        }
        if self.verbose {
          pkts.push(enco::Packet{
            destination: vec![String::from("ALL")],
            message: enco::Message(
              enco::Content::ManoGanada{
                autor: self.ronda.manos[mej_ix].ganador.clone(),
                valor: self.ronda.mano_en_juego as usize
              }
            )
          });
        }
      }

      let (empieza_nueva_ronda, mut pkt2) = self.evaluar_ronda();

      // fn append(&mut self, other: &mut Vec<T>)
      // Moves all the elements of `other` into `Self`, leaving `other` empty.
      // .extend() copies elements of `other`
      pkts.append(&mut pkt2);

      (empieza_nueva_ronda, pkts)
  }

  pub fn perspectiva(&mut self, jid:&str) -> Partida {
    let mut copia = self.clone();
    for mix in 0..copia.ronda.manojos.len() {
      let m = &copia.ronda.manojos[mix];
      let hay_que_censurar = 
        m.jugador.equipo != self.ronda.manojo(jid).jugador.equipo;
      if hay_que_censurar {
        for cix in 0..m.cartas.len() {
          if !copia.ronda.manojos[mix].tiradas[cix] {
            copia.ronda.manojos[mix].cartas[cix].valor = 0;
            copia.ronda.manojos[mix].cartas[cix].palo = Palo::Copa;
          }
        }
      }
    }
    copia
  }

  pub fn bye_bye(&self) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    if self.terminada() {
      if self.verbose {
        pkts.push(enco::Packet{
          destination: vec![String::from("ALL")],
          message: enco::Message(
            enco::Content::ByeBye {
              msg: self.el_que_va_ganando().to_string(),
            }
          )
        });
      }
    }
    pkts
  }


}