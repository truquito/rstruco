use truco::*;

#[test]
fn partida_test(){
  println!("sakdjnaskjd");
  let p = Partida::new(
    20,
    vec!["alice".to_string()],
    vec!["bob".to_string()],
    true,
  ).unwrap();
  println!("--> {:?}", p.puntajes);
}