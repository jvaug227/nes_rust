# nes_rust
## What is it
An NES emulator written in rust!
It's specifically a library for the NMOS 6502 processor and other components used to compose the NES. The
actual NES behavior comes from the runrom example where the components are interconnected.

This started as a learning project back in 2022 when I was first learning the rust language; it was a mess of tutorial code and 
some rust technologies I had just picked up. I have since revived the project with new goals and motiviation to make a complete product.  

## Overview
This emulation of the NMOS 6502 is focused on half-cycle timing - splitting up the processing of instructions into high and low portions of the clock cycle.
I chose this level of accuracy because it should be able to emulate most of the instructions perfectly as well as the irregularities present in the original
silicon.  

A lot of emulators I have referenced like to emulate entire instructions in a single call and make IO calls to an internally owned/shared 'BUS' at instant speed. 
The problem with single-call instructions is that they lose a lot of the nuance of cycle timings and emulators have to do a lot of work to ensure that unexpected
events like interrupts and IO are handled properly. This level of emulation also generally couples all the components in a monolithic structure.

Emulating instructions by the cycle solves most problems with the read/write timings with IO, but I was looking into how cartridges work with
supplying ROM/RAM as well as cycle timings and I came across the article [A new cycle-stepped 6502 CPU emulator](https://floooh.github.io/2019/12/13/cycle-stepped-6502.html)
by Brain Dump. Inspiration was taken from the idea of passing the 'pins' of the chip as a refernce to some POD.

```rust
fn my_clock(&mut self) {
  let address_bus = 0;
  let rw_pin = true; // 1 on r/w indicates a read
  let data_bus = 0;

  let phi = false; // phi1
  cpu.clock(&mut address_bus, &mut data_bus, &mut rw_pin, phi);

  if rw_pin {
    // Cpu is configured to read,
    // expect a valid address and put something on data_bus
  }

  let phi = true; // phi2
  cpu.clock(&mut address_bus, &mut data_bus, &mut rw_pin, phi);
  if !rw_pin {
    // Cpu is configured to write
    // expect valid data and address
  }
}
```

## Project Goals
- [ ] Half-cycle instruction emulation
- [ ] Implements all 256 opcodes for the NES' 6502 along with its bugs
- [ ] Can play SMB3
- [ ] Audio
- [ ] OpenBus behavior
- [ ] The most popular mappers
- [ ] Official desktop app (Separate library)

## Immediate Goals
- [x] ~~Try using u8::overflowing_add to get overflows as an immediate result.~~ Should now be a feature flag.
- [x] ~~Finish implementing opcodes~~ All opcodes are implemented, though some improperly in terms of the openbus.
- [ ] Splitting cycle handling into clock_low() and clock_high() rather than relying on a phi boolean
- [ ] Finish decoding Cartidge data
- [ ] Official testing infrastrucutre and semi-automating testing.

## References
NESDev wiki: https://www.nesdev.org  
"A new cycle-stepped 6502 CPU emulator": https://floooh.github.io/2019/12/13/cycle-stepped-6502.html  
This thing: http://www.6502.org/users/andre/petindex/local/64doc.txt  
c74 Project & microcode documentation: https://c74project.com/microcode/ & https://c74project.com/wp-content/uploads/2020/04/c74-6502-microcode.pdf  

