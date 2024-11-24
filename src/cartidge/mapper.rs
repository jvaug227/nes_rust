trait Mapper {
    fn access_prg(&mut self, address: u16, data: &mut u8, rw: bool);
    fn access_chr(&mut self, address: u16, data: &mut u8, rw: bool);
}

pub struct MapperNROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper for MapperNROM {
    fn access_prg(&mut self, address: u16, data: &mut u8, rw: bool) {
        
    }
    fn access_chr(&mut self, address: u16, data: &mut u8, rw: bool) {
        
    }
} 
