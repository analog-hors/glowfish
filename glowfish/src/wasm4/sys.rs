macro_rules! memory_map {
    ($($addr:expr => $name:ident: $type:ty),*$(,)?) => {
        $(pub const $name: *mut $type = $addr as _;)*
    }
}

memory_map! {
    0x04 => PALETTE: [u32; 4],
    0x14 => DRAW_COLORS: u16,
    0x16 => GAMEPADS: [u8; 4],
}

extern "C" {
    pub fn traceUtf8(str: *const u8, length: u32);
    pub fn rect(x: i32, y: i32, width: u32, height: u32);
    pub fn blit(sprite: *const u8, x: i32, y: i32, width: u32, height: u32, flags: u32);
    pub fn textUtf8(str: *const u8, length: u32, x: i32, y: i32);
    pub fn tone(frequency: u32, duration: u32, volume: u32, flags: u32);
}
