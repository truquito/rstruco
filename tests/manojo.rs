use truco::*;

#[test]
fn manojo_test() {
  let mut m = Manojo{
    jugador: Jugador {
      id: String::from("Alice"),
      equipo: Equipo::Azul,
    },
    se_fue_al_mazo: false,
    ultima_tirada: -1,
    tiradas: [false, false, false],
    cartas: [
      Carta::new(1, "copa").unwrap(),
      Carta::new(2, "copa").unwrap(),
      Carta::new(3, "copa").unwrap(),
    ]
  };
  println!("{:?}", m);

  m.cartas = [
    Carta::new(4, "oro").unwrap(),
    Carta::new(5, "oro").unwrap(),
    Carta::new(6, "oro").unwrap(),
  ];

  let mut cs = get_cartas_random(6);
  m.cartas[0] = cs.pop().unwrap();
  m.cartas[1] = cs.pop().unwrap();
  m.cartas[2] = cs.pop().unwrap();

  println!("{:?}", m);
}

#[test]
fn cant_tiradas_test() {
  let m = Manojo{
    jugador: Jugador {
      id: String::from("Alice"),
      equipo: Equipo::Azul,
    },
    se_fue_al_mazo: false,
    ultima_tirada: -1,
    tiradas: [true, false, true],
    cartas: [
      Carta::new(1, "copa").unwrap(),
      Carta::new(2, "copa").unwrap(),
      Carta::new(3, "copa").unwrap(),
    ]
  };

  assert_eq!(2, m.get_cant_cartas_tiradas());
}

#[test]
fn carta_idx_test() {
  let m = Manojo{
    jugador: Jugador {
      id: String::from("Alice"),
      equipo: Equipo::Azul,
    },
    se_fue_al_mazo: false,
    ultima_tirada: -1,
    tiradas: [true, false, true],
    cartas: [
      Carta::new(1, "copa").unwrap(),
      Carta::new(2, "copa").unwrap(),
      Carta::new(3, "copa").unwrap(),
    ]
  };

  let c = Carta::new(3, "copa").unwrap();
  assert_eq!(2, m.get_carta_idx(&c));

  let c = Carta::new(1, "copa").unwrap();
  assert_eq!(0, m.get_carta_idx(&c));

  let c = Carta::new(2, "copa").unwrap();
  assert_eq!(1, m.get_carta_idx(&c));
}


#[test]
fn tiene_flor_test() {
  let muestra = Carta::new(5, "copa").unwrap();

  let mut m = Manojo::new(
    Jugador { id: String::from("Alice"), equipo: Equipo::Azul },
    [
      Carta::new(6, "oro").unwrap(),
      Carta::new(10, "copa").unwrap(),
      Carta::new(7, "copa").unwrap(),
    ]
  );
  let got = m.tiene_flor(&muestra);
  assert_eq!(false, got.0);
  assert_eq!(-1, got.1);

  m.cartas = [
    Carta::new(1, "copa").unwrap(),
    Carta::new(2, "oro").unwrap(),
    Carta::new(3, "basto").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(false, got.0);
  assert_eq!(-1, got.1);

  m.cartas = [
    Carta::new(4, "copa").unwrap(),
    Carta::new(2, "copa").unwrap(),
    Carta::new(3, "basto").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(true, got.0);
  assert_eq!(1, got.1);

  let muestra = Carta::new(1, "copa").unwrap();
  m.cartas = [
    Carta::new(12, "copa").unwrap(),
    Carta::new(10, "copa").unwrap(),
    Carta::new(1, "basto").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(false, got.0);
  assert_eq!(-1, got.1);

  let muestra = Carta::new(1, "copa").unwrap();
  m.cartas = [
    Carta::new(12, "copa").unwrap(),
    Carta::new(10, "copa").unwrap(),
    Carta::new(1, "basto").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(false, got.0);
  assert_eq!(-1, got.1);

  let muestra = Carta::new(5, "copa").unwrap();
  m.cartas = [
    Carta::new(4, "copa").unwrap(),
    Carta::new(10, "espada").unwrap(),
    Carta::new(7, "espada").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(true, got.0);
  assert_eq!(3, got.1);

  m.cartas = [
    Carta::new(1, "oro").unwrap(),
    Carta::new(2, "oro").unwrap(),
    Carta::new(3, "oro").unwrap(),
  ];
  let got = m.tiene_flor(&muestra);
  assert_eq!(true, got.0);
  assert_eq!(2, got.1);
}

#[test]
fn calc_flor_test() {
  let muestra = Carta::new(11, "oro").unwrap();
  let m = Manojo::new(
    Jugador { id: String::from("Alice"), equipo: Equipo::Azul },
    [
      Carta::new(2, "oro").unwrap(),
      Carta::new(4, "oro").unwrap(),
      Carta::new(5, "oro").unwrap(),
    ]
  );
  let got = m.tiene_flor(&muestra);
  assert_eq!(true, got.0);
  assert_eq!(47, m.calc_flor(&muestra));
}

#[test]
fn calc_envido_test() {
  let muestra = Carta::new(1, "espada").unwrap();
  let mut m = Manojo::new(
    Jugador { id: String::from("Alice"), equipo: Equipo::Azul },
    [
      Carta::new(6, "oro").unwrap(),
      Carta::new(12, "oro").unwrap(),
      Carta::new(5, "copa").unwrap(),
    ]
  );
  assert_eq!(26, m.calcular_envido(&muestra));

  m.cartas = [
    Carta::new(12, "copa").unwrap(),
    Carta::new(11, "copa").unwrap(),
    Carta::new(3, "basto").unwrap(),
  ];
  assert_eq!(20, m.calcular_envido(&muestra));

  m.cartas = [
    Carta::new(2, "copa").unwrap(),
    Carta::new(6, "copa").unwrap(),
    Carta::new(1, "basto").unwrap(),
  ];
  assert_eq!(28, m.calcular_envido(&muestra));

  m.cartas = [
    Carta::new(2, "oro").unwrap(),
    Carta::new(3, "oro").unwrap(),
    Carta::new(2, "basto").unwrap(),
  ];
  assert_eq!(25, m.calcular_envido(&muestra));

  m.cartas = [
    Carta::new(6, "basto").unwrap(),
    Carta::new(7, "basto").unwrap(),
    Carta::new(5, "oro").unwrap(),
  ];
  assert_eq!(33, m.calcular_envido(&muestra));

  m.cartas = [
    Carta::new(3, "copa").unwrap(),
    Carta::new(4, "copa").unwrap(),
    Carta::new(4, "oro").unwrap(),
  ];
  assert_eq!(27, m.calcular_envido(&muestra));
}