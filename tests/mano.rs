use truco::*;

#[test]
fn mano_ord_test(){
  let p = NumMano::Primera;
  let s = NumMano::Segunda;
  // let t = Mano::Primera;
  assert!(p < s && s == NumMano::Segunda && s < NumMano::Tercera)
}