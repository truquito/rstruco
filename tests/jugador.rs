#[test]
fn struct_test() {
  let j: truco::Jugador = truco::Jugador{
    id: String::from("pepe"),
    equipo: truco::Equipo::Azul
  };
  {
    let exp = r#"Jugador { id: "pepe", equipo: Azul }"#;
    assert!(format!("{:?}", j) == exp);
    assert!(j.id == "pepe");
  }
 
  let json = r#"{"id": "rOsA","equipo": "rojo"}"#;
  let p: truco::Jugador = serde_json::from_str(json).unwrap();
  {
    let got = format!("{:?}", p);
    let exp = r#"Jugador { id: "rOsA", equipo: Rojo }"#;
    assert!(got==exp);
    assert!(p.id == "rOsA");
  }
  // deben ser de equipos diferentes
  assert!(j.equipo != p.equipo);
}

// si el valor del atribute está en case raro -> tira error
#[test]
#[should_panic]
fn json_invalido_test() {
  let json = r#"{"id": "alice","equipo": "aZUl"}"#;
  let _x: truco::Jugador = serde_json::from_str(json)
    .expect("JSON was not well-formatted");
}

// si el nombre del atributo esta en case raro -> tira error
#[test]
#[should_panic(expected="JSON was not well-formatted")]
fn json_invalido_test2() {
  let json = r#"{"id": "alice","Equipo": "azul"}"#;
  let _x: truco::Jugador = serde_json::from_str(json)
    .expect("JSON was not well-formatted");
}