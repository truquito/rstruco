use std::fmt::Debug;
use crate::partida::{Partida};
use crate::{enco, EstadoEnvite, NumMano, EstadoTruco};
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
      p.ronda.get_el_turno().jugador.id == p.ronda.manojo(&self.jid).jugador.id;
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
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&p.ronda.manojo(&self.jid).jugador.id);
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
    let yo_gite_el_truco = truco_gritado && p.ronda.manojo(&self.jid).jugador.id == p.ronda.truco.cantado_por;
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
    let es_su_turno = p.ronda.get_el_turno().jugador.id == p.ronda.manojo(&self.jid).jugador.id;
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
      let _jid = p.ronda.envite.sin_cantar[0].clone();

      // todo
      todo!()
      // let siguienteJugada = CantarFlor{jid};
      // let res = siguienteJugada.Hacer(p);
      // pkts = append(pkts, res...)

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
    let es_su_turno = p.ronda.get_el_turno().jugador.id == p.ronda.manojo(&self.jid).jugador.id;
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
      // let jid = p.ronda.envite.sin_cantar[0];
      todo!();
      // todo!
      // j = p.ronda.manojo(jid);
      // let siguienteJugada = CantarFlor{jid};
      // let res = siguienteJugada.Hacer(p);
      // pkts = append(pkts, res...)
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
    let es_su_turno = p.ronda.get_el_turno().jugador.id == p.ronda.manojo(&self.jid).jugador.id;
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
      todo!();
      // todo
      // let jid = p.ronda.envite.sin_cantar[0];
      // // j = p.ronda.manojo(jid);
      // let siguienteJugada = CantarFlor{jid};
      // let res = siguienteJugada.Hacer(p);
      // pkts = append(pkts, res...)
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

B.foo() retornaba un vector xs.
yo queria obtenerlo de forma mutable
en una funcion que tenia como arg a &A

fn bar(&A)
  mut xs = foo()

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
    let no_canto_flor_aun = p.ronda.envite.no_canto_flor_aun(&p.ronda.manojo(&self.jid).jugador.id);
    
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

    pkts
  }
}
*/
