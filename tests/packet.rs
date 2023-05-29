use truco::*;

#[test]
fn message_json_test(){
  let m = enco::Message(
    enco::Content::Error{ msg: String::from("teletubbies") }
  );
  let _json = serde_json::to_string(&m).unwrap();
  println!("the json is: {}", _json);

  let m = enco::Message(
    enco::Content::LaManoResultaParda{}
  );
  let _json = serde_json::to_string(&m).unwrap();
  println!("the json is: {}", _json);
}