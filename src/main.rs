use celestial_hub_astrolabe::{lexer::Lexer, parser::Parser};

use celestial_hub_sextant::vm::VM;

fn main() {
  let source_code = r#"
  .data
prompt: .asciiz "The sum of is: "

	.text
	.global main
main:
  ; Read number 1
  li $v0, 5
  syscall
  move $t0, $v0

  ; Read number 2
  li $v0, 5
  syscall
  move $t1, $v0

  ; Add numbers
	add $t2, $t0, $t1

  ; Print prompt
  li $v0, 4
  la $a0, prompt
  syscall

  ; Print added value
  li $v0, 1
  move $a0, $t2
  syscall

  ; Exit
  li $v0, 0xA
  syscall
    "#;

  let ast = Parser::new()
    .parse(Lexer::new(source_code, "test_eval"))
    .expect("parse failed");

  println!("{:#?}", ast);

  let mut vm = VM::new(Default::default()).expect("vm init failed");

  vm.eval(ast).expect("eval failed");
}
