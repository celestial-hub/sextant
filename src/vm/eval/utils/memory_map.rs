use celestial_hub_astrolabe::{
  ast::Variable,
  lexer::tokens::{Type, Value},
};

pub trait Memory {
  fn size(&self) -> usize;
  fn to_bytes(&self) -> Vec<u8>;
}

impl Memory for Variable {
  fn size(&self) -> usize {
    match self.type_ {
      Type::Asciiz => {
        let Value::String(s) = &self.value;
        s.len()
      }
    }
  }

  fn to_bytes(&self) -> Vec<u8> {
    match &self.value {
      Value::String(s) => s.as_bytes().to_vec(),
    }
  }
}
