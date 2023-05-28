use truco::*;

#[test]
fn packet_test(){
  let p = enco::Packet{
    destination: vec![String::from("foo")],
    message: String::from("bar"),
  };
  println!("the Packet is: {:?}", p);
}