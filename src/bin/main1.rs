fn main() {
  let p = truco::Partida::new(
    20,
    vec!["alice".to_string(), "anna".to_string()],
    vec!["bob".to_string(), "ben".to_string()],
    true,
  );
  println!("main1 {:?}!!!!", p);
}