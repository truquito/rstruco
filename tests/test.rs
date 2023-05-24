#[test]
fn it_works() {
  let result = truco::add(2, 2);
  assert_eq!(result, 4);
}

#[test]
fn enum_test() {
  // let e1 = truco::equipo::Equipo::Azul;
  let e1 = truco::Equipo::Azul;
  println!("enum is: {}", e1); // <- llama a Display

  let e2 = truco::Equipo::Rojo;
  println!("enum is: {}", e2); // <- llama a Debug

  println!("are equal? {}", e1==e2); // <- llama a Debug
  println!("are equal? {}", e1==truco::Equipo::Azul); // <- llama a Debug

}

#[test]
fn jugador_test() {
  let j = truco::Jugador{
    id: String::from("pepe"),
    equipo: truco::Equipo::Azul
  };
  println!("{:?} su nombre es {:?}", j, j.id);

  let json = r#"{"id": "rOsA","equipo": "rojo"}"#;
  let p: truco::Jugador = serde_json::from_str(json).unwrap();
  println!("{:?}", p);
  println!("{:?}", j.equipo == p.equipo);
}

// si el valor del atribute está en case raro -> tira error
#[test]
#[should_panic]
fn jugador_invalid_json_test() {
  let json = r#"{"id": "alice","equipo": "aZUl"}"#;
  let _x: truco::Jugador = serde_json::from_str(json).expect("JSON was not well-formatted");
}

// si el nombre del atributo esta en case raro -> tira error
#[test]
#[should_panic(expected="JSON was not well-formatted")]
fn jugador_invalid_json_test2() {
  let json = r#"{"id": "alice","Equipo": "azul"}"#;
  let _x: truco::Jugador = serde_json::from_str(json).expect("JSON was not well-formatted");
}