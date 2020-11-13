
pub struct Registers {
	ip: Vec<u32>,
	acc: Vec<i16>,
	mem: Vec<i16>,
	inn: u16,
	g_reg: Vec<Vec<i16>>,
	p_reg: Vec<Vec<i16>>,
	p_stack_frames: Vec<u8>,
	stack_frame: u8
}

impl Registers {
	pub fn new() -> Registers {
		Registers {
			ip: vec![0; 256],
			acc: vec![0; 256],
			mem: vec![0; 256],
			inn: 0,
			g_reg: vec![vec![0; 32]; 256],
			p_reg: vec![vec![0; 28]; 256],
			p_stack_frames: vec![0; 28],
			stack_frame: 0
		}
	}

	pub fn write(&mut self, register: u8, value: i16) {
		match register {
			0 => return,
			1 => self.acc[self.stack_frame as usize] = value,
			2 => self.mem[self.stack_frame as usize] = value,
			3 => self.inn = value as u16,
			4..=35 => self.g_reg[self.stack_frame as usize][(register - 4) as usize] = value,
			36..=63 => self.p_reg[self.p_stack_frames[(register - 36) as usize] as usize]
				[(register - 36) as usize] = value,
			_ => panic!("Invalid register!"),
		}
	}

	pub fn read(&self, register: u8) -> i16 {
		match register {
			0 => return 0,
			1 => return self.acc[self.stack_frame as usize],
			2 => return self.mem[self.stack_frame as usize],
			3 => return self.inn as i16,
			4..=35 => return self.g_reg[self.stack_frame as usize][(register - 4) as usize],
			36..=63 => return self.p_reg[self.p_stack_frames[(register - 36) as usize] as usize]
				[(register - 36) as usize],
			_ => panic!("Invalid register!"),
		}
	}

	pub fn push_reg(&mut self, register: u8){
		if register >= 36 && register <= 63{
			self.p_stack_frames[register as usize] += 1;

			//clear out frame for use
			self.p_reg[self.p_stack_frames[register as usize] as usize][register as usize] = 0;
		}
	}

	pub fn pop_reg(&mut self, register: u8){
		if register >= 36 && register <= 63 { self.p_stack_frames[register as usize] -= 1; }
	}

	pub fn push_frame(&mut self){
		self.stack_frame += 1;

		//clear out frame for use
		self.acc[self.stack_frame as usize] = 0;
		self.mem[self.stack_frame as usize] = 0;
		self.g_reg[self.stack_frame as usize] = vec![0; 32];
		self.ip[self.stack_frame as usize] = 0;
	}

	pub fn pop_frame(&mut self){ self.stack_frame -= 1; }

	pub fn increment_ip(&mut self) { self.ip[self.stack_frame as usize] += 1; }

	pub fn increment_ip_by_n(&mut self, n: u32) { self.ip[self.stack_frame as usize] += n; }

	pub fn set_ip(&mut self, n: u32) {
		self.ip[self.stack_frame as usize] = n;
	}

	pub fn get_ip(&self) -> u32 {
		return self.ip[self.stack_frame as usize];
	}
}

#[cfg(test)]
mod test {
	use crate::Registers;
	use std::panic;

	#[test]
	fn set_and_read() {
		let mut r = Registers::new();
		r.write(0, 5);
		r.write(1, 10);
		r.write(2, 15);
		r.write(3, 9);
		r.write(4, 25);
		r.write(10, 20);
		r.write(56, 2);

		assert_eq!(r.read(0), 0);
		assert_eq!(r.read(1), 10);
		assert_eq!(r.read(2), 15);
		assert_eq!(r.read(3), 9);
		assert_eq!(r.read(4), 25);
		assert_eq!(r.read(10), 20);
		assert_eq!(r.read(56), 2);
		assert_eq!(r.read(34), 0);
	}

	#[test]
	#[should_panic]
	fn test_read_bounds() {
		let mut r = Registers::new();
		r.read(64);
	}

	#[test]
	#[should_panic]
	fn test_write_bounds() {
		let mut r = Registers::new();
		r.write(64, 2);
	}

	#[test]
	fn instruction_pointer() {
		let mut r = Registers::new();

		assert_eq!(r.get_ip(), 0);

		r.set_ip(1234);
		assert_eq!(r.get_ip(), 1234);

		r.increment_ip();
		assert_eq!(r.get_ip(), 1235);

		r.increment_ip_by_n(5);
		assert_eq!(r.get_ip(), 1240);

		r.set_ip(12);
		assert_eq!(r.get_ip(), 12);
	}

	#[test]
	fn main_stack() {
		let mut r = Registers::new();

		r.set_ip(12);
		r.write(26, 12);
		r.write(4, 6);
		r.write(35, 3);

		r.push_frame();

		assert_eq!(r.get_ip(), 0);
		assert_eq!(r.read(26), 0);
		assert_eq!(r.read(4), 0);
		assert_eq!(r.read(35), 0);

		r.pop_frame();

		assert_eq!(r.get_ip(), 12);
		assert_eq!(r.read(26), 12);
		assert_eq!(r.read(4), 6);
		assert_eq!(r.read(35), 3);
	}

	#[test]
	fn reg_stacks(){

	}
}
