use truco::*;

#[test]
fn partida_test(){
  let json = r#"{"puntuacion":20,"puntajes":{"azul":0,"rojo":0},"ronda":{"mano_en_juego":"primera","cant_jugadores_en_juego":{"azul":1,"rojo":1},"el_mano":0,"turno":0,"envite":{"estado":"deshabilitado","puntaje":0,"cantado_por":"","sin_cantar":[]},"truco":{"cantado_por":"alice","estado":"vale4"},"manojos":[{"se_fue_al_mazo":false,"cartas":[{"valor":5,"palo":"espada"},{"valor":1,"palo":"oro"},{"valor":4,"palo":"basto"}],"tiradas":[true,true,true],"ultima_tirada":0,"jugador":{"id":"alice","equipo":"azul"}},{"se_fue_al_mazo":false,"cartas":[{"valor":5,"palo":"basto"},{"valor":3,"palo":"basto"},{"valor":7,"palo":"espada"}],"tiradas":[false,true,false],"ultima_tirada":1,"jugador":{"id":"bob","equipo":"rojo"}}],"muestra":{"valor":4,"palo":"oro"},"manos":[{"resultado":"ganoRojo","ganador":"bob","cartas_tiradas":[{"jugador":"alice","carta":{"valor":4,"palo":"basto"}},{"jugador":"bob","carta":{"valor":3,"palo":"basto"}},{"jugador":"alice","carta":{"valor":1,"palo":"oro"}},{"jugador":"alice","carta":{"valor":5,"palo":"espada"}}]},{"resultado":"indeterminado","ganador":"","cartas_tiradas":[]},{"resultado":"indeterminado","ganador":"","cartas_tiradas":[]}]}}"#;
  let p: Partida = serde_json::from_str(json).unwrap();
  println!("{:?}", p)
}