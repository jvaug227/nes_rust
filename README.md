# nes_rust
## What is it
An NES emulator written in rust! Well, it might be one day. Right now, it's actually an emulator of an NES-6502 microprocessor.  

This started as a learning project back in 2022 when I was first learning the rust language; it was a mess of tutorial code and 
some rust technologies I had just picked up. I have since revived the project with new goals and motiviation to make a complete product.  

## Project Goals
- Half-cycle instruction emulation,
- Implements all 256 opcodes for the NES-6502 along with its bugs,
- Can play SMB3

## Overview
A lot of emulators I have referenced like to emulate entire instructions in a single call and make IO calls to an internally owned 'BUS'. 
While this is alright if just emulating the 6502 microprocessor, it would be a pain to time IO calls and indicate dummy reads/writes.  

Emulating instructions by the cycle solves most problems with the read/write timings with IO, but I was looking into how cartridges work with
supplying ROM/RAM as well as cycle timings and I came across the article [A new cycle-stepped 6502 CPU emulator](https://floooh.github.io/2019/12/13/cycle-stepped-6502.html)
by Brain Dump. Here, I saw how the buses were being treated just as a mutable integer representing the pins of the chip.

The idea of the buses being parameterized integers combined with a few posts I read between the Nesdev wiki and forums talking about how the cpu times itself with the bus using
the phi 0, 1, and 2 cycles. I knew I wanted to emulate the cpu <-> bus interactions correctly using half-cycle timings.

Using 1/2 cycles with an external bus found the benfits that my cpu struct became plain-old-data that was easily copyable, as well as
the instructions becoming increasingly repetitive in nature. Another benefit was that the bus is now a part of the higher picture that users 
can configure as needed - e.g. I found it made grabbing a copy of the cpu at the start of instructions for debugging purposes a simple task. Listening in on 
bus reads/writes is also as easy as just inserting a print statement after each 1/2 cycle, phi1 for the address+read byte, and phi2 for the byte written.
No need for pass callback functions or modifying a deeply embedded read/write function.

```rust
fn my_clock(&mut self) {
  let address_bus = 0;
  let rw_pin = true; // 1 on r/w indicates a read
  let data_bus = 0;

  let phi = false; // phi1
  cpu.clock(&mut address_bus, &mut data_bus, &mut rw_pin, phi1);

  if rw_pin {
    // Cpu is configured to read,
    // expect a valid address and put something on data_bus
  }

  let phi = true; // phi2
  cpu.clock(&mut address_bus, &mut data_bus, &mut rw_pin, phi1);
  if !rw_pin {
    // Cpu is configured to write
    // expect valid data and address
  }
}
```

## Immediate Goals
- Try using u8::overflowing_add to get overflows as an immediate result.
- Finish implementing opcodes (addressing should be complete minus yet-to-be-caught/verified bugs)
- Splitting cycle handling into clock_low() and clock_high() rather than relying on a phi boolean
- Finish decoding Cartidge data
- More tests, finishing tests, semi-automating tests.
- Separate project with TUI for better display.

## References
NESDev wiki: https://www.nesdev.org  
"A new cycle-stepped 6502 CPU emulator": https://floooh.github.io/2019/12/13/cycle-stepped-6502.html  
This thing: http://www.6502.org/users/andre/petindex/local/64doc.txt  
c74 Project & microcode documentation: https://c74project.com/microcode/ & https://c74project.com/wp-content/uploads/2020/04/c74-6502-microcode.pdf  

