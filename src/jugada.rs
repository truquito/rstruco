use std::fmt::Debug;
use crate::partida::{Partida};
use crate::{enco, EstadoEnvite, NumMano};
use crate::carta::{Carta};

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
            autor: "".to_string(),
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