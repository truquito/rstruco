use std::collections::HashMap;
use truco::*;

#[test]
fn ronda_manual_json_test(){
  let r = Ronda {
    mano_en_juego: NumMano::Primera,
    cant_jugadores_en_juego: HashMap::from([
      (Equipo::Azul, 4),
      (Equipo::Rojo, 2)
    ]),
    // Indices
    el_mano: 0,
    turno:  0,
    // gritos y toques/cantos
    envite: Envite {
      estado: EstadoEnvite::NoCantadoAun,
      puntaje: 0,
      cantado_por: String::from(""),
      jugadores_con_flor: Vec::new(),
      // alternativa
      // pub jugadores_con_flor: Vec<String>,
      sin_cantar: Vec::new(),
    },
    truco: Truco{
      cantado_por: String::from(""),
      estado: EstadoTruco::NoCantado,
    },
    manojos: Vec::new(),
    muestra: Carta::new(4, "copa").unwrap(),
    mixs: HashMap::new(),
    manos: Default::default(),
  };

  let _json = serde_json::to_string(&r).unwrap();
  // println!("the JSON is: {}", _json);
}

#[test]
#[should_panic] // ya que le estoy enviado 2 usuarios con el mismo nombre 
fn new_ronda_duplicate_ids_test(){
  let azules = vec![String::from("alice"), String::from("ana")];
  let rojos = vec![String::from("bob"), String::from("alice")];
  Ronda::new(azules, rojos).unwrap();
}

#[test]
fn new_ronda_json_test(){
  let azules = vec!["alice".to_string(), "ana".to_string()];
  let rojos = vec!["bob".to_string(), "ben".to_string()];
  let r = Ronda::new(azules, rojos).unwrap();
  let _json = serde_json::to_string(&r).unwrap();
  // println!("the JSON is: {}", _json);
}