use std::panic::UnwindSafe;

pub struct Registers {
	ip: u16,
	acc: i16,
	mem: i16,
	inn: i16,
	g_reg: Vec<i16>,
	p_reg: Vec<i16>,
}

impl Registers {
	pub fn new() -> Registers {
		Registers {
			ip: 0,
			acc: 0,
			mem: 0,
			inn: 0,
			g_reg: vec![0; 32],
			p_reg: vec![0; 28],
		}
	}

	pub fn write(&mut self, register: u8, value: i16) {
		match register {
			0 => return,
			1 => self.acc = value,
			2 => self.mem = value,
			3 => self.inn = value,
			4..=34 => self.g_reg[(register - 4) as usize] = value,
			35..=63 => self.p_reg[(register - 36) as usize] = value,
			_ => panic!("Invalid register!"),
		}
	}

	pub fn read(&self, register: u8) -> i16 {
		match register {
			0 => return 0,
			1 => return self.acc,
			2 => return self.mem,
			3 => return self.inn,
			4..=34 => return self.g_reg[(register - 4) as usize],
			35..=63 => return self.p_reg[(register - 36) as usize],
			_ => panic!("Invalid register!"),
		}
	}

	pub fn increment_ip(&mut self) {
		self.ip += 1;
	}

	pub fn increment_ip_by_n(&mut self, n: u16) {
		self.ip += n;
	}

	pub fn set_ip(&mut self, n: u16) {
		self.ip = n;
	}

	pub fn get_ip(&self) -> u16 {
		return self.ip;
	}
}

#[cfg(test)]
mod test {
	use registers::Registers;
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
}
