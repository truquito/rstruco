use std::fmt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

// u8 ~ (0..=255)
pub const PRIMES: &'static [u8] = &[
  2, 3, 5, 7, 11, 13, 17, 19, 23, 29, // Basto
  31, 37, 41, 43, 47, 53, 59, 61, 67, 71, // Copa
  73, 79, 83, 89, 97, 101, 103, 107, 109, 113, // Espada
  127, 131, 137, 139, 149, 151, 157, 163, 167, 17 // Oro
];


#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Palo {
  Basto,
  Copa,
  Espada,
  Oro,
}

impl Palo {
  pub fn to_int(&self) -> u8 {
    match self {
      Palo::Basto  => 0,
      Palo::Copa   => 1,
      Palo::Espada => 2,
      Palo::Oro    => 3,
    }
  }

  pub fn parse(p: &str) -> Result<Palo, &str> {
    match p.to_lowercase().as_str() {
      "basto"  => Ok(Palo::Basto),
      "copa"   => Ok(Palo::Copa),
      "espada" => Ok(Palo::Espada),
      "oro"    => Ok(Palo::Oro),
      _ => Err("Palo invalido"),
    }
  }
}

impl fmt::Display for Palo {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Palo::Basto  => write!(f, "Basto"),
      Palo::Copa   => write!(f, "Copa"),
      Palo::Espada => write!(f, "Espada"),
      Palo::Oro    => write!(f, "Oro"),
    }
  }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Carta {
  pub valor: u8,
  pub palo: Palo,
}

impl Carta {
  // constructor
  pub fn new(valor: u8, palo: &str) -> Result<Carta, &str> {
    if (8..=9).contains(&valor) || !(1 <= valor && valor <= 12) {
      return Err("Valor invalido")
    }

    Ok(
      Carta{
        valor:valor,
        palo:Palo::parse(palo)?
      }
    )
  }

  pub fn id(&self) -> u8 {
    let mut id = 10 * self.palo.to_int();
    id += self.valor - 1;
    if self.valor >= 10 { id -= 2; }
    id 
  }

  pub fn puid(&self) -> u8 {
    return PRIMES[self.id() as usize]
  }

  pub fn es_numericamente_pieza(&self) -> bool {
    [2,4,5,10,11].contains(&self.valor)
  }

  pub fn es_pieza(&self, muestra: &Carta) -> bool {
    // caso 1
    let es_de_la_muestra = self.palo == muestra.palo;
    let es_pieza_caso_1 = self.es_numericamente_pieza() && es_de_la_muestra;
    // caso 2
    let es_doce = self.valor == 12;
    let es_pieza_caso_2 = es_doce && es_de_la_muestra &&
      muestra.es_numericamente_pieza();

    return es_pieza_caso_1 || es_pieza_caso_2;
  }

  pub fn es_mata(&self) -> bool {
    if [Palo::Espada, Palo::Basto].contains(&self.palo) && self.valor == 1 {
      return true;
    }

    if [Palo::Espada, Palo::Oro].contains(&self.palo) && self.valor == 7 {
      return true;
    }

    false
  }

  pub fn calc_puntaje(&self, muestra: &Carta) -> u8 {
    if self.es_pieza(muestra) {
      match self.valor {
        2       => 30,
        4       => 29,
        5       => 28,
        10 | 11 => 27,
        12 => {
          let vale_como = Carta{valor:muestra.valor, palo:self.palo};
          return vale_como.calc_puntaje(&muestra);
        }
        _ => unreachable!()
      }
    } else if self.es_mata() {
      return self.valor
    
    } else if self.valor <= 3 {
      return self.valor
    
    } else if 10 <= self.valor && self.valor <= 12 {
      return 0
    
    } else {
      return self.valor
    }
  }

  pub fn calc_poder(&self, muestra: &Carta) -> u8 {
    if self.es_pieza(muestra) {
      match self.valor {
        2  => 34,
        4  => 33,
        5  => 32,
        11 => 31,
        10 => 30,
        12 => {
          let vale_como = Carta{valor:muestra.valor, palo:self.palo};
          return vale_como.calc_puntaje(&muestra);
        },
        _ => unreachable!()
      }
    } else if self.es_mata() {
      // matas
      match (self.valor, self.palo) {
        (1, Palo::Espada) => 23,
        (1, Palo::Basto) => 22,
        (7, Palo::Espada) => 21,
        (7, Palo::Oro) => 20,
        _ => unreachable!()
      }
    } else {
      // chicas
      match self.valor {
        3 => 19,
        2 => 18,
        1 => 17,
        12 => 16,
        11 => 15,
        10 => 14,
        7 => 13,
        6 => 12,
        5 => 11,
        4 => 10,
        _ => unreachable!()
      }
    }
  }
}

impl fmt::Display for Carta {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} de {}", self.valor, self.palo.to_string().to_lowercase())
  }
}

fn carta_from_id(id: u8) -> Carta {
  // valor
  let ultimo_digito = id % 10;
  let valor = 
    if ultimo_digito <= 6 {ultimo_digito + 1}
    else {10 + ultimo_digito - 7};
  let palo = 
    if id <= 9 {Palo::Basto}
    else if 10 <= id && id <= 19 {Palo::Copa}
    else if 20 <= id && id <= 29 {Palo::Espada}
    else {Palo::Oro};
  
  Carta{ valor, palo }
}

pub fn get_cartas_random(n: u8) -> Vec<Carta> {
  let max_carta_id = 40;
  let mut indices: Vec<u8> = (0..max_carta_id).collect();
  let mut rng = rand::thread_rng();
  indices.shuffle(&mut rng);
  indices[0..n as usize]
    .iter()
    .map(|ix| carta_from_id(*ix))
    .collect::<Vec<Carta>>()
}
