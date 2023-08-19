// mod CPU;
// mod Bus;
// mod opcodes;

struct CPU<'a> {
    // byte: u8, word: u16
    a: u8,  // accumulator
    x: u8,  // X register
    y: u8,  // Y register
    sp: u8,   // stack pointer
    pc: u16,   // program counter
    sr: u8,   // status register
    fetched: u8,    // data that has been fetched by fetch()
    addr_abs: u16,  // memory address to read from (absolute)
    addr_rel: u16,  // memory address to read from (relative)
    opcode: u8,     // current opcode
    cycles: u8,     // cycles left to run
    bus: Option<&'a Bus<'a>>,
    lookup: Vec<&'a Instruction<'a>>,
}

impl<'a> CPU<'a> {
    // fn new(a: u8, x: u8, y: u8, sp: u8, pc: u16, sr: u8, fetched: u8, addr_abs: u16, addr_rel: u16, opcode: u8, cycles: u8) -> Self {
    //     return CPU {
    //         a,
    //         x,
    //         y,
    //         sp,
    //         pc,
    //         sr,
    //         fetched,
    //         addr_abs,
    //         addr_rel,
    //         opcode,
    //         cycles,
    //     };
    // }

    fn empty() -> Self {
        return CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x00,
            pc: 0x0000,
            sr: 0x00,
            fetched: 0x00,
            addr_abs: 0x0000,
            addr_rel: 0x00,
            opcode: 0x00,
            cycles: 0,
            bus: None,
            lookup: vec![   // TODO: create entire lookup table
                &Instruction { name: "BRK", operate: Some(&Self::BRK), addr_mode: &Self::IMM, cycles: 7 }
            ]
        };
    }

    fn connect_bus(&mut self, bus: &'a Bus<'a>) {
        self.bus = Some(bus);
    }

    // Write & read bus

    fn write(&self, addr: u16, data: u8) {
        self.bus.unwrap().write(addr, data);
    }

    fn read(&self, addr: u16) -> u8 {
        return self.bus.unwrap().read(addr, false);
    }

    // Flags

    fn get_flag(&self, f: Flags) -> u8 {
        0
    }

    fn set_flag(&mut self, f: Flags, v: bool) {
        if v {
            self.sr |= f;
        } else {
            self.sr &= !f;
        }
    }

    // Addressing modes
    fn IMP(&self) -> u8 {
        self.fetched = self.a;
        return 0;
    }
    fn ZP0(&self) -> u8 {
        self.addr_abs = self.read(self.pc).into();
        self.pc += 1;
        self.addr_abs &= 0x00FF;
        return 0;
    }
    fn ABS(&self) -> u8 {
        let lo: u16 = self.read(self.pc).into();
        self.pc += 1;
        let hi: u16 = self.read(self.pc).into();
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        return 0;
    }
    fn ABX(&self) -> u8 {
        let lo: u16 = self.read(self.pc).into();
        self.pc += 1;
        let hi: u16 = self.read(self.pc).into();
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += <u8 as Into<u16>>::into(self.x);

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }
    fn ABY(&self) -> u8 {
        let lo: u16 = self.read(self.pc).into();
        self.pc += 1;
        let hi: u16 = self.read(self.pc).into();
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += <u8 as Into<u16>>::intro(self.x);

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }
    fn IMM(&self) -> u8 {
        self.addr_abs = self.pc;    // pc++ in example, does this work?
        self.pc += 1;
        return 0;
    }
    fn ZPX(&self) -> u8 {
        self.addr_abs = (self.read(self.pc) + self.x).into();
        self.pc += 1;
        self.addr_abs &= 0x00FF;
        return 0;
    }
    fn ZPY(&self) -> u8 {
        self.addr_abs = (self.read(self.pc) + self.y).into();
        self.pc += 1;
        self.addr_abs &= 0x00FF;
        return 0;
    }
    fn REL() -> u8 { 0 }

    fn IND(&self) -> u8 {
        let ptr_lo: u16 = self.read(self.pc).into();
        self.pc += 1;
        let ptr_hi: u16 = self.read(self.pc).into();
        self.pc += 1;

        let ptr: u16 = (ptr_hi << 8) | ptr_lo;
    
        self.addr_abs = ((self.read(ptr + 1) << 8) | self.read(ptr + 0)).into();

        return 0;
    }

    fn IZX() -> u8 {

    }
    fn IZY() -> u8 { 0 }

    // Opcodes
    fn ADC() -> u8 { 0 }
    fn AND() -> u8 { 0 }
    fn ASL() -> u8 { 0 }
    fn BBR() -> u8 { 0 }
    fn BBS() -> u8 { 0 }
    fn BCC() -> u8 { 0 }
    fn BCS() -> u8 { 0 }
    fn BEQ() -> u8 { 0 }
    fn BIT() -> u8 { 0 }
    fn BMI() -> u8 { 0 }
    fn BNE() -> u8 { 0 }
    fn BPL() -> u8 { 0 }
    fn BRA() -> u8 { 0 }
    fn BRK(&self) -> u8 { 0 }
    fn BVC() -> u8 { 0 }
    fn BVS() -> u8 { 0 }
    fn CLC() -> u8 { 0 }
    fn CLD() -> u8 { 0 }
    fn CLI() -> u8 { 0 }
    fn CLV() -> u8 { 0 }
    fn CMP() -> u8 { 0 }
    fn CPX() -> u8 { 0 }
    fn CPY() -> u8 { 0 }
    fn DEC() -> u8 { 0 }
    fn DEX() -> u8 { 0 }
    fn EOR() -> u8 { 0 }
    fn INC() -> u8 { 0 }
    fn INX() -> u8 { 0 }
    fn INY() -> u8 { 0 }
    fn JMP() -> u8 { 0 }
    fn JSR() -> u8 { 0 }
    fn LDA() -> u8 { 0 }
    fn LDX() -> u8 { 0 }
    fn LDY() -> u8 { 0 }
    fn LSR() -> u8 { 0 }
    fn NOP() -> u8 { 0 }
    fn ORA() -> u8 { 0 }
    fn PHA() -> u8 { 0 }
    fn PHP() -> u8 { 0 }
    fn PHX() -> u8 { 0 }
    fn PHY() -> u8 { 0 }
    fn PLA() -> u8 { 0 }
    fn PLP() -> u8 { 0 }
    fn PLX() -> u8 { 0 }
    fn PLY() -> u8 { 0 }
    fn RMB() -> u8 { 0 }
    fn ROL() -> u8 { 0 }
    fn ROR() -> u8 { 0 }
    fn RTI() -> u8 { 0 }
    fn RTS() -> u8 { 0 }
    fn SBC() -> u8 { 0 }
    fn SEC() -> u8 { 0 }
    fn SED() -> u8 { 0 }
    fn SEI() -> u8 { 0 }
    fn SMB() -> u8 { 0 }
    fn STA() -> u8 { 0 }
    fn STP() -> u8 { 0 }
    fn STX() -> u8 { 0 }
    fn STY() -> u8 { 0 }
    fn STZ() -> u8 { 0 }
    fn TAX() -> u8 { 0 }
    fn TAY() -> u8 { 0 }
    fn TRB() -> u8 { 0 }
    fn TSB() -> u8 { 0 }
    fn TXA() -> u8 { 0 }
    fn TXS() -> u8 { 0 }
    fn TYA() -> u8 { 0 }
    fn WAI() -> u8 { 0 }
    fn XXX() -> u8 { 0 }

    // Clock
    fn clock(&mut self) {
        if self.cycles == 0 {
            self.opcode = self.read(self.pc);
            self.pc += 1;

            // TODO: Get Starting number of cycles
            self.cycles = self.lookup[self.opcode as usize].cycles;

            let additional_cycle1: u8 = (self.lookup[self.opcode as usize].addr_mode.unwrap())();
            let additional_cycle2: u8 = (self.lookup[self.opcode as usize].operate.unwrap())();

            self.cycles += additional_cycle1 & additional_cycle2;
        }

        self.cycles -= 1;
    }

    // Reset
    fn reset(&mut self) {
        self.pc = 0xFFFC;
    }

    // Interrupt request (irq)
    fn irq(&mut self) {

    }

    // Not maskeable interrupt (nmi)
    fn nmi(&mut self) {

    }

    // Fetch data
    fn fetch(&mut self) -> u8 {
        0
    }
}

enum Flags {
    C = 0b0000_0001,    // carry
    Z = 0b0000_0010,    // zero
    I = 0b0000_0100,    // disable interrupt
    D = 0b0000_1000,    // decimal
    B = 0b0001_0000,    // break
    U = 0b0010_0000,    // unused
    V = 0b0100_0000,    // overflow
    N = 0b1000_0000,    // negative
}

impl BitOrAssign for Flags {
    fn big_or_assign() {

    }
}

struct Bus<'a> {
    cpu: Option<&'a CPU<'a>>,
    ram: [u8; 64 * 1024],
}

impl<'a> Bus<'a> {
    fn new(cpu: &'a CPU<'a>) -> Self {
        Bus {
            cpu: Some(cpu),
            ram: [0; 64 * 1024],
        }
    }

    // Write data to addr in RAM
    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        } else {
            panic!("Memory access out of bounds. RAM can only access between 0x0000 and 0xFFFF.");
        }
    }

    // Read from RAM at addr
    fn read(&self, addr: u16, read_only: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
            return self.ram[addr as usize];
        } else {
            panic!("Memory access out of bounds. RAM can only access between 0x0000 and 0xFFFF.");
        }
    }
}

struct Instruction<'a> {
    name: &'a str,
    operate: Option<&'a fn()>,
    addr_mode: Option<&'a fn()>,   // 
    cycles: u8,
}

impl<'a> Instruction<'a> {
    fn new(name: &str) -> Self {
        Instruction {
            name,
            operate: None,
            addr_mode: None,
            cycles: 0,
        }
    }
}

fn main() {
    println!("Hello world");
    let mut cpu: CPU = CPU::empty();
    let mut bus: Bus = Bus::new(&cpu);
    cpu.connect_bus(&bus);
    cpu.reset();
}
