use std::fmt::Debug;
use crate::partida::{Partida};
use crate::{enco, EstadoEnvite, NumMano, EstadoTruco, Resultado};
use crate::carta::{Carta};
use crate::equipo::{Equipo};

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
  fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet>;
}

pub fn jugadear(p: &Partida) -> bool {
  p.puntuacion > 0 
}

pub struct TirarCarta {
  pub jid: String,
	pub carta: Carta
}

impl TirarCarta {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdTirarCarta
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();

    // checkeo si se fue al mazo
    let no_se_fue_al_mazo = !p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let ok = no_se_fue_al_mazo;

    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tirar una carta porque ya te fuiste al mazo".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    // esto es un tanto redundante porque es imposible que no sea su turno
    // (checkeado mas adelante) y que al mismo tiempo tenga algo para tirar
    // luego de haber jugado sus 3 cartas; aun asi lo dejo
    let ya_tiro_todas_sus_cartas = 
      p.ronda.manojo(&self.jid).get_cant_cartas_tiradas() == 3;
    if ya_tiro_todas_sus_cartas {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tirar una carta porque ya las tiraste todas".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    // checkeo flor en juego
    let envite_en_juego = p.ronda.envite.estado >= EstadoEnvite::Envido;
    if envite_en_juego {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tirar una carta ahora porque el envite esta en juego".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    // primero que nada: tiene esa carta?
    // pide por un error, pero p.Manojo NO retorna error alguno !
    let idx = p.ronda.manojo(&self.jid).get_carta_idx(&self.carta);
    // todo: checkear que tiene esa carta
    // if p.verbose {
    //   pkts.push(enco::Packet{
    //     destination: vec![self.jid.clone()],
    //     message: enco::Message(
    //       enco::Content::Error {
    //         msg: err.Error(),
    //       }
    //     )
    //   });
    // }
    // return (pkts, false);
    
    // ya jugo esa carta?
    let todavia_no_la_tiro = !p.ronda.manojo(&self.jid).tiradas[idx];
    if !todavia_no_la_tiro {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "Ya tiraste esa carta".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    // luego, era su turno?
    let era_su_turno = 
      p.ronda.get_el_turno().jugador.id == self.jid;
    if !era_su_turno {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No era su turno, no puede tirar la carta".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    // checkeo si tiene flor
    let flor_habilitada = (p.ronda.envite.estado >= EstadoEnvite::NoCantadoAun && p.ronda.envite.estado <= EstadoEnvite::Flor) && p.ronda.mano_en_juego == NumMano::Primera;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&self.jid);
    let no_puede_tirar = flor_habilitada && tiene_flor && no_canto_flor_aun;
    if no_puede_tirar {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tirar una carta sin antes cantar la flor".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    // cambio: ahora no puede tirar carta si el grito truco
    let truco_gritado = p.ronda.truco.estado.es_truco_respondible();
    let uno_del_equipo_contrario_grito_truco = truco_gritado && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let yo_gite_el_truco = truco_gritado && self.jid == p.ronda.truco.cantado_por;
    let el_truco_es_respondible = truco_gritado && uno_del_equipo_contrario_grito_truco && !yo_gite_el_truco;
    if el_truco_es_respondible {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tirar una carta porque tu equipo debe responder la propuesta del truco".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    // ok
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    // ok la tiene y era su turno -> la juega
    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::TirarCarta {
            autor: self.jid.clone(),
            palo: self.carta.palo.to_string(),
            valor: self.carta.valor,
          }
        )
      });
    }

    let idx = p.ronda.manojo(&self.jid).get_carta_idx(&self.carta);
    p.tirar_carta(&self.jid, idx);

    // era el ultimo en tirar de esta mano?
	let era_el_ultimo_en_tirar = p.ronda.get_siguiente_habilitado(p.ronda.manojo(&self.jid)).is_none();
	if era_el_ultimo_en_tirar {
		// de ser asi tengo que checkear el resultado de la mano
		let (empieza_nueva_ronda, mut res) = p.evaluar_mano();
    pkts.append(&mut res);

		if !empieza_nueva_ronda {
			let se_termino_la_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
			let nadie_canto_envite = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun;
			if se_termino_la_primera_mano && nadie_canto_envite {
				p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
				p.ronda.envite.sin_cantar = Vec::new();
			}
			// actualizo el mano
			p.ronda.mano_en_juego.inc();
			// p.ronda.SetNextTurnoPosMano();
			p.ronda.set_next_turno_pos_mano();
      // lo envio
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::SigTurnoPosMano {
              pos: p.ronda.turno,
            }
          )
        });
      }
		} else {

			if !p.terminada() {
				// ahora se deberia de incrementar el mano
				// y ser el turno de este
				let sig_mano = p.ronda.get_sig_el_mano();
				p.ronda.nueva_ronda(sig_mano); // todo: el tema es que cuando llama aca
				// no manda mensaje de que arranco nueva ronda
				// falso: el padre que llama a .EvaluarRonda tiene que fijarse si
				// retorno true
				// entonces debe crearla el
				// no es responsabilidad de EvaluarRonda arrancar una ronda nueva!!
				// de hecho, si una ronda es terminable y se llama 2 veces consecutivas
				// al mismo metodo booleano, en ambas oportunidades retorna diferente
				// ridiculo

        if p.verbose {
          pkts.append(
            &mut p.ronda.manojos
              .iter()
              .map(|m| 
                enco::Packet{
                  destination: vec![m.jugador.id.clone()],
                  message: enco::Message(
                    enco::Content::NuevaRonda {
                      // todo: aca va una perspctiva de p segun m
                      // todo!()
                    }
                  )
                }
              )
              .collect::<Vec<enco::Packet>>()
          )
        }
			}

		}

		// el turno del siguiente queda dado por el ganador de esta
	} else {
		p.ronda.set_next_turno();
    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::SigTurno {
            pos: p.ronda.turno,
          }
        )
      });
    }
	}

    pkts
  }
}

pub struct TocarEnvido {
  pub jid: String,
}
impl TocarEnvido {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdEnvido
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // checkeo flor en juego
    let flor_en_juego = p.ronda.envite.estado >= EstadoEnvite::Flor;
    if flor_en_juego {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar el envido ahora porque la flor esta en juego".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let es_su_turno = p.ronda.get_el_turno().jugador.id == self.jid;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let envido_habilitado = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun || p.ronda.envite.estado == EstadoEnvite::Envido;
    
    if !envido_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar envido ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    let es_del_equipo_contrario = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun || p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let ya_estabamos_en_envido = p.ronda.envite.estado == EstadoEnvite::Envido;
    // apuestaSaturada = p.ronda.envite.Puntaje >= p.CalcPtsFalta()
    let apuesta_saturada = p.ronda.envite.puntaje >= 4;
    let truco_no_cantado = p.ronda.truco.estado == EstadoTruco::NoCantado;

    let esta_iniciando_por_primera_vez_el_envido = es_su_turno && p.ronda.envite.estado == EstadoEnvite::NoCantadoAun && truco_no_cantado;
    let esta_redoblando_la_apuesta = p.ronda.envite.estado == EstadoEnvite::Envido && es_del_equipo_contrario; // cuando redobla una apuesta puede o no ser su turno
    let el_envido_esta_primero = !es_su_turno && p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    let puede_tocar_envido = esta_iniciando_por_primera_vez_el_envido || esta_redoblando_la_apuesta || el_envido_esta_primero;

    let ok = !se_fue_al_mazo && (envido_habilitado && es_primera_mano && !tiene_flor && es_del_equipo_contrario) && puede_tocar_envido && !apuesta_saturada;

    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar 'Envido'".to_string(),
            }
          )
        });
      }  
      return (pkts, false)
    }
    
    // ok
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let ya_estabamos_en_envido = p.ronda.envite.estado == EstadoEnvite::Envido;
    let el_envido_esta_primero = p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    if el_envido_esta_primero {
      // deshabilito el truco
      p.ronda.truco.estado = EstadoTruco::NoCantado;
      p.ronda.truco.cantado_por = "".to_string();
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::ElEnvidoEstaPrimero {
              autor: self.jid.clone(),
            }
          )
        });
      }
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::TocarEnvido {
            autor: self.jid.clone(),
          }
        )
      });
    }

    // ahora checkeo si alguien tiene flor
    let hay_flor = p.ronda.envite.sin_cantar.len() > 0;
    if hay_flor {
      // todo: deberia ir al estado magico en el que espera
      // solo por jugadas de tipo flor-related
      // lo mismo para el real-envido; falta-envido
      let jid = p.ronda.envite.sin_cantar[0].clone();

      // todo
      let siguiente_jugada = CantarFlor{ jid };
      let mut res = siguiente_jugada.hacer(p);
      pkts.append(&mut res);

    } else {
      p.tocar_envido(&self.jid);
    }

    pkts
  }

  pub fn eval(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    
    p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
    p.ronda.envite.sin_cantar = Vec::new();
    let (j_ix, _, mut res) = p.ronda.exec_el_envido(p.verbose);
    pkts.append(&mut res);
    let jug = &p.ronda.manojos[j_ix].jugador;
    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::SumaPts {
            autor: jug.id.clone(),
            razon: enco::Razon::EnvidoGanado,
            pts: p.ronda.envite.puntaje
          }
        )
      });
    }
    p.suma_puntos(jug.equipo, p.ronda.envite.puntaje);
    pkts
  }
}

pub struct TocarRealEnvido {
  pub jid: String,
}
impl TocarRealEnvido {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdRealEnvido
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // checkeo flor en juego
    let flor_en_juego = p.ronda.envite.estado >= EstadoEnvite::Flor;
    if flor_en_juego {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar real envido ahora porque la flor esta en juego".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let es_su_turno = p.ronda.get_el_turno().jugador.id == self.jid;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let real_envido_habilitado = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun || p.ronda.envite.estado == EstadoEnvite::Envido;

    if !real_envido_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar real-envido ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    let es_del_equipo_contrario = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun || p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let ya_estabamos_en_envido = p.ronda.envite.estado == EstadoEnvite::Envido;
    let truco_no_cantado = p.ronda.truco.estado == EstadoTruco::NoCantado;

    let esta_iniciando_por_primera_vez_el_envido = es_su_turno && p.ronda.envite.estado == EstadoEnvite::NoCantadoAun && truco_no_cantado;
    let esta_redoblando_la_apuesta = p.ronda.envite.estado == EstadoEnvite::Envido && es_del_equipo_contrario; // cuando redobla una apuesta puede o no ser su turno;
    let el_envido_esta_primero = !es_su_turno && p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    let puede_tocar_real_envido = esta_iniciando_por_primera_vez_el_envido || esta_redoblando_la_apuesta || el_envido_esta_primero;
    let ok = !se_fue_al_mazo && (real_envido_habilitado && es_primera_mano && !tiene_flor && es_del_equipo_contrario) && puede_tocar_real_envido;

    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar 'Real Envido'".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    // ok
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let ya_estabamos_en_envido = p.ronda.envite.estado == EstadoEnvite::Envido;
    let el_envido_esta_primero = p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    if el_envido_esta_primero {
      // deshabilito el truco
      p.ronda.truco.estado = EstadoTruco::NoCantado;
      p.ronda.truco.cantado_por = "".to_string();
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::ElEnvidoEstaPrimero {
              autor: self.jid.clone(),
            }
          )
        });
      }
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::TocarRealEnvido {
            autor: self.jid.clone(),
          }
        )
      });
    }

    p.tocar_real_envido(&self.jid);
  
    // ahora checkeo si alguien tiene flor
    let hay_flor = p.ronda.envite.sin_cantar.len() > 0;
  
    if hay_flor {
      let jid = p.ronda.envite.sin_cantar[0].clone();
      let siguiente_jugada = CantarFlor{ jid };
      let mut res = siguiente_jugada.hacer(p);
      pkts.append(&mut res);
    }

    pkts
  }
}

pub struct TocarFaltaEnvido {
  pub jid: String,
}

impl TocarFaltaEnvido {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdFaltaEnvido
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // ok
    // checkeo flor en juego
    let flor_en_juego = p.ronda.envite.estado >= EstadoEnvite::Flor;
    if flor_en_juego {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar falta envido ahora porque la flor esta en juego".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let es_su_turno = p.ronda.get_el_turno().jugador.id == self.jid;
    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let falta_envido_habilitado = p.ronda.envite.estado >= EstadoEnvite::NoCantadoAun && p.ronda.envite.estado < EstadoEnvite::FaltaEnvido;

    if !falta_envido_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible tocar real-envido ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    let es_del_equipo_contrario = p.ronda.envite.estado == EstadoEnvite::NoCantadoAun || p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let ya_estabamos_en_envido = p.ronda.envite.estado >= EstadoEnvite::Envido;
    let truco_no_cantado = p.ronda.truco.estado == EstadoTruco::NoCantado;

    let esta_iniciando_por_primera_vez_el_envido = es_su_turno && p.ronda.envite.estado == EstadoEnvite::NoCantadoAun && truco_no_cantado;
    let esta_redoblando_la_apuesta = p.ronda.envite.estado >= EstadoEnvite::Envido && p.ronda.envite.estado < EstadoEnvite::FaltaEnvido && es_del_equipo_contrario; // cuando redobla una apuesta puede o no ser su turno;
    let el_envido_esta_primero = !es_su_turno && p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    let puede_tocar_falta_envido = esta_iniciando_por_primera_vez_el_envido || esta_redoblando_la_apuesta || el_envido_esta_primero;
    let ok = !se_fue_al_mazo && (falta_envido_habilitado && es_primera_mano && !tiene_flor && es_del_equipo_contrario) && puede_tocar_falta_envido;

    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar 'Falta Envido'".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }

    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    let es_primera_mano = p.ronda.mano_en_juego == NumMano::Primera;
    let ya_estabamos_en_envido = p.ronda.envite.estado == EstadoEnvite::Envido || p.ronda.envite.estado == EstadoEnvite::RealEnvido;
    let el_envido_esta_primero = p.ronda.truco.estado == EstadoTruco::Truco && !ya_estabamos_en_envido && es_primera_mano;

    if el_envido_esta_primero {
      // deshabilito el truco
      p.ronda.truco.estado = EstadoTruco::NoCantado;
      p.ronda.truco.cantado_por = "".to_string();
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::ElEnvidoEstaPrimero {
              autor: self.jid.clone(),
            }
          )
        });
      }
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::TocarFaltaEnvido {
            autor: self.jid.clone(),
          }
        )
      });
    }

    p.tocar_falta_envido(&self.jid);

    // ahora checkeo si alguien tiene flor
    let hay_flor = p.ronda.envite.sin_cantar.len() > 0;
    if hay_flor {
      let jid = p.ronda.envite.sin_cantar[0].clone();
      let siguiente_jugada = CantarFlor{ jid };
      let mut res = siguiente_jugada.hacer(p);
      pkts.append(&mut res);
    }

    pkts
  }

  pub fn eval(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();

    p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
	  p.ronda.envite.sin_cantar = Vec::new();

    // computar envidos
    let (j_idx, _, mut res) = p.ronda.exec_el_envido(p.verbose);

    pkts.append(&mut res);

    // jug es el que gano el (falta) envido
    let jug = &p.ronda.manojos[j_idx].jugador;

    let pts = p.calc_pts_falta_envido(jug.equipo);

    p.ronda.envite.puntaje += pts;

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::SumaPts{
            autor: jug.id.clone(),
            razon: enco::Razon::FaltaEnvidoGanado,
            pts: p.ronda.envite.puntaje
          }
        )
      });
    }

    p.suma_puntos(jug.equipo, p.ronda.envite.puntaje);

    pkts
  }
}

pub struct CantarFlor {
  pub jid: String,
}

/*

struct A
  ronda B

B.() retornaba un vector xs.
yo queria obtenerlo de forma mutable
en una funcion que tenia como arg a &A

fn bar(&A)
  mut xs = ()

no me dejaba hasta que defini fn como `fn bar(&mut A)`


*/

fn eval_flor(p: &mut Partida) -> Vec<enco::Packet> {
  let mut pkts: Vec<enco::Packet> = Vec::new();

	let flor_en_juego = p.ronda.envite.estado >= EstadoEnvite::Flor;
	let todos_los_jugadores_con_flor_cantaron = p.ronda.envite.sin_cantar.len() == 0;
	let ok = todos_los_jugadores_con_flor_cantaron && flor_en_juego;
	if !ok {
		return pkts
	}

  // cual es la flor ganadora?
	// empieza cantando el autor del envite no el que "quizo"
	let autor_idx = p.ronda.mixs[&p.ronda.envite.cantado_por];

  let equipo_ganador: Equipo;
  let ganador: String;

  {
    let (manojo_con_la_flor_mas_alta, _, mut res) = 
      p.ronda.exec_las_flores(autor_idx, p.verbose);
  
    pkts.append(&mut res);
    equipo_ganador = manojo_con_la_flor_mas_alta.jugador.equipo;
    ganador = manojo_con_la_flor_mas_alta.jugador.id.clone();
  }

  let puntos_asumar = p.ronda.envite.puntaje;
	p.suma_puntos(equipo_ganador, puntos_asumar);

	let habia_solo1_jugador_con_flor = p.ronda.envite.jugadores_con_flor.len() == 1;
  let razon = match habia_solo1_jugador_con_flor {
      true => enco::Razon::LaUnicaFlor,
      false => enco::Razon::LaFlorMasAlta,
  };
	if p.verbose {
    pkts.push(enco::Packet{
      destination: vec!["ALL".to_string()],
      message: enco::Message(
        enco::Content::SumaPts { 
          autor: ganador, 
          razon: razon, 
          pts: puntos_asumar
        }
      )
    });
  }

  p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
	p.ronda.envite.sin_cantar = Vec::new();

  pkts
}

impl CantarFlor {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdFlor
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // manojo dice que puede cantar flor;
    // es esto verdad?
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let flor_habilitada = (p.ronda.envite.estado >= EstadoEnvite::NoCantadoAun) && p.ronda.mano_en_juego == NumMano::Primera;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&self.jid);
    
    let ok = !se_fue_al_mazo && flor_habilitada && tiene_flor && no_canto_flor_aun;

    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar flor".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    // yo canto
    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::CantarContraFlor {
            autor: self.jid.clone(),
          }
        )
      });
    }

    p.ronda.truco.cantado_por = "".to_string();
	  p.ronda.truco.estado = EstadoTruco::NoCantado;

    // y me elimino de los que no-cantaron
    p.ronda.envite.canto_flor(&self.jid);
    p.cantar_flor(&self.jid);

    // es el ultimo en cantar flor que faltaba?
    // o simplemente es el unico que tiene flor (caso particular)
    let todos_los_jugadores_con_flor_cantaron = p.ronda.envite.sin_cantar.len() == 0;
    if todos_los_jugadores_con_flor_cantaron {
      let mut res = eval_flor(p);
      pkts.append(&mut res);
    } else {
      // cachear esto
      // solos los de su equipo tienen flor?
      // si solos los de su equipo tienen flor (y los otros no) -> las canto todas
      let mut solo_los_de_su_equipo_tienen_flor = true;

      for jid in p.ronda.envite.jugadores_con_flor.iter() {
        let manojo = p.ronda.manojo(jid);
        if manojo.jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo {
          solo_los_de_su_equipo_tienen_flor = false;
          break
        }
      }

      if solo_los_de_su_equipo_tienen_flor {
        // los quiero llamar a todos, pero no quiero Hacer llamadas al pedo
        // entonces: llamo al primero sin cantar, y que este llame al proximo
        // y que el proximo llame al siguiente, y asi...
        let jid = p.ronda.envite.sin_cantar[0].clone();
        // j = p.ronda.manojo(jid);
        let siguiente_jugada = CantarFlor{ jid };
        let mut res = siguiente_jugada.hacer(p);
        pkts.append(&mut res);
      }
    }

    pkts
  }
}

pub struct CantarContraFlor {
  pub jid: String,
}
impl CantarContraFlor {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdContraFlor
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let contra_flor_habilitada = p.ronda.envite.estado == EstadoEnvite::Flor && p.ronda.mano_en_juego == NumMano::Primera;
    let es_del_equipo_contrario = contra_flor_habilitada && p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&self.jid);
    let ok = !se_fue_al_mazo && contra_flor_habilitada && tiene_flor && es_del_equipo_contrario && no_canto_flor_aun;
    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar contra flor".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    // la canta
    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::CantarContraFlor {
            autor: self.jid.clone(),
          }
        )
      });
    }

    p.cantar_contra_flor(&self.jid);
    // y ahora tengo que esperar por la respuesta de la nueva
    // propuesta de todos menos de el que canto la contraflor
    // restauro la copia
    p.ronda.envite.canto_flor(&self.jid);

    pkts
  }
}

pub struct CantarContraFlorAlResto {
  pub jid: String,
}
impl CantarContraFlorAlResto {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdContraFlorAlResto
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let contra_flor_habilitada = (p.ronda.envite.estado == EstadoEnvite::Flor || p.ronda.envite.estado == EstadoEnvite::ContraFlor) && p.ronda.mano_en_juego == NumMano::Primera;
    let es_del_equipo_contrario = contra_flor_habilitada && p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&self.jid);
    let ok = !se_fue_al_mazo && contra_flor_habilitada && tiene_flor && es_del_equipo_contrario && no_canto_flor_aun;
    if !ok {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar contra flor al resto".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::CantarContraFlorAlResto {
            autor: self.jid.clone(),
          }
        )
      });
    }

    p.cantar_contra_flor_al_resto(&self.jid);
    // y ahora tengo que esperar por la respuesta de la nueva
    // propuesta de todos menos de el que canto la contraflor
    // restauro la copia
    p.ronda.envite.canto_flor(&self.jid);

    pkts
  }
}

pub struct GritarTruco {
  pub jid: String,
}
impl GritarTruco {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdTruco
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // checkeos:
    let no_se_fue_al_mazo = !p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let no_se_esta_jugando_el_envite = p.ronda.envite.estado <= EstadoEnvite::NoCantadoAun;

    let yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto = p.ronda.hay_equipo_sin_cantar(p.ronda.manojo(&self.jid).jugador.equipo);

    let la_flor_esta_primero = yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto;
    let truco_no_se_jugo_aun = p.ronda.truco.estado == EstadoTruco::NoCantado;
    let es_su_turno = p.ronda.get_el_turno().jugador.id == self.jid;
    let truco_habilitado = no_se_fue_al_mazo && truco_no_se_jugo_aun && no_se_esta_jugando_el_envite && !la_flor_esta_primero && es_su_turno;

    if !truco_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar truco ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false)
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::GritarTruco {
            autor: self.jid.clone(),
          }
        )
      });
    }
  
    p.gritar_truco(&self.jid);

    pkts
  }
}

pub struct GritarReTruco {
  pub jid: String,
}
impl GritarReTruco {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdTirarCarta
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();

    let no_se_fue_al_mazo = !p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let no_se_esta_jugando_el_envite = p.ronda.envite.estado <= EstadoEnvite::NoCantadoAun;
    let yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto = p.ronda.hay_equipo_sin_cantar(p.ronda.manojo(&self.jid).jugador.equipo);
    let la_flor_esta_primero = yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto;

    // CASO I:
    let truco_gritado = p.ronda.truco.estado == EstadoTruco::Truco;
    let uno_del_equipo_contrario_grito_truco = truco_gritado && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let caso_i = truco_gritado && uno_del_equipo_contrario_grito_truco;

    // CASO II:
    let truco_ya_querido = p.ronda.truco.estado == EstadoTruco::TrucoQuerido;
    let uno_de_mi_equipo_quizo = truco_ya_querido && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo == p.ronda.manojo(&self.jid).jugador.equipo;
    // esTurnoDeMiEquipo = p.ronda.get_el_turno().jugador.equipo == p.ronda.manojo(&self.jid).jugador.equipo;
    let caso_ii = truco_ya_querido && uno_de_mi_equipo_quizo; // && esTurnoDeMiEquipo;

    let re_truco_habilitado = no_se_fue_al_mazo && no_se_esta_jugando_el_envite && (caso_i || caso_ii) && !la_flor_esta_primero;

    if !re_truco_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar re-truco ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::GritarReTruco {
            autor: self.jid.clone(),
          }
        )
      });
    }
  
    p.gritar_retruco(&self.jid);

    pkts
  }
}

pub struct GritarVale4 {
  pub jid: String,
}
impl GritarVale4 {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdVale4
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let no_se_fue_al_mazo = !p.ronda.manojo(&self.jid).se_fue_al_mazo;
    let no_se_esta_jugando_el_envite = p.ronda.envite.estado <= EstadoEnvite::NoCantadoAun;
    let yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto = p.ronda.hay_equipo_sin_cantar(p.ronda.manojo(&self.jid).jugador.equipo);
    let la_flor_esta_primero = yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto;
    // CASO I:
    let re_truco_gritado = p.ronda.truco.estado == EstadoTruco::ReTruco;
    // para eviat el nil primero checkeo que haya sido gritado reTrucoGritado &&
    let uno_del_equipo_contrario_grito_re_truco = re_truco_gritado && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let caso_i = re_truco_gritado && uno_del_equipo_contrario_grito_re_truco;
    // CASO I:
    let retruco_ya_querido = p.ronda.truco.estado == EstadoTruco::ReTrucoQuerido;
    // para eviat el nil primero checkeo que haya sido gritado reTrucoGritado &&
    let su_equipotiene_el_quiero = retruco_ya_querido && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo == p.ronda.manojo(&self.jid).jugador.equipo;
    let caso_ii = retruco_ya_querido && su_equipotiene_el_quiero;
    let vale4_habilitado = no_se_fue_al_mazo && (caso_i || caso_ii) && no_se_esta_jugando_el_envite && !la_flor_esta_primero;
    if !vale4_habilitado {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible cantar vale-4 ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    if p.verbose {
      pkts.push(enco::Packet{
        destination: vec!["ALL".to_string()],
        message: enco::Message(
          enco::Content::GritarVale4 {
            autor: self.jid.clone(),
          }
        )
      });
    }
  
    p.gritar_vale4(&self.jid);

    pkts
  }
}

pub struct ResponderQuiero {
  pub jid: String,
}
impl ResponderQuiero {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdQuiero
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    if se_fue_al_mazo {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "Te fuiste al mazo; no podes Hacer esta jugada".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    // checkeo flor en juego
    // caso particular del checkeo:
    // no se le puede decir quiero ni al envido* ni al truco si se esta jugando la flor
    // no se le puede decir quiero a la flor -> si la flor esta en juego -> error
    // pero si a la contra flor o contra flor al resto

    // casos posibles:
    // alguien dijo envido/truco, otro responde quiero, pero hay uno que tiene flor que todavia no la jugo -> deberia saltar error: "alguien tiene flor y no la jugo aun"
    // alguien tiene flor, uno dice quiero -> no deberia dejarlo porque la flor no se responde con quiero
    // se esta jugando la contra-flor/CFAR -> ok

    let flor_en_juego = p.ronda.envite.estado == EstadoEnvite::Flor;
    if flor_en_juego {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible responder quiero ahora".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    let no_han_cantado_la_flor_aun = p.ronda.envite.estado < EstadoEnvite::Flor;
    let yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto = p.ronda.hay_equipo_sin_cantar(p.ronda.manojo(&self.jid).jugador.equipo);
    if no_han_cantado_la_flor_aun && yo_ouno_de_mis_compas_tiene_flor_yaun_no_canto {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No es posible responder 'quiero' porque alguien con flor no ha cantado aun".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }
    // se acepta una respuesta 'quiero' solo cuando:
    // - CASO I: se toco un envite+ (con autor del equipo contario)
    // - CASO II: se grito el truco+ (con autor del equipo contario)
    // en caso contrario, es incorrecto -> error

    let el_envido_es_respondible = p.ronda.envite.estado >= EstadoEnvite::Envido && p.ronda.envite.estado <= EstadoEnvite::FaltaEnvido;
    // ojo: solo a la contraflor+ se le puede decir quiero; a la flor sola no
    let la_contra_flor_es_respondible = p.ronda.envite.estado >= EstadoEnvite::ContraFlor && p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let el_truco_es_respondible = p.ronda.truco.estado.es_truco_respondible() && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;

    let ok = el_envido_es_respondible || la_contra_flor_es_respondible || el_truco_es_respondible;
    if !ok {
      // si no, esta respondiendo al pedo
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "No hay nada 'que querer'; ya que: el estado del envido no es 'envido' (o mayor) y el estado del truco no es 'truco' (o mayor) o bien fue cantado por uno de su equipo".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    if el_envido_es_respondible {
      let es_del_equipo_contrario = p.ronda.manojo(&self.jid).jugador.equipo != p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo;
      if !es_del_equipo_contrario {
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec![self.jid.clone()],
            message: enco::Message(
              enco::Content::Error {
                msg: "La jugada no es valida".to_string(),
              }
            )
          });
        }
        return (pkts, false);
      }

    } else if la_contra_flor_es_respondible {
      // tengo que verificar si efectivamente tiene flor
      let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
      let es_del_equipo_contrario = p.ronda.manojo(&self.jid).jugador.equipo != p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo;
      let ok = tiene_flor && es_del_equipo_contrario;
      if !ok {
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec![self.jid.clone()],
            message: enco::Message(
              enco::Content::Error {
                msg: "La jugada no es valida".to_string(),
              }
            )
          });
        }
        return (pkts, false);
      }
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    // se acepta una respuesta 'quiero' solo cuando:
    // - CASO I: se toco un envite+ (con autor del equipo contario)
    // - CASO II: se grito el truco+ (con autor del equipo contario)
    // en caso contrario, es incorrecto -> error

    let el_envido_es_respondible = p.ronda.envite.estado >= EstadoEnvite::Envido && p.ronda.envite.estado <= EstadoEnvite::FaltaEnvido;
    // ojo: solo a la contraflor+ se le puede decir quiero; a la flor sola no
    let la_contra_flor_es_respondible = p.ronda.envite.estado >= EstadoEnvite::ContraFlor && p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    let el_truco_es_respondible = p.ronda.truco.estado.es_truco_respondible() && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;

    if el_envido_es_respondible {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::QuieroEnvite {
              autor: self.jid.clone(),
            }
          )
        });
      }

      if p.ronda.envite.estado == EstadoEnvite::FaltaEnvido {
        let mut res = TocarFaltaEnvido{jid: self.jid.clone()}.eval(p);
        pkts.append(&mut res);
        return pkts;
      }
      // si no, era envido/real-envido o cualquier
      // combinacion valida de ellos

      let mut res = TocarEnvido{jid: self.jid.clone()}.eval(p);
      pkts.append(&mut res);
      return pkts;

    } else if la_contra_flor_es_respondible {

      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::QuieroEnvite {
              autor: self.jid.clone(),
            }
          )
        });
      }

      // empieza cantando el autor del envite no el que "quizo"
      let autor_idx = p.ronda.mixs[&p.ronda.envite.cantado_por];
      let equipo_ganador: Equipo;
      let ganador: String;
      {
        let (manojo_con_la_flor_mas_alta, _, mut res) = 
          p.ronda.exec_las_flores(autor_idx, p.verbose);
        equipo_ganador = manojo_con_la_flor_mas_alta.jugador.equipo;
        ganador = manojo_con_la_flor_mas_alta.jugador.id.clone();
        pkts.append(&mut res);
      }

      if p.ronda.envite.estado == EstadoEnvite::ContraFlor {
        let puntos_asumar = p.ronda.envite.puntaje;
        p.suma_puntos(equipo_ganador, puntos_asumar);
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec!["ALL".to_string()],
            message: enco::Message(
              enco::Content::SumaPts {
                autor: ganador,
                razon: enco::Razon::ContraFlorGanada,
                pts: puntos_asumar
              }
            )
          });
        }
      } else {
        // el equipo del ganador de la contraflor al resto
        // gano la partida
        // duda se cuentan las flores?
        // puntosASumar = p.ronda.envite.puntaje + p.CalcPtsContraFlorAlResto(equipoGanador);
        let puntos_asumar = p.calc_pts_contraflor_al_resto(equipo_ganador);
        p.suma_puntos(equipo_ganador, puntos_asumar);
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec!["ALL".to_string()],
            message: enco::Message(
              enco::Content::SumaPts {
                autor: ganador,
                razon: enco::Razon::ContraFlorAlRestoGanada,
                pts: puntos_asumar
              }
            )
          });
        }
      }
      p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
      p.ronda.envite.sin_cantar = Vec::new();
    } else if el_truco_es_respondible {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::QuieroTruco {
              autor: self.jid.clone(),
            }
          )
        });
      }
      p.querer_truco(&self.jid)
    }

    pkts
  }
}

pub struct ResponderNoQuiero {
  pub jid: String,
}
impl ResponderNoQuiero {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdNoQuiero
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    
    let se_fue_al_mazo = p.ronda.manojo(&self.jid).se_fue_al_mazo;
    if se_fue_al_mazo {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: "Te fuiste al mazo; no podes Hacer esta jugada".to_string(),
            }
          )
        });
      }
      return (pkts, false);
    }

    // checkeo flor en juego
    // caso particular del checkeo: no se le puede decir quiero a la flor
    // pero si a la contra flor o contra flor al resto
    // FALSO porque el no quiero lo estoy contando como un "con flor me achico"
    // todo: agregar la jugada: "con flor me achico" y editar la variale:
    // AHORA:
    // laFlorEsRespondible = p.ronda.Flor >= EstadoEnvite::Flor && p.ronda.manojo[p.ronda.envite.cantado_por].jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
    // LUEGO DE AGREGAR LA JUGADA "con flor me achico"
    // laFlorEsRespondible = p.ronda.Flor > EstadoEnvite::Flor;
    // FALSO ---> directamente se va la posibilidad de reponderle
    // "no quiero a la flor"

    // se acepta una respuesta 'no quiero' solo cuando:
    // - CASO I: se toco el envido (o similar)
    // - CASO II: se grito el truco (o similar)
    // en caso contrario, es incorrecto -> error

    let el_envido_es_respondible = (p.ronda.envite.estado >= EstadoEnvite::Envido && p.ronda.envite.estado <= EstadoEnvite::FaltaEnvido) && p.ronda.envite.cantado_por != self.jid;
    let la_flor_es_respondible = p.ronda.envite.estado >= EstadoEnvite::Flor && p.ronda.envite.cantado_por != self.jid;
    let el_truco_es_respondible = p.ronda.truco.estado.es_truco_respondible() && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;

    let ok = el_envido_es_respondible || la_flor_es_respondible || el_truco_es_respondible;

    if !ok {
      // si no, esta respondiendo al pedo
      let err = 
        format!(
          "{} esta respondiendo al pedo; no hay nada respondible",
          self.jid.clone()
        );
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec![self.jid.clone()],
          message: enco::Message(
            enco::Content::Error {
              msg: err,
            }
          )
        });
      }
      return (pkts, false);
    }

    if el_envido_es_respondible {
      let es_del_equipo_contrario = p.ronda.manojo(&self.jid).jugador.equipo != p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo;
      if !es_del_equipo_contrario {
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec![self.jid.clone()],
            message: enco::Message(
              enco::Content::Error {
                msg: "La jugada no es valida".to_string(),
              }
            )
          });
        }
        return (pkts, false);
      }
    } else if la_flor_es_respondible {
      // tengo que verificar si efectivamente tiene flor
      let (tiene_flor, _) = p.ronda.manojo(&self.jid).tiene_flor(&p.ronda.muestra);
      let es_del_equipo_contrario = p.ronda.manojo(&self.jid).jugador.equipo != p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo;
      let ok = tiene_flor && es_del_equipo_contrario;

      if !ok {
        if p.verbose {
          pkts.push(enco::Packet{
            destination: vec![self.jid.clone()],
            message: enco::Message(
              enco::Content::Error {
                msg: "La jugada no es valida".to_string(),
              }
            )
          });
        }
        return (pkts, false);
      }
    }
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    let el_envido_es_respondible = (p.ronda.envite.estado >= EstadoEnvite::Envido && p.ronda.envite.estado <= EstadoEnvite::FaltaEnvido) && p.ronda.envite.cantado_por != self.jid;
    let la_flor_es_respondible = p.ronda.envite.estado >= EstadoEnvite::Flor && p.ronda.envite.cantado_por != self.jid;
    let el_truco_es_respondible = p.ronda.truco.estado.es_truco_respondible() && p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;

    if el_envido_es_respondible {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::NoQuiero {
              autor: self.jid.clone(),
            }
          )
        });
      }

      //	no se toma en cuenta el puntaje total del ultimo toque

      let total_pts: usize = match p.ronda.envite.estado {
        EstadoEnvite::Envido => p.ronda.envite.puntaje - 1,
        EstadoEnvite::RealEnvido => p.ronda.envite.puntaje - 2,
        EstadoEnvite::FaltaEnvido => p.ronda.envite.puntaje + 1,
        _ => unreachable!()
      };

      p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
      p.ronda.envite.sin_cantar = Vec::new();
      p.ronda.envite.puntaje = total_pts;

      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::SumaPts {
              autor: p.ronda.envite.cantado_por.clone(),
              razon: enco::Razon::EnviteNoQuerido,
              pts: total_pts
            }
          )
        });
      }

      p.suma_puntos(
        p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo, 
        total_pts
      );

    } else if la_flor_es_respondible {

      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::ConFlorMeAchico {
              autor: self.jid.clone(),
            }
          )
        });
      }

      // sumo todas las flores del equipo contrario
      let mut total_pts = 0;

      for m in p.ronda.manojos.iter() {
        let es_del_equipo_contrario = p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo != p.ronda.manojo(&self.jid).jugador.equipo;
        let (tiene_flor, _) = m.tiene_flor(&p.ronda.muestra);
        if tiene_flor && es_del_equipo_contrario {
          total_pts += 3
        }
      }

      if p.ronda.envite.estado == EstadoEnvite::ContraFlor || p.ronda.envite.estado == EstadoEnvite::ContraFlorAlResto {
        // si es contraflor o al resto
        // se suma 1 por el `no quiero`
        total_pts += 1;
      }

      p.ronda.envite.estado = EstadoEnvite::Deshabilitado;
      p.ronda.envite.sin_cantar = Vec::new();
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::SumaPts { 
              autor: p.ronda.envite.cantado_por.clone(),
              razon: enco::Razon::FlorAchicada,
              pts: total_pts
            }
          )
        });
      }
      p.suma_puntos(
        p.ronda.manojo(&p.ronda.envite.cantado_por).jugador.equipo, 
        total_pts
      );

    } else if el_truco_es_respondible {
      if p.verbose {
        pkts.push(enco::Packet{
          destination: vec!["ALL".to_string()],
          message: enco::Message(
            enco::Content::NoQuiero { 
              autor: self.jid.clone(),
            }
          )
        });
      }

      // pongo al equipo que propuso el truco como ganador de la mano actual
      let mano_actual = p.ronda.mano_en_juego as usize;
      p.ronda.manos[mano_actual].ganador = p.ronda.truco.cantado_por.clone();
      let mut equipo_ganador = Resultado::GanoAzul;
      if p.ronda.manojo(&p.ronda.truco.cantado_por).jugador.equipo == Equipo::Rojo {
        equipo_ganador = Resultado::GanoRojo;
      }
      p.ronda.manos[mano_actual].resultado = equipo_ganador;

      let (nueva_ronda, mut res) = p.evaluar_ronda();
      pkts.append(&mut res);
      if nueva_ronda {
        if !p.terminada() {
          // ahora se deberia de incrementar el mano
          // y ser el turno de este
          let sig_mano = p.ronda.get_sig_el_mano();
          p.ronda.nueva_ronda(sig_mano); // todo: el tema es que cuando llama aca
          // no manda mensaje de que arranco nueva ronda
          // falso: el padre que llama a .EvaluarRonda tiene que fijarse si
          // retorno true
          // entonces debe crearla el
          // no es responsabilidad de EvaluarRonda arrancar una ronda nueva!!
          // de hecho, si una ronda es terminable y se llama 2 veces consecutivas
          // al mismo metodo booleano, en ambas oportunidades retorna diferente
          // ridiculo
          for _m in p.ronda.manojos.iter() {

            // todo: aca va una perspctiva de p segun m
            // todo!()
            // pkts = append(pkts, enco.Pkt(
            //   enco.Dest(m.jugador.id),
            //   enco.Msg(enco.NuevaRonda, p.PerspectivaCacheFlor(&m)),
            // ))

          }

        } // else {
        // p.byeBye()
        // }

      }

    }

    pkts
  }
}

/*
pub struct Foo {
  pub jid: String,
}
impl Foo {
  pub fn id() -> IJugadaId {
    IJugadaId::JIdTirarCarta
  }
  pub fn ok(&self, p:&Partida) -> (Vec<enco::Packet>, bool) {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    // checkeos
    (pkts, true)
  }

  pub fn hacer(&self, p:&mut Partida) -> Vec<enco::Packet> {
    let mut pkts: Vec<enco::Packet> = Vec::new();
    let (mut pre, ok) = self.ok(p);
    pkts.append(&mut pre);

    if !ok {
      return pkts
    }

    pkts
  }
}
*/
