mod registers;

use registers::Registers;
use std::fs;
use std::io;
use std::io::stdin;
use std::char::decode_utf16;
use std::borrow::Borrow;

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

    let mut sleeping = false;

    let mut interrupts:Vec<i8> = vec![];
    let mut isr_fn_addrs:Vec<isize> = vec![-1; 64];

    loop {

        for i in interrupts.iter(){
            if isr_fn_addrs[*i as usize] != -1 {
                if reg.push_frame() == false { panic!("Stuck in stack overflow!"); }
                reg.set_ip(isr_fn_addrs[*i as usize] as u32);
            }
        }

        if !sleeping {
            let instruction = read_word(&ram, reg.get_ip());

            let (opcode, args) = parse_instruction(instruction);

            let mut ip_inc = 1; // The amount ip will be incremented after instruction ends.

            match opcode {
                0x0 => reg.write(args[0], reg.read(args[1])), //mov
                0x1 => { //jmp
                    reg.set_ip(u16_to_u32(reg.read(args[0]) as u16, reg.read(args[1]) as u16));
                    ip_inc = 0;
                },
                0x2 => { //mtr
                    let (r, o) = alu_op(args[0], reg.read(1), reg.read(args[1]));
                    if o { interrupts.push(0); }

                    reg.write(1, r);
                },
                0x3 => { //mti
                    let (r, o) = alu_op(args[0], reg.read(1), args[1] as i16);
                    if o { interrupts.push(0); }

                    reg.write(1, r);
                },
                0x4 => { //read
                    reg.write(2, read_word(&ram, u16_to_u32(args[0] as u16, args[1] as u16)) as i16)
                },
                0x5 => { //store
                    let adr = 2 * u16_to_u32(reg.read(args[0]) as u16, reg.read(args[1]) as u16) as usize;
                    if adr == 0 {
                        print!("{}", decode_utf16(vec![reg.read(2) as u16])
                        .map(|r| r.map_err(|e| e.unpaired_surrogate()))
                        .collect::<Vec<_>>()[0].unwrap()) // Yep, this is what it takes to decode a char of UTF-16 in Rust.
                    }
                    else {
                        ram[adr] = (((reg.read(2) as u16) & 0xFF00) >> 8) as u8;
                        ram[adr + 1] = ((reg.read(2) as u16) & 0x00FF) as u8;
                    }
                },
                0x6 =>{ //regisr
                    isr_fn_addrs[args[0] as usize] = reg.read(args[1] as u8) as isize;
                },
                0x7 => {
                    sleeping = true;
                    println!("\nCPU sleeping.");
                }, //slp
                0x8 => if !reg.push_reg(args[0]) {interrupts.push(1)}, //sav
                0x9 => if !reg.pop_reg(args[0]) {interrupts.push(1)}, //res
                0xA => if !reg.push_frame() {interrupts.push(1)}, //push
                0xB => if !reg.pop_frame() {interrupts.push(1)}, //pop
                0xC => (), //noop
                0xD => if reg.read(1) == 0 { ip_inc = 2 }, //skpeq
                0xE => if reg.read(1) < 0 { ip_inc = 2 }, //skplt
                0xF => if reg.read(1) > 0 { ip_inc = 2 }, //skpmt
                _ => (panic!("Invalid opcode! (how did you manage to do this?)"))
            }

            reg.increment_ip_by_n(ip_inc);

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

        _ => panic!("Invalid opcode! (how did you manage to do this?)")
    }

    return (opcode, args);

}

fn extract_bits(num:u16, start:u8, len:u8) -> u16{ //Starts from LSB!
    return ((num) & (((1<<len)-1)<<start))>>start;
}

//Does ALU operation based on opcode and two operands
fn alu_op(op:u8, a:i16, b:i16) -> (i16, bool) {
    return match op {
        0x0 => a.overflowing_add(b),
        0x1 => a.overflowing_sub(b),
        0x2 => a.overflowing_mul(b),
        0x3 => a.overflowing_div(b),
        0x4 => a.overflowing_pow(b as u32),
        0x5 => a.overflowing_rem(b),
        0x6 => (a&b, false),
        0x7 => (a|b, false),
        0x8 => (a^b, false),
        0x9 => (!a, false),
        0xA => (a<<b, false),
        0xB => (a>>b, false),
        _ => (0, false)

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
        assert_eq!(alu_op(0x0, 1, 1), (1+1, false));
        assert_eq!(alu_op(0x0, i16::MAX, 1), (i16::MIN, true));

        //sub
        assert_eq!(alu_op(0x1, 1, 4), (1-4, false));

        //mul
        assert_eq!(alu_op(0x2, 6, 2), (6*2, false));

        //div
        assert_eq!(alu_op(0x3, 10, 2), (10/2, false));
        assert_eq!(alu_op(0x3, 10, 3), (10/3, false));
        assert_eq!(alu_op(0x3, 8, 6), (8/6, false));
        assert_eq!(alu_op(0x3, 10, 6), (10/6, false));
        assert_eq!(alu_op(0x3, 1, 126), (1/126, false));
        assert_eq!(alu_op(0x3, 126, 125), (126/125, false));

        //exp
        assert_eq!(alu_op(0x4, 1, 1), (1_i16.pow(1), false));
        assert_eq!(alu_op(0x4, 1, 2), (1_i16.pow(2), false));
        assert_eq!(alu_op(0x4, 0, 9), (0_i16.pow(9), false));
        assert_eq!(alu_op(0x4, 3, 2), (3_i16.pow(2), false));

        //mod
        assert_eq!(alu_op(0x5, 4, 5 ), (4%5, false));
        assert_eq!(alu_op(0x5, 8, 2 ), (8%2, false));
        assert_eq!(alu_op(0x5, 0, 1 ), (0%1, false));

        //and
        assert_eq!(alu_op(0x6, 1, 1), (1&1, false));
        assert_eq!(alu_op(0x6, 0, 0), (0&0, false));
        assert_eq!(alu_op(0x6, 8, 25), (8&25, false));

        //or
        assert_eq!(alu_op(0x7, 1, 1), (1|1, false));
        assert_eq!(alu_op(0x7, 0, 0), (0|0, false));
        assert_eq!(alu_op(0x7, 7, 28), (7|28, false));

        //xor
        assert_eq!(alu_op(0x8, 1, 1), (1^1, false));
        assert_eq!(alu_op(0x8, 1, 1), (0^0, false));
        assert_eq!(alu_op(0x8, 30, 126), (30^126, false));

        //not
        assert_eq!(alu_op(0x9, 0, 67), (!0, false));
        assert_eq!(alu_op(0x9, 1, 5), (!1, false));
        assert_eq!(alu_op(0x9, 9, 1),(!9, false));

        //<<
        assert_eq!(alu_op(0xA, 1, 1), (1<<1, false));
        assert_eq!(alu_op(0xA, 0, 4), (0<<4, false));
        assert_eq!(alu_op(0xA, 12, 8), (12<<8, false));

        //>>
        assert_eq!(alu_op(0xB, 1, 1), (1>>1, false));
        assert_eq!(alu_op(0xB, 0, 9), (0>>9, false));
        assert_eq!(alu_op(0xB, 12, 2), (12>>2, false));

    }

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