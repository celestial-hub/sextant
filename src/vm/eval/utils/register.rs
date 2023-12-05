use celestial_hub_astrolabe::ast::Register;
use phf::phf_map;

static COUNTRIES: phf::Map<&'static str, i32> = phf_map! {
    "$zero" => 0,
    "$at" => 1,
    "$v0" => 2,
    "$v1" => 3,
    "$a0" => 4,
    "$a1" => 5,
    "$a2" => 6,
    "$a3" => 7,
    "$t0" => 8,
    "$t1" => 9,
    "$t2" => 10,
    "$t3" => 11,
    "$t4" => 12,
    "$t5" => 13,
    "$t6" => 14,
    "$t7" => 15,
    "$s0" => 16,
    "$s1" => 17,
    "$s2" => 18,
    "$s3" => 19,
    "$s4" => 20,
    "$s5" => 21,
    "$s6" => 22,
    "$s7" => 23,
    "$t8" => 24,
    "$t9" => 25,
    "$k0" => 26,
    "$k1" => 27,
    "$gp" => 28,
    "$sp" => 29,
    "$s8" => 30,
    "$ra" => 31,
};

pub fn get_register_position(register_name: &str) -> anyhow::Result<usize> {
  let register_position = COUNTRIES
    .get(register_name)
    .ok_or_else(|| anyhow::anyhow!("register not found"))?;

  Ok(*register_position as usize)
}

pub trait RegisterPosition {
  fn get_register_position(&self) -> anyhow::Result<usize>;
}

impl RegisterPosition for Register {
  fn get_register_position(&self) -> anyhow::Result<usize> {
    let register_name = self.name.clone();

    get_register_position(&register_name)
  }
}
