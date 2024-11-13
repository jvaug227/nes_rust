pub mod core;
pub mod instructions;

pub use core::*;

/*#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::core::*;

    fn create_cpu() -> Cpu {
        let mut bus = Bus::new();
		bus.write(0xFFFA, 0x00);
		bus.write(0xFFFB, 0x80);
		bus.write(0xFFFC, 0x00);
		bus.write(0xFFFD, 0x80);
		bus.write(0xFFFE, 0x00);
		bus.write(0xFFFF, 0x80);
        let bus = Rc::new(RefCell::new(bus));
        Cpu::new(bus)
    }

    fn print_cpu(cpu: &Cpu) {
        println!("CPU: PC 0x{:0>4X}, A {:0>3}, X {:0>3}, Y {:0>3}, STK {:0>3}, ADDR: {:0>6X}, Fetched {:0>3}, S {:0>8b}, Pipeline: {:?}, INS: {}, PGC: {}",
            cpu.pc, cpu.a, cpu.x, cpu.y, cpu.stkpt, cpu.addr_data, cpu.fetched, cpu.get_status(), cpu.pipeline_status, cpu.opcode, cpu.page_boundary_crossed);
    }

    fn write_program(cpu: &mut Cpu, address: u16, instructions: &[u8]) {
        (0..instructions.len()).for_each(|i| {
            cpu.bus_mut().write(address + i as u16, instructions[i]);
        });
    }

    fn clock_instruction_debug(cpu: &mut Cpu) -> usize {
        let mut cycles = 0; // Assume instruction byte was read as the last one ended
        let mut should_clock = true;
        while should_clock {
            should_clock = !cpu.clock();
            print_cpu(cpu);
            cycles += 1;
        }
        cycles
    }

    #[test]
    fn cpu_correctly_initialized() {
        let cpu = create_cpu();
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.stkpt, 0);

        assert_eq!(cpu.get_status(), Flags6502::empty());
    }

    // TODO: Move flag sets to after brk instruction in ADC, LDY, and LDX
    // TODO: Further test Overflow bit
    #[test]
    fn adc_immediate() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, EXPECTED_VALUE]); // ADC #8
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_immediate_a_initially_not_zero() {
        const EXPECTED_VALUE: u8 = 8 + 5;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 8]); // ADC #8
        cpu.a = 5;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_immediate_a_becomes_zero() {
        const EXPECTED_VALUE: u8 = 0;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 0]); // ADC #0
        cpu.a = 0;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should be set, the rest unchanged!");
    }
    #[test]
    fn adc_immediate_a_hardware_overflows() {
        const EXPECTED_VALUE: u8 = 3;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 0xFF]); // ADC #FF
        cpu.a = 0x04;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::C, end_status, "Carry flag should be set, the rest unchanged!");
    }
    #[test]
    fn adc_immediate_a_hardware_overflows_to_zero() {
        const EXPECTED_VALUE: u8 = 0;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 0xFF]); // ADC #FF
        cpu.a = 0x01;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and Carry flags should be set, the rest unchanged!");
    }
    #[test]
    fn adc_immediate_a_software_overflows() {
        const EXPECTED_VALUE: u8 = 0x7F + 0x01;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 0x01]); // ADC #1
        cpu.a = 0x7F;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N | Flags6502::V, end_status, "Negative and Overflow flags should be set, the rest unchanged!");
    }
    #[test]
    fn adc_immediate_add_with_carry_flag_enabled() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01 + 0x01;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x69, 0x01]); // ADC #1
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.set_flag(Flags6502::C, true);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status & !Flags6502::C, end_status, "Carry flag should be unset, rest unchanged!");
    }
    #[test]
    fn adc_zp() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0005, &[0x01]); // $05 -> 1
        write_program(&mut cpu, 0x8000, &[0x65, 0x05]); // ADC $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_zp_x() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x000B, &[0x01]); // $0B -> 1
        write_program(&mut cpu, 0x8000, &[0x75, 0x05]); // ADC $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.x = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_abs() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2003, &[0x01]); // $2003 -> 1
        write_program(&mut cpu, 0x8000, &[0x6D, 0x03, 0x20]); // ADC $2003
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_abs_x() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x200B, &[0x01]); // $200B -> 1
        write_program(&mut cpu, 0x8000, &[0x7D, 0x05, 0x20]); // ADC $2005,X
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.x = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_abs_x_page_boundary() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2105, &[0x01]); // $2105 -> 1
        write_program(&mut cpu, 0x8000, &[0x7D, 0xFF, 0x20]); // ADC $20FF,X
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.x = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_abs_y() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x200B, &[0x01]); // $200B -> 1
        write_program(&mut cpu, 0x8000, &[0x79, 0x05, 0x20]); // ADC $2005,Y
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.y = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_abs_y_page_boundary() {
        const EXPECTED_VALUE: u8 = 0x03 + 0x01;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2105, &[0x01]); // $2105 -> 1
        write_program(&mut cpu, 0x8000, &[0x79, 0xFF, 0x20]); // ADC $20FF,Y
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.y = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_indexed_indirect() {
        const EXPECTED_VALUE: u8 = 0x04;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0036, &[0x03, 0x20]); // $36 ($30 + x) -> $2003
        write_program(&mut cpu, 0x2003, &[0x01]); // $2003 -> 1
        write_program(&mut cpu, 0x8000, &[0x61, 0x30]); // ADC ($30, X)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.x = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC ($ZZ,X) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_indirect_indexed() {
        const EXPECTED_VALUE: u8 = 0x04;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0030, &[0x00, 0x20]); // $30 -> $2000
        write_program(&mut cpu, 0x2006, &[0x01]); // $2006 ($2000 + Y) -> 1
        write_program(&mut cpu, 0x8000, &[0x71, 0x30]); // ADC ($30),Y
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.y = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC ($ZZ),Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn adc_indirect_indexed_page_boundary() {
        const EXPECTED_VALUE: u8 = 0x04;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0030, &[0xFF, 0x20]); // $30 -> $20FF
        write_program(&mut cpu, 0x2105, &[0x01]); // $2105 ($2000 + Y) -> 1
        write_program(&mut cpu, 0x8000, &[0x71, 0x30]); // ADC ($30),Y
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0x03;
        cpu.y = 0x06;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "ADC ($ZZ),Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    
    #[test]
    fn and_immediate() {
        const EXPECTED_VALUE: u8 = 0b00001010;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x29, EXPECTED_VALUE]); // AND #0b00001010
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00001110;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn and_immediate_zero_flag() {
        const EXPECTED_VALUE: u8 = 0b00000000;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x29, 0b00000100]); // AND #0b00000100
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00001010;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn and_immediate_sign_flag() {
        const EXPECTED_VALUE: u8 = 0b10000000;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x29, 0b10000100]); // AND #0b10000100
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b10001010;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn asl_accumulator() {
        const EXPECTED_VALUE: u8 = 0b00010100;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x0A]); // AND #0b00001010
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00001010;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn asl_accumulator_carry_flag() {
        const EXPECTED_VALUE: u8 = 0b00000010;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x0A]); // AND #0b00001010
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b10000001;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::C, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn asl_accumulator_sign_flag() {
        const EXPECTED_VALUE: u8 = 0b10000010;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x0A]); // AND #0b00001010
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b01000001;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of index register A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn asl_zp() {
        const EXPECTED_VALUE: u8 = 100;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0008, &[50]); // AND #0b00001010
        write_program(&mut cpu, 0x8000, &[0x06, 0x08]); // AND #0b00001010
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result = cpu.read_byte(0x0008);
        assert_eq!(result, EXPECTED_VALUE, "Value of ZP is {} and should be {}", result, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "AND $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    fn generic_branch_should_fail(branch_instruction: u8, flags: Flags6502) {
        const EXPECTED_VALUE: u8 = 35;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8008, &[0xA0, 50]); // AND #0b00001010
        write_program(&mut cpu, 0x8000, &[branch_instruction, 0x08, 0xA0, EXPECTED_VALUE]);
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        cpu.y = 0;
        cpu.set_flags(flags);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        _ = clock_instruction_debug(&mut cpu); // LDY #50
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "branch expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    fn generic_branch_should_pass(branch_instruction: u8, flags: Flags6502) {
        const EXPECTED_VALUE: u8 = 50;
        const EXPECTED_CYCLES: usize = 3;
        const START_ADDRESS: u16 = 0x8000;
        const OFFSET: u8 = 0x08;
        const EXPECTED_ADDRESS: u16 = START_ADDRESS + OFFSET as u16;
        let mut cpu = create_cpu();
        write_program(&mut cpu, EXPECTED_ADDRESS, &[0xA0, 50]); // LDY #50
        write_program(&mut cpu, START_ADDRESS, &[branch_instruction, OFFSET.wrapping_sub(2)]);
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        cpu.y = 0;
        cpu.set_flags(flags);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        _ = clock_instruction_debug(&mut cpu); // LDY #50
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "branch expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    fn generic_branch_should_pass_with_page_boundary(branch_instruction: u8, flags: Flags6502) {
        const EXPECTED_VALUE: u8 = 50;
        const EXPECTED_CYCLES: usize = 4;
        const START_ADDRESS: u16 = 0x80F0;
        const OFFSET: u8 = 0x11;
        const EXPECTED_ADDRESS: u16 = START_ADDRESS + OFFSET as u16;
        let mut cpu = create_cpu();
        write_program(&mut cpu, EXPECTED_ADDRESS, &[0xA0, 50]); // LDY #50
        write_program(&mut cpu, START_ADDRESS, &[branch_instruction, OFFSET.wrapping_sub(2)]);
        write_program(&mut cpu, 0xFFFE, &[lo_byte(START_ADDRESS), hi_byte(START_ADDRESS)]);
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        cpu.y = 0;
        cpu.set_flags(flags);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        _ = clock_instruction_debug(&mut cpu); // LDY #50
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "branch expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    fn generic_branch_should_pass_with_negative_offset(branch_instruction: u8, flags: Flags6502) {
        const EXPECTED_VALUE: u8 = 50;
        const EXPECTED_CYCLES: usize = 4;
        const START_ADDRESS: u16 = 0x8008;
        const OFFSET: u8 = (-8i8) as u8;
        const EXPECTED_ADDRESS: u16 = START_ADDRESS.wrapping_add(OFFSET as i8 as i16 as u16);
        let mut cpu = create_cpu();
        write_program(&mut cpu, EXPECTED_ADDRESS, &[0xA0, 50]); // LDY #50
        write_program(&mut cpu, START_ADDRESS, &[branch_instruction, OFFSET.wrapping_sub(2)]);
        write_program(&mut cpu, 0xFFFE, &[lo_byte(START_ADDRESS), hi_byte(START_ADDRESS)]);
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        cpu.y = 0;
        cpu.set_flags(flags);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        _ = clock_instruction_debug(&mut cpu); // LDY #50
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "branch expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    fn generic_branch_should_pass_with_negative_offset_on_page_boundary(branch_instruction: u8, flags: Flags6502) {
        const EXPECTED_VALUE: u8 = 50;
        const EXPECTED_CYCLES: usize = 4;
        const START_ADDRESS: u16 = 0x8000;
        const OFFSET: u8 = (-8i8) as u8;
        const EXPECTED_ADDRESS: u16 = START_ADDRESS.wrapping_add(OFFSET as i8 as i16 as u16);
        let mut cpu = create_cpu();
        write_program(&mut cpu, EXPECTED_ADDRESS, &[0xA0, 50]); // LDY #50
        write_program(&mut cpu, START_ADDRESS, &[branch_instruction, OFFSET.wrapping_sub(2)]);
        write_program(&mut cpu, 0xFFFE, &[lo_byte(START_ADDRESS), hi_byte(START_ADDRESS)]);
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0;
        cpu.y = 0;
        cpu.set_flags(flags);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        _ = clock_instruction_debug(&mut cpu); // LDY #50
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "branch expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn bcc_branch_fails() {
        generic_branch_should_fail(0x90, Flags6502::C);
    }
    #[test]
    fn bcc_branch_passes() {
        generic_branch_should_pass(0x90, Flags6502::empty());
    }
    #[test]
    fn bcc_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0x90, Flags6502::empty());
    }
    #[test]
    fn bcc_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0x90, Flags6502::empty());
    }
    #[test]
    fn bcc_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0x90, Flags6502::empty());
    }
    #[test]
    fn bcs_branch_fails() {
        generic_branch_should_fail(0xB0, Flags6502::empty());
    }
    #[test]
    fn bcs_branch_passes() {
        generic_branch_should_pass(0xB0, Flags6502::C);
    }
    #[test]
    fn bcs_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0xB0, Flags6502::C);
    }
    #[test]
    fn bcs_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0xB0, Flags6502::C);
    }
    #[test]
    fn bcs_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0xB0, Flags6502::C);
    }

        #[test]
    fn bnq_branch_fails() {
        generic_branch_should_fail(0xD0, Flags6502::Z);
    }
    #[test]
    fn bnq_branch_passes() {
        generic_branch_should_pass(0xD0, Flags6502::empty());
    }
    #[test]
    fn bnq_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0xD0, Flags6502::empty());
    }
    #[test]
    fn bnq_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0xD0, Flags6502::empty());
    }
    #[test]
    fn bnq_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0xD0, Flags6502::empty());
    }
    #[test]
    fn beq_branch_fails() {
        generic_branch_should_fail(0xF0, Flags6502::empty());
    }
    #[test]
    fn beq_branch_passes() {
        generic_branch_should_pass(0xF0, Flags6502::Z);
    }
    #[test]
    fn beq_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0xF0, Flags6502::Z);
    }
    #[test]
    fn beq_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0xF0, Flags6502::Z);
    }
    #[test]
    fn beq_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0xF0, Flags6502::Z);
    }

    #[test]
    fn bpl_branch_fails() {
        generic_branch_should_fail(0x10, Flags6502::N);
    }
    #[test]
    fn bpl_branch_passes() {
        generic_branch_should_pass(0x10, Flags6502::empty());
    }
    #[test]
    fn bpl_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0x10, Flags6502::empty());
    }
    #[test]
    fn bpl_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0x10, Flags6502::empty());
    }
    #[test]
    fn bpl_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0x10, Flags6502::empty());
    }
    #[test]
    fn bmi_branch_fails() {
        generic_branch_should_fail(0x30, Flags6502::empty());
    }
    #[test]
    fn bmi_branch_passes() {
        generic_branch_should_pass(0x30, Flags6502::N);
    }
    #[test]
    fn bmi_branch_passes_with_page_boundary() {
        generic_branch_should_pass_with_page_boundary(0x30, Flags6502::N);
    }
    #[test]
    fn bmi_branch_passes_with_negative_offset() {
        generic_branch_should_pass_with_negative_offset(0x30, Flags6502::N);
    }
    #[test]
    fn bmi_branch_passes_with_negative_offset_on_page_boundary() {
        generic_branch_should_pass_with_negative_offset_on_page_boundary(0x30, Flags6502::N);
    }


    #[test]
    fn bit_zp_no_flags_changed() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x11, &[0b00011100]); // $11 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x24, 0x11]); // BIT $11
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn bit_zp_zero_flag_set() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x11, &[0b00011100]); // $11 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x24, 0x11]); // BIT $11
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00000000; // A = 0 : a value that always results in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should have been set, and the rest unchanged!");
    }
    #[test]
    fn bit_zp_sign_flag_set() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x11, &[0b10011100]); // $11 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x24, 0x11]); // BIT $11
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "SigN flag should have been set, and the rest unchanged!");
    }
    #[test]
    fn bit_zp_overflow_flag_set() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x11, &[0b01011100]); // $11 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x24, 0x11]); // BIT $11
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::V, end_status, "OVerflow flag should have been set, and the rest unchanged!");
    }
    #[test]
    fn bit_abs_no_flags_changed() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x1107, &[0b00011100]); // $1107 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x2C, 0x07, 0x11]); // BIT $1107
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn bit_abs_zero_flag_set() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x1107, &[0b00011100]); // $1107 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x2C, 0x07, 0x11]); // BIT $1107
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00000000; // A = 0 : a value that always results in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should have been set, and the rest unchanged!");
    }
    #[test]
    fn bit_abs_sign_flag_set() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x1107, &[0b10011100]); // $1107 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x2C, 0x07, 0x11]); // BIT $1107
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "SigN flag should have been set, and the rest unchanged!");
    }
    #[test]
    fn bit_abs_overflow_flag_set() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x1107, &[0b01011100]); // $1107 -> 28 (0b00011100)
        write_program(&mut cpu, 0x8000, &[0x2C, 0x07, 0x11]); // BIT $1107
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b00010100; // A = 20 : a value that does not result in the zero flag being set
        cpu.set_flags(cpu.get_status() & !(Flags6502::N | Flags6502::V | Flags6502::Z)); // unset the testing bits
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BIT $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::V, end_status, "OVerflow flag should have been set, and the rest unchanged!");
    }

    #[test]
    fn brk_test() {
        const EXPECTED_CYCLES: usize = 7;
        let mut cpu = create_cpu();
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "BRK expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::I, end_status, "");
    }

    #[test]
    fn clc_no_change() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x18]); // CLC
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLC expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn clc_clears_flag() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x18]); // CLC
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.set_flag(Flags6502::C, true);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLC expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status & !Flags6502::C, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn cld_no_change() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xD8]); // CLD
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLD expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn cld_clears_flag() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xD8]); // CLD
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.set_flag(Flags6502::D, true);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLD expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status & !Flags6502::D, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn cli_no_change() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x58]); // CLI
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.set_flag(Flags6502::I, false); // Clear inturrupt flag as it is set during the break
                                           // instruction
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLI expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn cli_clears_flag() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x58]); // CLI
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.set_flag(Flags6502::I, true);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLI expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status & !Flags6502::I, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn clv_no_change() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xB8]); // CLV
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLV expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn clv_clears_flag() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xB8]); // CLV
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.set_flag(Flags6502::V, true);
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CLV expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status & !Flags6502::V, end_status, "Status flags should not have changed!");
    }

    // TODO: Write more tests for cmp, cpx, cpy to test overflow and underflow edgecases
    #[test]
    fn cmp_immediate_eq() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC9, 0b0111]); // CMP 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and Carry flag should have been set.");
    }
    #[test]
    fn cmp_immediate_a_gt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC9, 0b0111]); // CMP 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b1111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::C, end_status, "Carry flag should have been set.");
    }
    #[test]
    fn cmp_immediate_a_lt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC9, 0b1111]); // CMP 0b1111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set.");
    }
    #[test]
    fn cmp_zp() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xC5, 0x05]); // CMP $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_zp_x() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x08, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xD5, 0x05]); // CMP $05,X
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.x = 3;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_abs() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xCD, 0x05, 0x02]); // CMP $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.x = 3;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_abs_x() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0208, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xDD, 0x05, 0x02]); // CMP $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.x = 3;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_abs_y() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0208, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xD9, 0x05, 0x02]); // CMP $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.y = 3;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_ind_x() {
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0208, &[0b0111]);
        write_program(&mut cpu, 0x0008, &[0x08, 0x02]);
        write_program(&mut cpu, 0x8000, &[0xC1, 0x05]); // CMP $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.x = 3;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP ($ZZ,X) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cmp_ind_y() {
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b0111]);
        write_program(&mut cpu, 0x0005, &[0x03, 0x02]);
        write_program(&mut cpu, 0x8000, &[0xD1, 0x05]); // CMP $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.a = 0b0111;
        cpu.y = 2;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CMP ($ZZ),Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }

    #[test]
    fn cpx_immediate_eq() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xE0, 0b0111]); // CPX 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and Carry flag should have been set.");
    }
    #[test]
    fn cpx_immediate_a_gt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xE0, 0b0111]); // CPX 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 0b1111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::C, end_status, "Carry flag should have been set.");
    }
    #[test]
    fn cpx_immediate_a_lt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xE0, 0b1111]); // CPX 0b1111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set.");
    }
    #[test]
    fn cpx_zp() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xE4, 0x05]); // CPX $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPX $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cpx_abs() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xEC, 0x05, 0x02]); // CPX $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPX $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }

    #[test]
    fn cpy_immediate_eq() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC0, 0b0111]); // CPY 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and Carry flag should have been set.");
    }
    #[test]
    fn cpy_immediate_a_gt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC0, 0b0111]); // CPY 0b0111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 0b1111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::C, end_status, "Carry flag should have been set.");
    }
    #[test]
    fn cpy_immediate_a_lt_m() {
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC0, 0b1111]); // CPY 0b1111
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set.");
    }
    #[test]
    fn cpy_zp() {
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xC4, 0x05]); // CPY $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPY $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }
    #[test]
    fn cpy_abs() {
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b0111]);
        write_program(&mut cpu, 0x8000, &[0xCC, 0x05, 0x02]); // CPY $0205
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 0b0111;
        cpu.set_flags(Flags6502::empty());
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cycles, EXPECTED_CYCLES, "CPY $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z | Flags6502::C, end_status, "Zero and carry flags should have been set.");
    }

    // TODO: Test flags for DEC, DEX, and DEY; especially on underflows
    #[test]
    fn dec_zp() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[8]);
        write_program(&mut cpu, 0x8000, &[0xC6, 0x05]); // DEC $05
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x05);
        assert_eq!(result_value, EXPECTED_VALUE, "Value in ZP at 0x05 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEC $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn dec_zp_x() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[8]);
        write_program(&mut cpu, 0x8000, &[0xD6, 0x02]); // DEC $05,X
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 3;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x05);
        assert_eq!(result_value, EXPECTED_VALUE, "Value in ZP at 0x05 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEC $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn dec_abs() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0203, &[8]);
        write_program(&mut cpu, 0x8000, &[0xCE, 0x03, 0x02]); // DEC $0203
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 3;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x0203);
        assert_eq!(result_value, EXPECTED_VALUE, "Value in ZP at 0x0203 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEC $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn dec_abs_x() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 7;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0203, &[8]);
        write_program(&mut cpu, 0x8000, &[0xDE, 0x00, 0x02]); // DEC $0203
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 3;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x0203);
        assert_eq!(result_value, EXPECTED_VALUE, "Value in ZP at 0x0203 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEC $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn dex_imp() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xCA]); // DEx
        cpu.x = 8;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.x;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in index register X is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEX expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn dey_imp() {
        const EXPECTED_VALUE: u8 = 7;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x88]); // DEY
        cpu.y = 8;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.y;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in index register Y is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "DEY expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    // TODO: Test flags
    #[test]
    fn eor_imm() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x49, 0b00010110]); // EOR #
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_zp() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x02, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x45, 0x02]); // EOR $ZZ
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_zp_x() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x55, 0x02]); // EOR $ZZ
        cpu.x = 3;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_abs() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0203, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x4D, 0x03, 0x02]); // EOR $ZZ
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_abs_x() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x5D, 0x03, 0x02]); // EOR $ZZ
        cpu.x = 2;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_abs_x_cross_page_boundary() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0302, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x5D, 0xFF, 0x02]); // EOR $ZZ
        cpu.x = 3;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_abs_y() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x59, 0x03, 0x02]); // EOR $ZZ
        cpu.y = 2;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_abs_y_cross_page_boundary() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0302, &[0b00010110]);
        write_program(&mut cpu, 0x8000, &[0x59, 0xFF, 0x02]); // EOR $ZZ
        cpu.y = 3;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_ind_x() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b00010110]);
        write_program(&mut cpu, 0x07, &[0x05, 0x02]);
        write_program(&mut cpu, 0x8000, &[0x41, 0x05]); // EOR $ZZ
        cpu.x = 2;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_ind_y() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[0b00010110]);
        write_program(&mut cpu, 0x05, &[0x03, 0x02]);
        write_program(&mut cpu, 0x8000, &[0x51, 0x05]); // EOR $ZZ
        cpu.y = 2;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn eor_ind_y_cross_page_boundary() {
        const EXPECTED_VALUE: u8 = 0b00011100;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0302, &[0b00010110]);
        write_program(&mut cpu, 0x05, &[0xFF, 0x02]);
        write_program(&mut cpu, 0x8000, &[0x51, 0x05]); // EOR $ZZ
        cpu.y = 3;
        cpu.a = 0b00001010;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.a;
        assert_eq!(result_value, EXPECTED_VALUE, "Value in accumulator A is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "EOR expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    // TODO: Test flags
    #[test]
    fn inc_zp() {
        const EXPECTED_VALUE: u8 = 36;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x03, &[35]);
        write_program(&mut cpu, 0x8000, &[0xE6, 0x03]); // INC $ZZ
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x03);
        assert_eq!(result_value, EXPECTED_VALUE, "Value of $0x05 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INC $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn inc_zp_x() {
        const EXPECTED_VALUE: u8 = 36;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x05, &[35]);
        write_program(&mut cpu, 0x8000, &[0xF6, 0x03]); // INC $ZZ,X
        cpu.x = 2;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x05);
        assert_eq!(result_value, EXPECTED_VALUE, "Value of $0x05 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INC $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn inc_abs() {
        const EXPECTED_VALUE: u8 = 36;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0203, &[35]);
        write_program(&mut cpu, 0x8000, &[0xEE, 0x03, 0x02]); // INC $AAAA
        cpu.x = 2;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x0203);
        assert_eq!(result_value, EXPECTED_VALUE, "Value of $0x0203 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INC $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn inc_abs_x() {
        const EXPECTED_VALUE: u8 = 36;
        const EXPECTED_CYCLES: usize = 7;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0205, &[35]);
        write_program(&mut cpu, 0x8000, &[0xFE, 0x03, 0x02]); // INC $AAAA,X
        cpu.x = 2;
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_value = cpu.read_byte(0x0205);
        assert_eq!(result_value, EXPECTED_VALUE, "Value of $0x0205 is {} and should be {}", result_value, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INC $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }


    #[test]
    fn inx_implied() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xE8]); // INX
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.x = 7;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INX expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn iny_implied() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xC8]); // INY
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        cpu.y = 7;
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "INY expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn jmp_abs() {
        const EXPECTED_ADDRESS: u16 = 0x0201;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x4C, 0x00, 0x02]); // JMP $0200
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        let result_address = cpu.pc;
        assert_eq!(result_address, EXPECTED_ADDRESS, "Value of pc is 0x{:X} and should be 0x{:X}", result_address, EXPECTED_ADDRESS);
        assert_eq!(cycles, EXPECTED_CYCLES, "JMP $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn jmp_ind() {
        const EXPECTED_ADDRESS: u16 = 0x0411;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0200, &[0x10, 0x04]);
        write_program(&mut cpu, 0x8000, &[0x6C, 0x00, 0x02]); // JMP ($0200)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        let result_address = cpu.pc;
        assert_eq!(result_address, EXPECTED_ADDRESS, "Value of pc is 0x{:X} and should be 0x{:X}", result_address, EXPECTED_ADDRESS);
        assert_eq!(cycles, EXPECTED_CYCLES, "JMP ($AAAA) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn jmp_ind_page_boundary() {
        const EXPECTED_ADDRESS: u16 = 0x0411;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        // JSR IND does not handle page boundaries
        write_program(&mut cpu, 0x0200, &[0x04]);
        write_program(&mut cpu, 0x02FF, &[0x10]);
        write_program(&mut cpu, 0x8000, &[0x6C, 0xFF, 0x02]); // JMP ($02FF)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        let result_address = cpu.pc;
        assert_eq!(result_address, EXPECTED_ADDRESS, "Value of pc is 0x{:X} and should be 0x{:X}", result_address, EXPECTED_ADDRESS);
        assert_eq!(cycles, EXPECTED_CYCLES, "JMP ($AAAA) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn jsr_abs() {
        const EXPECTED_ADDRESS: u16 = 0x0201;
        const EXPECTED_CYCLES: usize = 6;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0x20, 0x00, 0x02]); // JMP $0200
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        let result_address = cpu.pc;
        let last_stack_value = cpu.read_byte(0x0101 + cpu.stkpt as u16);
        let last_stack_value1 = cpu.read_byte(0x0102 + cpu.stkpt as u16);
        println!("Last stack values: 0x{:X}, 0x{:X}", last_stack_value, last_stack_value1);
        assert_eq!(last_stack_value, 0x02, "Lo byte on stack should be 0x02");
        assert_eq!(last_stack_value1, 0x80, "Hi byte on stack should be 0x80");
        assert_eq!(result_address, EXPECTED_ADDRESS, "Value of pc is 0x{:X} and should be 0x{:X}", result_address, EXPECTED_ADDRESS);
        assert_eq!(cycles, EXPECTED_CYCLES, "JSR $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    #[test]
    fn lda_immediate() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA9, EXPECTED_VALUE]); // LDA #8
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_zero_flag() {
        const EXPECTED_VALUE: u8 = 0;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA9, EXPECTED_VALUE]); // LDA #0
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should have been set, rest unchanged");
    }
    #[test]
    fn lda_sign_flag() {
        const EXPECTED_VALUE: u8 = 0b10000001;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA9, EXPECTED_VALUE]); // LDA #129 (0b10000001)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set, rest unchanged");
    }
    #[test]
    fn lda_zpg() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0008, &[EXPECTED_VALUE]); // zp $0x08 -> 27
        write_program(&mut cpu, 0x8000, &[0xA5, 0x08]); // LDA $0x08
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_zpg_x() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x000D, &[EXPECTED_VALUE]); // $08+x(5) -> $0D
        write_program(&mut cpu, 0x8000, &[0xB5, 0x08]); // LDA $08,X
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_zpg_x_wrap_around() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0003, &[EXPECTED_VALUE]); // $FE+x(5) -> $03
        write_program(&mut cpu, 0x8000, &[0xB5, 0xFE]); // LDA $FE,X
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_abs() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2004, &[EXPECTED_VALUE]); // $2004 -> 27
        write_program(&mut cpu, 0x8000, &[0xAD, 0x04, 0x20]); // LDA $2004
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_abs_x() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2009, &[EXPECTED_VALUE]); // $0x2004+x(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xBD, 0x04, 0x20]); // LDA $0x2004,x
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_abs_x_page_boundary() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2103, &[EXPECTED_VALUE]); // $20FE+x(5) -> $2103
        write_program(&mut cpu, 0x8000, &[0xBD, 0xFE, 0x20]); // LDA $20FE,x
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_abs_y() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2009, &[EXPECTED_VALUE]); // $0x2004+y(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xB9, 0x04, 0x20]); // LDA $0x2004,y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn lda_abs_y_page_boundary() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2103, &[EXPECTED_VALUE]); // $20FE+y(5) -> $2103
        write_program(&mut cpu, 0x8000, &[0xB9, 0xFE, 0x20]); // LDA $20FE,y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Clock LDY Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.a, EXPECTED_VALUE, "Value of accumulator A is {} and should be {}", cpu.a, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDA $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }

    //TODO: Test lda_ind_x and lda_ind_y

    #[test]
    fn ldy_immediate() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA0, EXPECTED_VALUE]); // LDY #8
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_zero_flag() {
        const EXPECTED_VALUE: u8 = 0;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA0, EXPECTED_VALUE]); // LDY #0
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should have been set, rest unchanged");
    }
    #[test]
    fn ldy_sign_flag() {
        const EXPECTED_VALUE: u8 = 0b10000001;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA0, EXPECTED_VALUE]); // LDY #129 (0b10000001)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set, rest unchanged");
    }
    #[test]
    fn ldy_zpg() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0008, &[EXPECTED_VALUE]); // zp $0x08 -> 27
        write_program(&mut cpu, 0x8000, &[0xA4, 0x08]); // LDY $0x08
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_zpg_x() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x000D, &[EXPECTED_VALUE]); // $08+x(5) -> $0D
        write_program(&mut cpu, 0x8000, &[0xB4, 0x08]); // LDY $08,X
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_zpg_x_wrap_around() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0003, &[EXPECTED_VALUE]); // $FE+x(5) -> $03
        write_program(&mut cpu, 0x8000, &[0xB4, 0xFE]); // LDY $FE,X
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $ZZ,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_abs() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2004, &[EXPECTED_VALUE]); // $2004 -> 27
        write_program(&mut cpu, 0x8000, &[0xAC, 0x04, 0x20]); // LDY $2004
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_abs_x() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2009, &[EXPECTED_VALUE]); // $0x2004+x(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xBC, 0x04, 0x20]); // LDY $0x2004,x
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldy_abs_x_page_boundary() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2103, &[EXPECTED_VALUE]); // $20FE+x(5) -> $2103
        write_program(&mut cpu, 0x8000, &[0xBC, 0xFE, 0x20]); // LDY $20FE,x
        cpu.x = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.y, EXPECTED_VALUE, "Value of index register Y is {} and should be {}", cpu.y, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_immediate() {
        const EXPECTED_VALUE: u8 = 8;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA2, EXPECTED_VALUE]); // LDX #8
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_zero_flag() {
        const EXPECTED_VALUE: u8 = 0;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA2, EXPECTED_VALUE]); // LDX #0
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::Z, end_status, "Zero flag should have been set, rest unchanged");
    }
    #[test]
    fn ldx_sign_flag() {
        const EXPECTED_VALUE: u8 = 0b10000001;
        const EXPECTED_CYCLES: usize = 2;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x8000, &[0xA2, EXPECTED_VALUE]); // LDX #129 (0b10000001)
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX #(IMM) expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status | Flags6502::N, end_status, "Sign flag should have been set, rest unchanged");
    }
    #[test]
    fn ldx_zpg() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 3;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0008, &[EXPECTED_VALUE]); // zp $0x08 -> 27
        write_program(&mut cpu, 0x8000, &[0xA6, 0x08]); // LDX $0x08
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Clock LDX Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX $ZZ expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_zpg_y() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x000D, &[EXPECTED_VALUE]); // $0x08+y(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xB6, 0x08]); // LDX $0x08,Y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX $ZZ,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_zpg_x_wrap_around() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x0003, &[EXPECTED_VALUE]); // $0xFE+y(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xB6, 0xFE]); // LDX $0xFE,Y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $ZZ,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_abs() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2004, &[EXPECTED_VALUE]); // $0x2004 -> 27
        write_program(&mut cpu, 0x8000, &[0xAE, 0x04, 0x20]); // LDX $2004
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDY $AAAA expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_abs_y() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 4;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2009, &[EXPECTED_VALUE]); // $0x2004+y(5) -> 27
        write_program(&mut cpu, 0x8000, &[0xBE, 0x04, 0x20]); // LDX $0x2004,y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX $AAAA,X expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
    #[test]
    fn ldx_abs_y_page_boundary() {
        const EXPECTED_VALUE: u8 = 27;
        const EXPECTED_CYCLES: usize = 5;
        let mut cpu = create_cpu();
        write_program(&mut cpu, 0x2103, &[EXPECTED_VALUE]); // $20FE+y(5) -> $2103
        write_program(&mut cpu, 0x8000, &[0xBE, 0xFE, 0x20]); // LDX $20FE,y
        cpu.y = 5;  // x = 5
        print_cpu(&cpu);
        _ = clock_instruction_debug(&mut cpu); // Clock brk instruction at 0x00
        let begin_status = cpu.get_status();
        print_cpu(&cpu);
        let cycles = clock_instruction_debug(&mut cpu); // Clock LDY Instruction
        print_cpu(&cpu);
        let end_status = cpu.get_status();
        assert_eq!(cpu.x, EXPECTED_VALUE, "Value of index register X is {} and should be {}", cpu.x, EXPECTED_VALUE);
        assert_eq!(cycles, EXPECTED_CYCLES, "LDX $AAAA,Y expected to take {} cycles, used {}", EXPECTED_CYCLES, cycles);
        assert_eq!(begin_status, end_status, "Status flags should not have changed!");
    }
}
*/
