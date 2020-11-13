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

        let mut ip_inc = 1;

        match opcode{
            0x0 => reg.write(args[0], reg.read(args[1])),
            0x1 => {
                reg.set_ip(u16_to_u32(reg.read(args[0]) as u16,reg.read(args[1]) as u16));
                ip_inc = 0;
            },
            0x2 => reg.write(1, alu_op(args[0], reg.read(1), reg.read(args[1]))),
            0x3 => reg.write(1, alu_op(args[0], reg.read(1), args[1] as i16)),
            0x4 => reg.write(2, read_word(&ram, u16_to_u32(args[0] as u16, args[1] as u16))as i16), //TODO: convert to fn and add memory mapped IO
            0x5 => { //TODO: convert to fn and add memory mapped IO
                let adr = 2*u16_to_u32(reg.read(args[0]) as u16, reg.read(args[1]) as u16)as usize;
                ram[adr] = (((reg.read(2) as u16)&0xFF00)>>8) as u8;
                ram[adr+1] = ((reg.read(2)as u16)&0x00FF) as u8;
            },
            // 0x6 =>,
            // 0x7 =>,
            // 0x8 =>,
            // 0x9 =>,
            // 0xA =>,
            // 0xB =>,
            0xC => (),
            0xD => if reg.read(1) == 0 {ip_inc = 2},
            0xE => if reg.read(1) < 0 {ip_inc = 2},
            0xF => if reg.read(1) > 0 {ip_inc = 2},
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
    return match op {
        0x0 => a+b,
        0x1 => a-b,
        0x2 => a*b,
        0x3 => a/b,
        0x4 => a.pow(b as u32),
        0x5 => a%b,
        0x6 => a&b,
        0x7 => a|b,
        0x8 => a^b,
        0x9 => !a,
        0xA => a<<b,
        0xB => a>>b,
        _ => 0

    }
}

//Self explanatory utility fns.
fn u16_to_u32(a:u16, b:u16) -> u32{return ((a as u32) << 16) | (b as u32)}
fn u8_to_u16(a:u8, b:u8) -> u16{return ((a as u16) << 8) | (b as u16)}
fn read_word(arr: &Vec<u8>, address: u32) -> u16{
    return u8_to_u16(arr[(address as usize)*2], arr[(address as usize)*2+1])
}

#[cfg(test)]
mod test {
    use std::panic;
    use crate::{parse_instruction, extract_bits, alu_op, u16_to_u32, u8_to_u16, read_word};

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
        assert_eq!(alu_op(0x3, 1, 126), 1/126);
        assert_eq!(alu_op(0x3, 126, 125), 126/125);

        //exp
        assert_eq!(alu_op(0x4, 1, 1), 1_i16.pow(1));
        assert_eq!(alu_op(0x4, 1, 2), 1_i16.pow(2));
        assert_eq!(alu_op(0x4, 0, 9), 0_i16.pow(9));
        assert_eq!(alu_op(0x4, 3, 2), 3_i16.pow(2));

        //mod
        assert_eq!(alu_op(0x5, 4, 5 ), 4%5);
        assert_eq!(alu_op(0x5, 8, 2 ), 8%2);
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
        assert_eq!(alu_op(0x8, 30, 126), 30^126);

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

    #[test]
    fn int_combinations(){
        assert_eq!(u16_to_u32(0x0F0F, 0x0A0A), 0x0F0F0A0A);
        assert_eq!(u8_to_u16(0x0F, 0xA0), 0x0FA0);
    }

    #[test]
    fn read_word_from_arr(){
        let mut dummy_data = vec![0x11, 0x25, 0x32, 0x44, 0x55, 0x23, 0x12, 0xAB, 0x54, 0xCD];
        assert_eq!(read_word(&dummy_data, 3), 0x12AB);
        assert_eq!(read_word(&dummy_data, 2), 0x5523);
        assert_eq!(read_word(&dummy_data, 0), 0x1125);
    }
}