use truco::*;

#[test]
fn mano_ord_test(){
  let p = NumMano::Primera;
  let s = NumMano::Segunda;
  // let t = Mano::Primera;
  assert!(p < s && s == NumMano::Segunda && s < NumMano::Tercera)
}

#[test]
fn mano_ix_test(){
  let p = NumMano::Primera;
  let s = NumMano::Segunda;
  let t = NumMano::Tercera;
  assert!(0 == p as usize);
  assert!(1 == s as usize);
  assert!(2 == t as usize);
}

#[test]
fn mano_struct_test(){
  let mut m = Mano {
    resultado: Resultado::Indeterminado,
    ganador: String::from(""),
    cartas_tiradas: Vec::new(),
  };
  
  assert!(m.cartas_tiradas.len() == 0);

  m.agregar_tirada(
    CartaTirada {
      jugador: String::from("alice"),
      carta: Carta::new(2, "copa").unwrap()
    }
  );

  assert!(m.cartas_tiradas.len() == 1);
}

#[test]
fn mano_json_test(){
  let mut m = Mano {
    resultado: Resultado::Indeterminado,
    ganador: String::from(""),
    cartas_tiradas: Vec::new(),
  };
  m.agregar_tirada(
    CartaTirada {
      jugador: String::from("alice"),
      carta: Carta::new(2, "copa").unwrap()
    }
  );
  let _json = serde_json::to_string(&m).unwrap();
  // println!("the JSON is: {}", _json);
  assert!(m.cartas_tiradas.len() == 1);
}
