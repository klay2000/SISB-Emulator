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

    reg.write(1, 12);

    let mut ram = prompt_and_load();

    println!("{}", String::from_utf8_lossy(&ram));
}
