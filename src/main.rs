// std::env::set_var::("RUST_BACKTRACE", "1");

use std::{
    io,
    thread,
    time::{Duration, Instant},
    sync::mpsc,
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::*,
    layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub mod cpu;
pub mod bus;
pub mod opcodes;

#[allow(unused_imports)]
use cpu::CPU;
#[allow(unused_imports)]
use cpu::Flags;
#[allow(unused_imports)]
use bus::Bus;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), io::Error> {
    // Init bus and CPU
    let bus: Bus = Bus::new();
    let mut cpu: CPU = CPU::new(bus);
    // cpu.write(0x00F1, 0x27);
    // cpu.load_program(vec![0xA9, 0xA5, 0x69, 0x37, 0x29, 0xF0, 0x0A, 0xA9, 0x5A, 0x69, 0xC3, 0x29, 0x0F, 0x0A, 0xA9, 0x12, 0x69, 0x34, 0x29, 0xAA, 0x0A, 0x00]);
    
    cpu.write(0x05D5, 0xA9);
    cpu.write(0x05D6, 0xFF);
    cpu.quick_start(vec![0xA9, 0x2A, 0x29, 0xC0, 0xF0, 0xD0, 0xA9, 0xAF]);
    
    cpu.reset();

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear terminal
    terminal.clear()?;

    // User event handler
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move | | {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(| | Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // Render loop
    loop {
        // Get values from CPU
        let cpu_state = cpu.get_state();
        let mem = cpu.get_memory();

        // Draw terminal
        terminal.draw(|f| {
            // Set size
            let size = f.size();
            let display_width;
            if (size.width / 2) % 2 == 0 {
                display_width = size.width / 2;
            } else {
                display_width = (size.width / 2) - 1;
            }
            let display_height = display_width / 256 * 240;

            // Divide screen into two halves, horizontally
            let halves = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Length(display_width),
                ])
                .split(size);

            // Left half
            let left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Length(3),
                    Constraint::Min(3),
                ])
                .split(halves[0]);

            // Right half
            let right_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(display_height),
                    Constraint::Min(3),
                    Constraint::Length(6),
                ])
                .split(halves[1]);

            // println!("A register: {}", cpu_state[0]);

            // Register table
            let registers = Table::new(vec![
                Row::new(vec!["A", "X", "Y", "SP", "PC", "SR", "OP"]),
                Row::new(cpu_state.iter().cloned().map(|value| format!("0x{:02X}", value).to_string()).collect::<Vec<_>>())
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Registers")
            )
            .widths(&[
                Constraint::Percentage(13),
                Constraint::Percentage(13),
                Constraint::Percentage(13),
                Constraint::Percentage(13),
                Constraint::Percentage(13),
                Constraint::Percentage(13),
                Constraint::Percentage(13)
            ])
            .column_spacing(1);
            f.render_widget(registers, left_layout[0]);

            // Status flags
            let flags = Table::new(vec![
                Row::new(vec!["C", "Z", "I", "D", "B", "U", "V", "N"]
                    .iter()
                    .map(|&flag| {
                        if cpu_state[5] & Flags::from_str(flag) as u16 == 1 {
                            Cell::from(flag).style(Style::default().bg(Color::White).fg(Color::Black))
                        } else {
                            Cell::from(flag)
                        }
                    })
                ),
                // Row::new(format!("{:08b}", cpu_state[5]).chars().map(|c| c.to_string()).collect::<Vec<_>>())
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Flags")
            )
            .widths(&[
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12)
            ])
            .column_spacing(1);
            f.render_widget(flags, left_layout[1]);

            // Memory (from program start)
            let program: Vec<u8> = mem.iter().cloned().skip(0x0600).collect::<Vec<_>>();
            let indices: Vec<u16> = (0..(0 + program.len() as u16)).collect();

            let program_list = Table::new(
                indices
                    .iter()
                    .map(|i| {
                        if indices[*i as usize] + 0x0600 == cpu_state[4] {
                            Row::new(vec![format!("0x{:04X}", 0x0600 + i), format!("0x{:02X}",program[*i as usize]).to_string()])
                                .style(Style::default().bg(Color::White).fg(Color::Black))
                        } else if program[*i as usize] == 0x00 {
                            Row::new(vec![format!("0x{:04X}", 0x0600 + i), "----".to_string()])
                        } else {
                            Row::new(vec![format!("0x{:04X}", 0x0600 + i), format!("0x{:02X}",program[*i as usize]).to_string()])
                        }
                    })
            )
            .header(
                Row::new(vec!["Address", "Value"])
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Memory (from program start)")
            )
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .column_spacing(1);
            f.render_widget(program_list, left_layout[2]);

            // TODO: proper implementation
            // For now, I just write to memory starting at address 0x3000.
            // In NES, there are 32 horizontal and 30 vertical tiles, each 8x8 pixels.
            // Each tile gets a memory address (starting left top), and the bits correspond to the pixels.
            // Here, I'll start with 8 by 8 virtual pixels.

            let v_pixels: Vec<u8> = vec![
                0, 0, 1, 1, 1, 1, 0, 0,
                0, 0, 0, 1, 1, 0, 0, 0,
                0, 0, 1, 0, 0, 1, 0, 0,
                0, 1, 0, 0, 0, 0, 1, 1,
                0, 1, 1, 1, 1, 1, 1, 1,
                0, 1, 1, 1, 1, 1, 1, 1,
                0, 0, 1, 1, 1, 1, 1, 0,
                0, 0, 0, 1, 1, 0, 0, 0,
            ];

            let v_pixel_array: Vec<Vec<u8>> = v_pixels.chunks(8).map(|chunk| chunk.to_vec()).collect();
            
            // TODO: Display (PPU)

            // Help
            let help = Paragraph::new("<space>: advance to next cycle\n<enter>: start clock\nr: reset CPU\nq: quit application")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Help")
                );
            f.render_widget(help, right_layout[2]);
        })?;

        // Handle user event
        match rx.recv().unwrap() {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    break;
                },
                KeyCode::Char(' ') => {
                    cpu.advance();
                },
                KeyCode::Char('r') => {
                    cpu.reset();
                },
                KeyCode::Enter => {
                    cpu.clock();
                },
                _ => {
                    
                },
            },
            Event::Tick => {
                
            },
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: Test all addressing modes
    use cpu::AddressingMode;

    #[test]
    fn imm() {
        let foo: u16 = 0x5931;
        let mut cpu: CPU = CPU::custom(0, 0, 0, 0, foo, 0, 0, Bus::new());
        assert_eq!(cpu.get_address(AddressingMode::IMM), foo);
    }

    #[test]
    fn zp0() {
        let foo: u8 = 0x72;
        let bar: u8 = 0xF3;
        let mut cpu: CPU = CPU::custom(0, 0, 0, 0, foo as u16, 0, 0, Bus::new());
        cpu.write(foo as u16, bar);
        assert_eq!(cpu.get_address(AddressingMode::ZP0), bar as u16);
    }

    #[test]
    fn zpx() {
        let foo: u8 = 0x72;
        let bar: u8 = 0x2C;
        let x: u8 = 0x1B;
        let mut cpu: CPU = CPU::custom(0, x, 0, 0, foo as u16, 0, 0, Bus::new());
        cpu.write(foo as u16, bar);
        assert_eq!(cpu.get_address(AddressingMode::ZPX), (bar + x) as u16);
    }

    #[test]
    fn zpy() {
        let foo: u8 = 0x72;
        let bar: u8 = 0x2C;
        let y: u8 = 0x1B;
        let mut cpu: CPU = CPU::custom(0, 0, y, 0, foo as u16, 0, 0, Bus::new());
        cpu.write(foo as u16, bar);
        assert_eq!(cpu.get_address(AddressingMode::ZPY), (bar + y) as u16);
    }

    #[test]
    fn abs() {
        let foo: u16 = 0x1234;
        let bar: u16 = 0x5621;
        let mut cpu: CPU = CPU::custom(0, 0, 0, 0, foo, 0, 0, Bus::new());
        cpu.write(foo, 0x21);
        cpu.write(foo + 1, 0x56);
        assert_eq!(cpu.get_address(AddressingMode::ABS), bar);
    }

    #[test]
    fn abx() {
        let foo: u16 = 0x1234;
        let bar: u16 = 0x5621;
        let x: u8 = 0x2F;
        let mut cpu: CPU = CPU::custom(0, x, 0, 0, foo, 0, 0, Bus::new());
        cpu.write(foo, 0x21);
        cpu.write(foo + 1, 0x56);
        assert_eq!(cpu.get_address(AddressingMode::ABX), bar + x as u16);
    }

    #[test]
    fn aby() {
        let foo: u16 = 0x1234;
        let bar: u16 = 0x5621;
        let y: u8 = 0x2F;
        let mut cpu: CPU = CPU::custom(0, 0, y, 0, foo, 0, 0, Bus::new());
        cpu.write(foo, 0x21);
        cpu.write(foo + 1, 0x56);
        assert_eq!(cpu.get_address(AddressingMode::ABY), bar + y as u16);
    }

    #[test]
    fn ind() {
        let pc: u16 = 0x0301;
        let ptr: u16 = 0x4230;
        let addr: u16 = 0x04A9;
        let mut cpu: CPU = CPU::custom(0, 0, 0, 0, pc, 0, 0, Bus::new());
        cpu.write(pc, 0x30);
        cpu.write(pc + 1, 0x42);
        cpu.write(ptr, 0xA9);
        cpu.write(ptr + 1, 0x04);
        assert_eq!(cpu.get_address(AddressingMode::IND), addr);
    }

    #[test]
    fn idx() {
        let pc: u16 = 0x0301;
        let x: u8 = 0x02;
        let base: u8 = 0x30;
        let ptr: u8 = base + x;
        let lo: u8 = 0x91;
        let hi: u8 = 0xEF;
        let addr: u16 = (hi as u16) << 8 | (lo as u16);
        let mut cpu: CPU = CPU::custom(0, x, 0, 0, pc as u16, 0, 0, Bus::new());
        cpu.write(pc, base);
        cpu.write(ptr as u16, lo);
        cpu.write(ptr as u16 + 1, hi);
        assert_eq!(cpu.get_address(AddressingMode::IDX), addr as u16);
    }

    #[test]
    fn idy() {
        let pc: u16 = 0x0301;
        let y: u8 = 0x02;
        let zp_addr: u8 = 0x30;
        let base: u8 = 0x91;
        let ptr: u8 = base + y;
        let addr: u8 = 0x00EF;
        let mut cpu: CPU = CPU::custom(0, 0, y, 0, pc, 0, 0, Bus::new());
        cpu.write(pc, zp_addr);
        cpu.write(zp_addr as u16, base);
        cpu.write(ptr as u16, addr);
        assert_eq!(cpu.get_address(AddressingMode::IDY), addr as u16);
    }

    #[test]
    fn rel() {
        // TODO: how to test relative addressing?
    }

    #[test]
    fn acc() {
        // TODO: how to test accumulator addressing?
    }

    // TODO: Test all instructions
    #[test]
    fn lda_imm_flags() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        cpu.quick_start(vec![0xA9, 0x05, 0x00]);

        assert_eq!(cpu.get_a(), 0x05);
        assert!(!cpu.get_flag(Flags::Z));
        assert!(!cpu.get_flag(Flags::N));
    }

    #[test]
    fn lda_imm_zero_flag() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        cpu.quick_start(vec![0xA9, 0x00, 0x00]);

        assert!(cpu.get_flag(Flags::Z));
    }

    #[test]
    fn lda_zp0() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        // write 0x55 to address 0x10
        cpu.write(0x10, 0x55);
        // LDA from address 0x10
        cpu.quick_start(vec![0xA5, 0x10, 0x00]);

        assert_eq!(cpu.get_a(), 0x55);
    }

    #[test]
    fn adc_imm() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);

        // No carry -> no carry
        {
            cpu.quick_start(vec![0xA9, 0x10, 0x69, 0x02, 0x00]);
            assert_eq!(cpu.get_a(), 0x12);
            assert!(!cpu.get_flag(Flags::C));
            assert!(!cpu.get_flag(Flags::V));
        }

        // Carry -> no carry
        {
            // TODO: how to verify carry?
            cpu.set_flag(Flags::C, true);
            cpu.quick_start(vec![0xA9, 0x10, 0x69, 0x02, 0x00]);
            assert_eq!(cpu.get_a(), 0x12);
            assert!(!cpu.get_flag(Flags::C));
            assert!(!cpu.get_flag(Flags::V));
        }

        // No carry -> carry
        {
            // LDA 0xFE, ADC 0x03. Should wrap around to 0x01.
            cpu.quick_start(vec![0xA9, 0xFE, 0x69, 0x03, 0x00]);
            assert_eq!(cpu.get_a(), 0x01);
            assert!(cpu.get_flag(Flags::C));
            assert!(!cpu.get_flag(Flags::V));
        }

        // No carry -> carry
        {
            // LDA 0xFE, ADC 0x12. Should wrap around to 0x10.
            cpu.quick_start(vec![0xA9, 0xFE, 0x69, 0x12, 0x00]);
            assert_eq!(cpu.get_a(), 0x10);
            assert!(cpu.get_flag(Flags::C));
            assert!(!cpu.get_flag(Flags::V));
        }

        // TODO: Overflow -> no overflow
        {
            cpu.quick_start(vec![]);
        }

        // TODO: No overflow -> overflow
    }

    #[test]
    fn and_imm() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        // LDA(IMM) with 0x6b, AND(IMM) with 0x2c
        cpu.quick_start(vec![0xA9, 0x6b, 0x29, 0x2c, 0x00]);

        assert_eq!(cpu.get_a(), 0x28);
    }

    #[test]
    fn and_imm_zero() {
        let mut cpu: CPU = CPU::new(Bus::new());
        // LDA 0x6B, AND 0x14, BRK
        cpu.quick_start(vec![0xA9, 0x6B, 0x29, 0x14, 0x00]);
        assert!(cpu.get_flag(Flags::Z));
        assert_eq!(cpu.get_a(), 0x00);
    }

    // write number to memory, lda immediate, ldx immediate, then adc with zpx
    #[test]
    fn adc_zpx() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        cpu.write(0x00F1, 0x27);
        cpu.quick_start(vec![0xA9, 0x03, 0xA2, 0x10, 0x75, 0xE1, 0x00]);

        assert_eq!(cpu.get_a(), 0x2A);
    }

    #[test]
    fn asl_acc() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);

        {
            cpu.quick_start(vec![0xA9, 0b0010_1000, 0x0A, 0x00]);
            assert_eq!(cpu.get_a(), 0b0101_0000);
            assert!(!cpu.get_flag(Flags::C));
            assert!(!cpu.get_flag(Flags::N));
            assert!(!cpu.get_flag(Flags::Z));
        }

        {
            cpu.quick_start(vec![0xA9, 0b1010_0000, 0x0A, 0x00]);
            assert_eq!(cpu.get_a(), 0b0100_0000);
            assert!(cpu.get_flag(Flags::C));
            // println!("A: {}", cpu.get_a());
        }

        {
            cpu.quick_start(vec![0xA9, 0b1000_0000, 0x0A, 0x00]);
            assert_eq!(cpu.get_a(), 0x00);
            assert!(cpu.get_flag(Flags::C));
            assert!(cpu.get_flag(Flags::Z));
            assert!(!cpu.get_flag(Flags::N));
        }

        {
            cpu.quick_start(vec![0xA9, 0b0100_0000, 0x0A, 0x00]);
            assert_eq!(cpu.get_a(), 0b1000_0000);
            assert!(cpu.get_flag(Flags::N));
            assert!(!cpu.get_flag(Flags::Z));
            assert!(!cpu.get_flag(Flags::C));
        }
    }

    #[test]
    fn beq_rel_pos() {
        let mut cpu: CPU = CPU::new(Bus::new());
        // LDA 0xA9, AND 0xC0, BEQ -> LDA 0xFF, BRK if no zero flag (A would remain 0xA9)
        cpu.quick_start(vec![0xA9, 0x2A, 0x29, 0xC0, 0xF0, 0x03, 0x00, 0x00, 0xA9, 0xFF, 0x00]);
        assert!(!cpu.get_flag(Flags::Z));
        assert_eq!(cpu.get_a(), 0xFF);
    }

    #[test]
    fn beq_rel_neg() {
        let mut cpu: CPU = CPU::new(Bus::new());
        cpu.write(0x05D5, 0xA9);
        cpu.write(0x05D6, 0xFF);
        // LDA 0xA9, AND 0xC0, BEQ -> LDA 0xFF, LDA 0xAF if no zero flag
        cpu.quick_start(vec![0xA9, 0x2A, 0x29, 0xC0, 0xF0, 0xD0, 0xA9, 0xAF]);
        assert_eq!(cpu.get_a(), 0xFF);
    }

    // TODO: test what happens if underflow occurs
    // #[test]
    // fn beq_rel_under() {

    // }

    #[test]
    fn clc_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::C));
        cpu.set_flag(Flags::C, true);
        assert!(cpu.get_flag(Flags::C));
        cpu.quick_start(vec![0x18, 0x00]);
        assert!(!cpu.get_flag(Flags::C));
    }

    #[test]
    fn cld_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::D));
        cpu.set_flag(Flags::D, true);
        assert!(cpu.get_flag(Flags::D));
        cpu.quick_start(vec![0xD8, 0x00]);
        assert!(!cpu.get_flag(Flags::D));
    }

    #[test]
    fn cli_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::I));
        cpu.set_flag(Flags::I, true);
        assert!(cpu.get_flag(Flags::I));
        cpu.quick_start(vec![0x58, 0x00]);
        assert!(!cpu.get_flag(Flags::I));
    }

    #[test]
    fn clv_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::V));
        cpu.set_flag(Flags::V, true);
        assert!(cpu.get_flag(Flags::V));
        cpu.quick_start(vec![0xB8, 0x00]);
        assert!(!cpu.get_flag(Flags::V));
    }

    // TODO: test ASL(ABS)
    // #[test]
    // fn test_asl_abs() {
    //     let bus: Bus = Bus::new();
    //     let mut cpu: CPU = CPU::new(bus);

    //     // TODO: LDA(IMM): 0b0010_1000, STA(ABS) to addr: 0x3B01, ASL(ABS) to addr: 0x3B01
    //     {
    //         cpu.quick_start(vec![0xA9, 0b0010_1000, 0x8D, 0x01, 0x3B, 0x0E, 0x01, 0x3B, 0x00]);
    //         assert_eq!(cpu.read(0x3B), 0b0101_0000);
    //     }
    // }
    
    // TODO: test ASL with different mode(s)

    #[test]
    fn jmp_abs() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        cpu.write(0x3000, 0xA9);
        cpu.write(0x3001, 0x04);
        cpu.write(0x3002, 0x00);

        // LDA 0x02, JMP to 0x3000. Then LDA 0x04 and BRK.
        cpu.quick_start(vec![0xA9, 0x02, 0x4C, 0x00, 0x30]);
        assert_eq!(cpu.get_a(), 0x04);
    }

    #[test]
    fn jmp_ind() {
        let bus: Bus = Bus::new();
        let mut cpu: CPU = CPU::new(bus);
        cpu.write(0x1234, 0x30);
        cpu.write(0x1235, 0x24);
        cpu.write(0x2430, 0xA9);
        cpu.write(0x2431, 0x04);
        cpu.write(0x2432, 0x00);

        // LDA 0x02, JMP to pointer specified by 0x1234 (so to 0x2430). Then LDA 0x04 and BRK.
        cpu.quick_start(vec![0xA9, 0x02, 0x6C, 0x34, 0x12]);
        assert_eq!(cpu.get_a(), 0x04);
    }

    #[test]
    fn sec_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::C));
        cpu.quick_start(vec![0x38, 0x00]);
        assert!(cpu.get_flag(Flags::C));
    }

    #[test]
    fn sed_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::D));
        cpu.quick_start(vec![0xF8, 0x00]);
        assert!(cpu.get_flag(Flags::D));
    }

    #[test]
    fn sei_imp() {
        let mut cpu: CPU = CPU::new(Bus::new());
        assert!(!cpu.get_flag(Flags::I));
        cpu.quick_start(vec![0x78, 0x00]);
        assert!(cpu.get_flag(Flags::I));
    }

    #[test]
    fn sta_zp0() {
        let mut cpu: CPU = CPU::new(Bus::new());
        cpu.quick_start(vec![0xA9, 0xFF, 0x85, 0xAB]);
        assert_eq!(cpu.read(0xAB), 0xFF);
    }

    // Test the CPU using a simple snake game
    // #[test]
    // fn snake() {
    //     use rand::Rng;

    //     let mut rng = rand::thread_rng();

    //     let game_code = vec![
    //         0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
    //         0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
    //         0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
    //         0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
    //         0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
    //         0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
    //         0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
    //         0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
    //         0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
    //         0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
    //         0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
    //         0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
    //         0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
    //         0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
    //         0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
    //         0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
    //         0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
    //         0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
    //         0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
    //         0xea, 0xca, 0xd0, 0xfb, 0x60,
    //     ];

    //     let mut cpu: CPU = CPU::new(Bus::new());
    //     cpu.load_program(game_code);

    //     // User event handler
    //     let (tx, rx) = mpsc::channel();
    //     let tick_rate = Duration::from_millis(200);
    //     thread::spawn(move | | {
    //         let mut last_tick = Instant::now();
    //         loop {
    //             let timeout = tick_rate
    //                 .checked_sub(last_tick.elapsed())
    //                 .unwrap_or_else(| | Duration::from_secs(0));

    //             if event::poll(timeout).expect("poll works") {
    //                 if let CEvent::Key(key) = event::read().expect("can read events") {
    //                     tx.send(Event::Input(key)).expect("can send events");
    //                 }
    //             }

    //             if last_tick.elapsed() >= tick_rate {
    //                 if let Ok(_) = tx.send(Event::Tick) {
    //                     last_tick = Instant::now();
    //                 }
    //             }
    //         }
    //     });

    //     // custom clock
    //     loop {
    //         // write random number to 0xFE
    //         let number: u8 = rng.gen();
    //         cpu.write(0xFE, number);

    //         // get user input
    //         let mut controller: u8 = 0x00;
    //         match rx.recv().unwrap() {
    //             Event::Input(event) => match event.code {
    //                 KeyCode::Char('q') => {
    //                     break;
    //                 },
    //                 KeyCode::Char('r') => {
    //                     cpu.reset();
    //                 },
    //                 KeyCode::Char('w') => {
    //                     controller |= 0b0000_1000;
    //                 },
    //                 KeyCode::Char('a') => {
    //                     controller |= 0b0000_0100;
    //                 },
    //                 KeyCode::Char('s') => {
    //                     controller |= 0b0000_0010;
    //                 },
    //                 KeyCode::Char('d') => {
    //                     controller |= 0b0000_0001;
    //                 },
    //                 _ => {
                        
    //                 },
    //             },
    //             Event::Tick => {
                    
    //             },
    //         }

    //         // TODO: write pixels to 32x32 matrix (0x0200 -> 0x021F horizontal, 0x021F -> )

    //         // write user input to 0xFF
    //         cpu.write(0xFF, controller);

    //         // advance to next cycle
    //         cpu.advance();
    //     }
    // }
}
