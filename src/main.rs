mod registers;
use registers::Registers;

fn main() {
    let mut reg = Registers::new();

    reg.write(1, 12);

    println!("{}", reg.read(1));
}
