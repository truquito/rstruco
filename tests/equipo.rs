#[test]
fn enum_test() {
  let e1 = truco::Equipo::Azul;
  assert_eq!("Azul", format!("{}", e1)); // <- llama a Display

  let e2 = truco::Equipo::Rojo;
  assert_eq!("Rojo", format!("{:?}", e2)); // <- llama a Debug

  // deriva de PartialEq
  assert!(e1!=e2);
  assert!(e1==truco::Equipo::Azul);
}

#[test]
fn contrario_test() {
  let e1 = truco::Equipo::Azul;
  assert!(e1.equipo_contrario()==truco::Equipo::Rojo);
}
