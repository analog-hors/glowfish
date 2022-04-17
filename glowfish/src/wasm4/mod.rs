mod sys;

#[derive(Debug, Clone, Copy)]
pub enum DrawColor {
    None,
    One,
    Two,
    Three,
    Four
}

#[derive(Debug, Clone, Copy)]
pub struct TwoBppSprite<'s> {
    data: &'s [u8],
    width: u32,
    height: u32
}

impl<'s> TwoBppSprite<'s> {
    pub const fn new(data: &'s [u8], width: u32, height: u32) -> Self {
        if data.len() as u32 * (u8::BITS as u32) / 2 < width * height {
            panic!("Insufficient data for sprite size");
        }
        Self {
            data,
            width,
            height
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct GamepadState(u8);

macro_rules! impl_gamepad_state {
    ($($index:expr => $button:ident),*$(,)?) => {
        impl GamepadState {$(
            pub fn $button(&self) -> bool {
                (self.0 & (1 << $index)) != 0
            }
        )*}
    }
}

impl_gamepad_state! {
    0 => button_x,
    1 => button_z,
    4 => left,
    5 => right,
    6 => up,
    7 => down
}

impl GamepadState {
    pub fn newly_pressed(&self, prev: GamepadState) -> GamepadState {
        GamepadState(self.0 & !prev.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    PulseOne,
    PulseTwo,
    Triangle,
    Noise
}

#[derive(Debug, Clone, Copy)]
pub struct Tone {
    pub start_freq: u16,
    pub end_freq: u16,
    pub attack: u16,
    pub decay: u16,
    pub sustain: u16,
    pub release: u16,
    pub channel: Channel
}

pub struct Wasm4(());

impl Wasm4 {
    pub fn set_palette(&mut self, palette: [u32; 4]) {
        unsafe { sys::PALETTE.write_volatile(palette); }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, fill: DrawColor, outline: DrawColor) {
        unsafe {
            sys::DRAW_COLORS.write_volatile((outline as u16) << 4 | (fill as u16));
            sys::rect(x, y, w, h);
        }
    }

    pub fn draw_2bpp_sprite(&mut self, sprite: TwoBppSprite, x: i32, y: i32, color_map: [DrawColor; 4]) {
        unsafe {
            let mut colors = 0;
            for (i, &c) in color_map.iter().enumerate() {
                colors |= (c as u16) << (i * 4);
            }
            sys::DRAW_COLORS.write_volatile(colors);
            sys::blit(sprite.data.as_ptr(), x, y, sprite.width, sprite.height, 0b0001);
        }
    }

    pub fn text(&mut self, str: &str, x: i32, y: i32, fill: DrawColor, background: DrawColor) {
        unsafe {
            sys::DRAW_COLORS.write_volatile((background as u16) << 4 | (fill as u16));
            sys::textUtf8(str.as_ptr(), str.len() as u32, x, y);
        }
    }

    pub fn tone(&self, tone: Tone) {
        let frequency = (tone.end_freq as u32) << 16 | tone.start_freq as u32;
        let mut duration = tone.attack as u32;
        duration = (duration << 16) | tone.decay as u32;
        duration = (duration << 16) | tone.sustain as u32;
        duration = (duration << 16) | tone.release as u32;
        unsafe { sys::tone(frequency, duration, 100, tone.channel as u32); }
    }

    pub fn gamepad_state(&self) -> [GamepadState; 4] {
        unsafe { sys::GAMEPADS.read_volatile().map(GamepadState) }
    }
}

pub fn trace(str: &str) {
    unsafe { sys::traceUtf8(str.as_ptr(), str.len() as u32); }
}

pub static mut WASM_4: Wasm4 = Wasm4(());

pub trait Runtime {
    fn init(ctx: &mut Wasm4) -> Self;
    fn update(&mut self, ctx: &mut Wasm4);
}

#[macro_export]
macro_rules! __wasm4_main {
    ($runtime:ty) => {
        static mut RUNTIME: Option<$runtime> = None;

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            core::arch::wasm32::unreachable()
        }

        #[no_mangle]
        pub unsafe fn update() {
            let ctx = &mut WASM_4;
            let runtime = RUNTIME.get_or_insert_with(|| <$runtime>::init(ctx));
            runtime.update(ctx);
        }
    }
}

pub use __wasm4_main as wasm4_main;
