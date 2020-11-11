mod registers;

use registers::Registers;
use std::fs;
use std::io;

fn prompt_and_load() -> Vec<u8>{

    println!("Choose a .hex file to load into ram:");

    let mut files: Vec<fs::DirEntry> = fs::read_dir("./").unwrap().map(|n| n.unwrap()).collect();

    for i in 0..files.len() {
        println!("{}: {}", i, files[i].path().to_str().unwrap());
    }

    let mut input = String::new();

    io::stdin().read_line(&mut input);

    let index = input.trim().parse::<usize>().unwrap();

    return fs::read(files.get(index).unwrap().path()).unwrap();
}

fn main() {
    let mut reg = Registers::new();

    let mut ram = prompt_and_load();

    loop {
        let instruction = ((ram[reg.get_ip() as usize]as u16) << 8) |
            ram[(reg.get_ip()+1) as usize] as u16;
        let (opcode, args) = parse_instruction(instruction);

        match opcode{
            0x0=> reg.write(args[0], reg.read(args[1])),
            0x1=> reg.set_ip((reg.read(args[0]) as u32) << 16  | (reg.read(args[1])as u32)),
            0x2=> reg.write(1, do_math_op(args[0], args[1], args[2])),
            // 0x3=>,
            // 0x4=>,
            // 0x5=>,
            // 0x6=>,
            // 0x7=>,
            // 0x8=>,
            // 0x9=>,
            // 0xA=>,
            // 0xB=>,
            // 0xC => null(),
            // 0xD=>,
            // 0xE=>,
            // 0xF=>,
            _=>(panic!("Invalid opcode! (how did you do this?)"))
        }
        


    }
}

//parse the instruction and return a vector of arguments and the opcode.
fn parse_instruction(ins: u16) -> (u8, Vec<u8>){
    let opcode = extract_bits(ins, 12, 4) as u8;

    let mut args:Vec<u8>;

    match opcode {

        0x0 | 0x1 | 0x4 | 0x5 | 0xA => {args = vec![extract_bits(ins, 6, 6) as u8,
                                                    extract_bits(ins, 0, 6) as u8]}, //REG
        0x2 => args = vec![extract_bits(ins, 8, 4) as u8,
                           extract_bits(ins, 2, 6) as u8], //MTR
        0x3 => args = vec![extract_bits(ins, 8, 4) as u8,
                           extract_bits(ins, 0, 8) as u8], //MTI
        0x6 => args = vec![extract_bits(ins, 6, 6) as u8,
                           extract_bits(ins, 0, 6) as u8], //INT
        0x8 | 0x9 => args = vec![extract_bits(ins, 6, 6) as u8], //SRG

        0x7 | 0xB | 0xC | 0xD | 0xE | 0xF => args = vec![], //NUL

        _ => panic!("Invalid opcode! (how did you do this?)")
    }

    return (opcode, args);

}

fn extract_bits(num:u16, start:u8, len:u8) -> u16{ //Starts from LSB!
    return ((num) & (((1<<len)-1)<<start))>>start;
}

//Does ALU operation based on opcode and two operands
fn alu_op(op:u8, a:i16, b:i16) -> i16{
    match op { }
}

#[cfg(test)]
mod test {
    use std::panic;
    use crate::{parse_instruction, extract_bits, alu_op};

    #[test]
    fn bit_extraction() {
        assert_eq!(extract_bits(0, 1, 10), 0b0);
        assert_eq!(extract_bits(0b10101010, 1, 2), 0b1);
        assert_eq!(extract_bits(0b10101010, 2, 3), 0b10);
        assert_eq!(extract_bits(0b10101010, 1, 4), 0b0101);
    }

    #[test]
    fn alu_ops() {
        //add
        assert_eq!(alu_op(0x0, 1, 1), 1+1);

        //sub
        assert_eq!(alu_op(0x1, 1, 4), 1-4);

        //mul
        assert_eq!(alu_op(0x2, 6, 2), 6*2);

        //div
        assert_eq!(alu_op(0x3, 10, 2), 10/2);
        assert_eq!(alu_op(0x3, 10, 3), 10/3);
        assert_eq!(alu_op(0x3, 8, 6), 8/6);
        assert_eq!(alu_op(0x3, 10, 6), 10/6);
        assert_eq!(alu_op(0x3, 1, 0xFFFF), 1/0xFFFF);
        assert_eq!(alu_op(0x3, 0xFFFF, 0xFFFE), 0xFFFF/0xFFFE);

        //exp
        assert_eq!(alu_op(0x4, 1, 1), 1**1);
        assert_eq!(alu_op(0x4, 1, 2), 1**2);
        assert_eq!(alu_op(0x4, 0, 9), 0**9);
        assert_eq!(alu_op(0x4, 3, 2), 3**2);

        //mod
        assert_eq!(alu_op(0x5, 4, 5 ), 4%5);
        assert_eq!(alu_op(0x5, 8, 2 ), 8%2);
        assert_eq!(alu_op(0x5, 0, 0 ), 0%0);
        assert_eq!(alu_op(0x5, 0, 1 ), 0%1);

        //and
        assert_eq!(alu_op(0x6, 1, 1), 1&1);
        assert_eq!(alu_op(0x6, 0, 0), 0&0);
        assert_eq!(alu_op(0x6, 8, 25), 8&25);

        //or
        assert_eq!(alu_op(0x7, 1, 1), 1|1);
        assert_eq!(alu_op(0x7, 0, 0), 0|0);
        assert_eq!(alu_op(0x7, 7, 28), 7|28);

        //xor
        assert_eq!(alu_op(0x8, 1, 1), 1^1);
        assert_eq!(alu_op(0x8, 1, 1), 0^0);
        assert_eq!(alu_op(0x8, 30, 200), 30^200);

        //not
        assert_eq!(alu_op(0x9, 0, 67), !0);
        assert_eq!(alu_op(0x9, 1, 5), !1);
        assert_eq!(alu_op(0x9, 9, 1),!9 );

        //<<
        assert_eq!(alu_op(0xA, 1, 1), 1<<1);
        assert_eq!(alu_op(0xA, 0, 4), 0<<4);
        assert_eq!(alu_op(0xA, 12, 8), 12<<8);

        //>>
        assert_eq!(alu_op(0xB, 1, 1), 1>>1);
        assert_eq!(alu_op(0xB, 0, 9), 0>>9);
        assert_eq!(alu_op(0xB, 12, 2), 12>>2);
    }

    //TODO test overflow

    #[test]
    fn instruction_parsing() {
        assert_eq!(parse_instruction(0b0000000000000000),(0, vec![0,0])); //All Zero
        assert_eq!(parse_instruction(0b0010101000010001),(2, vec![10,4])); //MTR
        assert_eq!(parse_instruction(0b0011010000010001),(3, vec![4,17])); //MTI
        assert_eq!(parse_instruction(0b0001000001000001),(1, vec![1,1])); //REG
        assert_eq!(parse_instruction(0b0110000101000001),(6, vec![5,1])); //INT
        assert_eq!(parse_instruction(0b0111010000010011),(7, vec![])); //NUL
        assert_eq!(parse_instruction(0b1001010000001010),(9, vec![16])); //SRG

    }
}