#[test]
fn it_does() {
  assert_eq!(1+1,2);
  // assert_eq!(1+1,3);
}

#[test]
fn it_works() {
  let result = truco::add(2, 2);
  assert_eq!(result, 4);
}

#[test]
fn json_test() {
  let json = r#"
{
  "article": "how to work with json in Rust",
  "author": "tdep",
  "paragraph": [
    {
      "name": "untyped!"
    },
    {
      "name": "strongly typed"
    },
    {
      "name": "writing json"
    }
  ]
}
"#;

  let parsed: truco::article::Article = serde_json::from_str(json).unwrap();
  println!("The name of the first paragraph is: {}", parsed.paragraph[0].name);
  println!("The name of the snd paragraph is: {}", parsed.paragraph[1].name);
  println!("The name of the thrd paragraph is: {}", parsed.paragraph[2].name);
}