use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Paragraph {
  pub name: String
}

#[derive(Serialize, Deserialize)]
pub struct Article {
  pub article: String,
  pub author: String,
  pub paragraph: Vec<Paragraph>
}