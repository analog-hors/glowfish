use crate::wasm4::*;

pub const SELECT_MOVE: Tone = Tone {
    start_freq: 300,
    end_freq: 300,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 5,
    channel: Channel::Triangle,
};

pub const PIECE_SELECT: Tone = Tone {
    start_freq: 150,
    end_freq: 400,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 10,
    channel: Channel::Triangle,
};

pub const PIECE_DESELECT: Tone = Tone {
    start_freq: 300,
    end_freq: 150,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 10,
    channel: Channel::Triangle,
};

pub const MOVE: Tone = Tone {
    start_freq: 300,
    end_freq: 600,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 10,
    channel: Channel::Triangle,
};

pub const ILLEGAL_MOVE: Tone = Tone {
    start_freq: 150,
    end_freq: 20,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 10,
    channel: Channel::Triangle,
};

pub const CAPTURE: Tone = Tone {
    start_freq: 300,
    end_freq: 100,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 15,
    channel: Channel::PulseOne,
};

pub const CHECK: Tone = Tone {
    start_freq: 150,
    end_freq: 150,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 10,
    channel: Channel::PulseOne,
};

pub const CHECKMATE: Tone = Tone {
    start_freq: 300,
    end_freq: 100,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 30,
    channel: Channel::Noise,
};

pub const DRAW: Tone = Tone {
    start_freq: 300,
    end_freq: 250,
    attack: 0,
    decay: 0,
    sustain: 0,
    release: 100,
    channel: Channel::Triangle,
};
