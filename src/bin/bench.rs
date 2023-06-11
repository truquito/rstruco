use truco::{*};
use std::time::{Duration, Instant};

fn main() {
  let timeout = Duration::from_secs(60 * 10);
  let start_time = Instant::now();
  let mut c: usize = 0;

  while start_time.elapsed() < timeout {
    let mut p = Partida::new(
      20,
      vec!["alice".to_string()],
      vec!["bob".to_string()],
      false,
    ).unwrap();
  
    while !p.terminada() {
      let a = random_action(&p, false);
      let _ = a.hacer(&mut p);
    }

    c += 1;
  }

  println!("timeout: {:?} -> count: {}", timeout, c);
}