use truco::{*};

fn main() {
  let mut p = Partida::new(
    20,
    vec!["alice".to_string()],
    vec!["bob".to_string()],
    false,
  ).unwrap();

  let mut c = 0;
  while !p.terminada() {
    let a = random_action(&p, false);
    let _ = a.hacer(&mut p);
    c += 1;
  }

  println!("termino luego de {} acciones aleatorias", c);
}