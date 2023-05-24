use std::error::Error;
use truco::*;

#[test]
fn enum_palo_test() {
  let p = Palo::Espada;
  assert!(p.to_int() == 2);
  let p = Palo::Oro;
  assert!(p.to_int() == 3);
}

#[test]
fn parse_palo_test() {
  assert_eq!(Palo::parse("espada").unwrap(), Palo::Espada);
  assert_eq!(Palo::parse("COPA").unwrap(), Palo::Copa);
}

#[test]
#[should_panic]
fn invalid_parse_palo_test() {
  assert_eq!(Palo::parse("asfasf").unwrap(), Palo::Espada);
}

#[test]
fn carta_constructor_test() -> Result<(), Box<dyn Error>> {
  let c1 = Carta::new(5, "espada")?;
  assert_eq!("Carta { valor: 5, palo: Espada }", format!("{:?}", c1));
  assert_eq!("5 de espada", c1.to_string());
  Ok(())
}

#[test]
fn invalid_carta_valor_test() {
  let res = 
    std::panic::catch_unwind(|| {
      Carta::new(9, "espada").unwrap()
    });
  assert!(res.is_err());

  let res = 
    std::panic::catch_unwind(|| {
      Carta::new(8, "espada").unwrap()
    });
  assert!(res.is_err());

  let res = 
    std::panic::catch_unwind(|| {
      Carta::new(12, "jaskdjkajskd").unwrap()
    });
  assert!(res.is_err());
}

/*
 Barajas; orden absoluto:
 ----------------------------------------------------------
| ID	| Carta	    ID | Carta	  ID | Carta	  ID | Carta |
|---------------------------------------------------------|
| 00 | 1,Basto   10 | 1,Copa   20 | 1,Espada   30 | 1,Oro |
| 01 | 2,Basto   11 | 2,Copa   21 | 2,Espada   31 | 2,Oro |
| 02 | 3,Basto   12 | 3,Copa   22 | 3,Espada   32 | 3,Oro |
| 03 | 4,Basto   13 | 4,Copa   23 | 4,Espada   33 | 4,Oro |
| 04 | 5,Basto   14 | 5,Copa   24 | 5,Espada   34 | 5,Oro |
| 05 | 6,Basto   15 | 6,Copa   25 | 6,Espada   35 | 6,Oro |
| 06 | 7,Basto   16 | 7,Copa   26 | 7,Espada   36 | 7,Oro |
 ----------------------------------------------------------
| 07 |10,Basto   17 |10,Copa   27 |10,Espada   37 |10,Oro |
| 08 |11,Basto   18 |11,Copa   28 |11,Espada   38 |11,Oro |
| 09 |12,Basto   19 |12,Copa   29 |12,Espada   39 |12,Oro |
 ----------------------------------------------------------
*/

#[test]
fn carta_id_test() {
  let c = Carta::new(5, "Basto").unwrap();
  assert_eq!(4, c.id());

  let c = Carta::new(11, "copa").unwrap();
  assert_eq!(18, c.id());

  let c = Carta::new(12, "espada").unwrap();
  assert_eq!(29, c.id());

  let c = Carta::new(5, "ORO").unwrap();
  assert_eq!(34, c.id());
}

#[test]
fn carta_puid_test() {
  let c = Carta::new(4, "espada").unwrap();
  assert_eq!(89, c.puid());
}

#[test]
fn es_num_pieza_test() {
  let c = Carta::new(4, "espada").unwrap();
  assert_eq!(true, c.es_numericamente_pieza());

  let c = Carta::new(1, "oro").unwrap();
  assert_eq!(false, c.es_numericamente_pieza());

  let c = Carta::new(10, "basto").unwrap();
  assert_eq!(true, c.es_numericamente_pieza());

  let c = Carta::new(6, "copa").unwrap();
  assert_eq!(false, c.es_numericamente_pieza());
}

use rand::{SeedableRng, Rng, rngs::StdRng};

#[test]
fn get_cartas_random_test() {
  let seed = 9998;
  let mut _rng: StdRng = SeedableRng::seed_from_u64(seed);
  println!("With seed {}, the first random u8 is: {}", seed, _rng.gen::<u8>());
  let cs = get_cartas_random(4);
  for c in cs {
    println!("{}", c)
  }
}
