#![no_std]

use cozy_chess::*;

mod wasm4;
mod game;
mod sprites;
mod sounds;
mod engine;
mod rng;

use wasm4::*;
use game::*;
use rng::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    VsPlayer,
    VsEngine
}

impl GameMode {
    pub const ALL: [GameMode; 2] = [
        GameMode::VsPlayer,
        GameMode::VsEngine
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Promotion {
    Queen,
    Knight,
    Rook,
    Bishop
}

impl Promotion {
    pub const ALL: [Promotion; 4] = [
        Promotion::Queen,
        Promotion::Knight,
        Promotion::Rook,
        Promotion::Bishop
    ];

    pub fn piece(&self) -> Piece {
        match self {
            Promotion::Queen => Piece::Queen,
            Promotion::Knight => Piece::Knight,
            Promotion::Rook => Piece::Rook,
            Promotion::Bishop => Piece::Bishop
        }
    }
}

enum MenuState {
    ModeSelect,
    ColorSelect,
    PromotionSelect(Promotion),
    GameOver
}

struct Glowfish {
    rng: Rng,
    menu: Option<MenuState>,
    game: ChessGame,
    mode: GameMode,
    engine_delay_timer: u32,
    p1_col: Color,
    selected_square: Square,
    selected_piece: Option<Square>,
    prev_gamepad_state: [GamepadState; 4]
}

const SCREEN_SIZE: u32 = 160;
const SQUARE_SIZE: u32 = 16;
const PIECE_SIZE: u32 = 14;
const BOARD_SIZE: u32 = SQUARE_SIZE * Rank::NUM as u32;
const BOARD_X: u32 = (SCREEN_SIZE - BOARD_SIZE) / 2;
const BOARD_Y: u32 = BOARD_X;
const OUTLINE_SIZE: u32 = 1;
const SELECTED_PIECE_OFFSET: i32 = -1;
const MENU_BORDER: u32 = 3;
const PROMOTION_MENU_X: u32 = BOARD_X + 2 * SQUARE_SIZE - MENU_BORDER;
const PROMOTION_MENU_Y: u32 = BOARD_Y + 3 * SQUARE_SIZE - MENU_BORDER;
const PROMOTION_MENU_W: u32 = 4 * SQUARE_SIZE + 2 * MENU_BORDER;
const PROMOTION_MENU_H: u32 = 1 * SQUARE_SIZE + 2 * MENU_BORDER;
const MENU_TITLE_PADDING: u32 = 4;
const MENU_ITEM_PADDING: u32 = 1;
const CHAR_WIDTH: u32 = 8;
const CHAR_HEIGHT: u32 = 8;
const MOVE_INDICATOR_SIZE: u32 = 4;

const LIGHT_SQUARE: DrawColor = DrawColor::One;
const DARK_SQUARE: DrawColor = DrawColor::Two;
const BLACK: DrawColor = DrawColor::Three;
const WHITE: DrawColor = DrawColor::Four;

const START_SQUARE: Square = Square::E2;

impl Glowfish {
    fn perspective(&self) -> Color {
        match self.mode {
            GameMode::VsPlayer => self.game.board().side_to_move(),
            GameMode::VsEngine => self.p1_col,
        }
    }

    fn active_player_gamepad(&self, ctx: &Wasm4) -> GamepadState {
        let gamepads = ctx.gamepad_state();
        let index = if self.p1_col == self.perspective() { 0 } else { 1 };
        gamepads[index].newly_pressed(self.prev_gamepad_state[index])
    }

    fn main_player_gamepad(&self, ctx: &Wasm4) -> GamepadState {
        let gamepads = ctx.gamepad_state();
        gamepads[0].newly_pressed(self.prev_gamepad_state[0])
    }

    fn update_game(&mut self, ctx: &mut Wasm4) {
        if self.mode == GameMode::VsEngine && self.p1_col != self.game.board().side_to_move() {
            self.engine_delay_timer += 1;
            if self.engine_delay_timer >= 30 {
                let mv = engine::best_move(&self.game, self.rng.next());
                self.try_play_move(ctx, mv);
                self.engine_delay_timer = 0;
            }
            return;
        }

        let pad = self.active_player_gamepad(ctx);
        let mut x_offset = 0;
        let mut y_offset = 0;
        x_offset -= pad.left() as i8;
        x_offset += pad.right() as i8;
        y_offset += pad.up() as i8;
        y_offset -= pad.down() as i8;
        if self.perspective() == Color::Black {
            x_offset *= -1;
            y_offset *= -1;
        }
        let init_square = self.selected_square;
        if let Some(selected_square) = self.selected_square.try_offset(x_offset, 0) {
            self.selected_square = selected_square;
        }
        if let Some(selected_square) = self.selected_square.try_offset(0, y_offset) {
            self.selected_square = selected_square;
        }
        if init_square != self.selected_square {
            ctx.tone(sounds::SELECT_MOVE);
        }

        if pad.button_x() {
            ctx.tone(sounds::PIECE_DESELECT);
            self.selected_piece = None;
        }

        if pad.button_z() {
            if let Some(selected_piece) = self.selected_piece {
                let mv = Move {
                    from: selected_piece,
                    to: self.selected_square,
                    promotion: None
                };
                let pawns = self.game.board().pieces(Piece::Pawn);
                let is_promotion = pawns.has(mv.from) && matches!(mv.to.rank(), Rank::First | Rank::Eighth);
                if is_promotion {
                    self.menu = Some(MenuState::PromotionSelect(Promotion::Queen));
                } else {
                    self.try_play_move(ctx, mv);
                }
            } else if self.game.board().colors(self.game.board().side_to_move()).has(self.selected_square) {
                ctx.tone(sounds::PIECE_SELECT);
                self.selected_piece = Some(self.selected_square);
            }
        }
    }

    fn try_play_move(&mut self, ctx: &mut Wasm4, mv: Move) {
        let pieces = self.game.board().occupied().popcnt();
        if self.game.try_play(mv) {
            let status = self.game.status();
            ctx.tone(match status {
                GameStatus::Won => sounds::CHECKMATE,
                GameStatus::Drawn => sounds::DRAW,
                GameStatus::Ongoing => {
                    if !self.game.board().checkers().is_empty() {
                        sounds::CHECK
                    } else if self.game.board().occupied().popcnt() < pieces {
                        sounds::CAPTURE
                    } else {
                        sounds::MOVE
                    }
                }
            });
            if status != GameStatus::Ongoing {
                self.menu = Some(MenuState::GameOver);
            }
            self.selected_piece = None;
        } else {
            ctx.tone(sounds::ILLEGAL_MOVE);
        }
    }

    fn draw(&self, ctx: &mut Wasm4) {
        self.draw_board_base(ctx);
        self.highlight_square(ctx, self.selected_square);
        if let Some(square) = self.selected_piece {
            self.highlight_square(ctx, square);
        }
        self.draw_pieces(ctx);
        if let Some(square) = self.selected_piece {
            let piece_color = self.game.board().color_on(square).unwrap();
            let (main_col, inverted_col) = match piece_color {
                Color::White => (WHITE, BLACK),
                Color::Black => (BLACK, WHITE)
            };
            self.game.board().generate_moves_for(square.bitboard(), |moves| {
                for mv in moves {
                    let (fill, outline) = if self.game.board().color_on(mv.to) == Some(piece_color) {
                        (main_col, inverted_col)
                    } else {
                        (inverted_col, main_col)
                    };
                    let (x, y) = self.square_coords(mv.to);
                    let offset = (SQUARE_SIZE - MOVE_INDICATOR_SIZE) as i32 / 2;
                    ctx.draw_2bpp_sprite(
                        sprites::MOVE_INDICATOR,
                        offset + x,
                        offset + y,
                        [DrawColor::None, fill, outline, DrawColor::None]
                    );
                }
                false
            });
        }
        match &self.menu {
            Some(MenuState::ModeSelect) => {
                self.draw_menu(
                    ctx,
                    &["Mode select"], 
                    &[
                        "VS Player",
                        "VS CPU"
                    ],
                    self.mode as usize
                );
            }
            Some(MenuState::ColorSelect) => {
                self.draw_menu(
                    ctx,
                    &["Color select"], 
                    &[
                        "White",
                        "Black"
                    ],
                    self.p1_col as usize
                );
            }
            Some(MenuState::PromotionSelect(current_promotion)) => {
                let (piece_color, inverted_color) = match self.game.board().side_to_move() {
                    Color::White => (WHITE, BLACK),
                    Color::Black => (BLACK, WHITE)
                };
                ctx.rect(
                    PROMOTION_MENU_X as i32,
                    PROMOTION_MENU_Y as i32,
                    PROMOTION_MENU_W,
                    PROMOTION_MENU_H,
                    inverted_color,
                    piece_color
                );
                for (i, promotion) in Promotion::ALL.iter().enumerate() {
                    let sprite = sprites::PIECE_SPRITES[promotion.piece() as usize];
                    let square_x = (PROMOTION_MENU_X + MENU_BORDER + i as u32 * SQUARE_SIZE) as i32;
                    let square_y = (PROMOTION_MENU_Y + MENU_BORDER) as i32;
                    let offset = (SQUARE_SIZE - PIECE_SIZE) as i32 / 2;
                    let piece_x = square_x + offset;
                    let mut piece_y = square_y + offset;
                    if promotion == current_promotion {
                        piece_y += SELECTED_PIECE_OFFSET;
                        ctx.rect(
                            square_x,
                            square_y,
                            SQUARE_SIZE,
                            SQUARE_SIZE,
                            DrawColor::None, 
                            LIGHT_SQUARE
                        );
                    }
                    ctx.draw_2bpp_sprite(
                        sprite,
                        piece_x,
                        piece_y, 
                        [DrawColor::None, piece_color, inverted_color, DrawColor::None]
                    );
                }
            }
            Some(MenuState::GameOver) => {
                let title = match self.game.status() {
                    GameStatus::Won => match self.game.board().side_to_move() {
                        Color::Black => "White wins",
                        Color::White => "Black wins"
                    }
                    GameStatus::Drawn => "Drawn game",
                    GameStatus::Ongoing => unreachable!()
                };
                self.draw_menu(ctx, &[title], &["New game"], 0);
            }
            None => {}
        }
    }

    fn draw_menu(&self, ctx: &mut Wasm4, title: &[&str], options: &[&str], selected: usize) {
        let max_str_width = title.iter().chain(options).map(|s| s.len()).max().unwrap();
        let inner_width = max_str_width as u32 * CHAR_WIDTH;
        let inner_height = (title.len() + options.len()) as u32 * (CHAR_HEIGHT + MENU_ITEM_PADDING) + MENU_TITLE_PADDING;
        let inner_x = (SCREEN_SIZE - inner_width) / 2;
        let inner_y = (SCREEN_SIZE - inner_height) / 2;
        ctx.rect(
            (inner_x - MENU_BORDER) as i32,
            (inner_y - MENU_BORDER) as i32,
            inner_width + 2 * MENU_BORDER,
            inner_height + 2 * MENU_BORDER,
            BLACK,
            WHITE
        );
        for (row, line) in title.iter().chain(options).enumerate() {
            let width = line.len() as u32 * CHAR_WIDTH;
            let x_offset = (inner_width - width) / 2;
            let is_title = row < title.len();
            let x = inner_x + x_offset;
            let mut y = inner_y + row as u32 * (CHAR_HEIGHT + MENU_ITEM_PADDING);
            if !is_title {
                y += MENU_TITLE_PADDING;
            }
            let (fill, background) = if is_title {
                (WHITE, DrawColor::None)
            } else if (row - title.len()) == selected {
                (DARK_SQUARE, WHITE)
            } else {
                (LIGHT_SQUARE, DrawColor::None)
            };
            ctx.text(line, x as i32, y as i32, fill, background);
        }
    }
    
    fn draw_board_base(&self, ctx: &mut Wasm4) {
        ctx.rect(
            (BOARD_X - OUTLINE_SIZE) as i32,
            (BOARD_Y - OUTLINE_SIZE) as i32, 
            BOARD_SIZE + OUTLINE_SIZE * 2, 
            BOARD_SIZE + OUTLINE_SIZE * 2,
            DrawColor::None,
            BLACK
        );
        for square in BitBoard::FULL {
            if square.file() as u8 % 2 == square.rank() as u8 % 2 {
                let (x, y) = self.square_coords(square);
                ctx.rect(x, y, SQUARE_SIZE, SQUARE_SIZE, DARK_SQUARE, DARK_SQUARE);
            }
        }
    }

    fn draw_pieces(&self, ctx: &mut Wasm4) {
        for color in Color::ALL {
            for piece in Piece::ALL {
                let sprite = sprites::PIECE_SPRITES[piece as usize];
                for square in self.game.board().colors(color) & self.game.board().pieces(piece) {
                    let (mut x, mut y) = self.square_coords(square);
                    let offset = (SQUARE_SIZE - PIECE_SIZE) as i32 / 2;
                    x += offset;
                    y += offset;
                    if Some(square) == self.selected_piece {
                        y += SELECTED_PIECE_OFFSET;
                    }

                    let (piece_color, inverted_color) = match color {
                        Color::White => (WHITE, BLACK),
                        Color::Black => (BLACK, WHITE)
                    };
                    ctx.draw_2bpp_sprite(
                        sprite,
                        x,
                        y, 
                        [DrawColor::None, piece_color, inverted_color, DrawColor::None]
                    );
                }
            }
        }
    }
    
    fn square_coords(&self, square: Square) -> (i32, i32) {
        let square = match self.perspective() {
            Color::White => square,
            Color::Black => square.flip_file().flip_rank()
        };
        let x = square.file() as i32 * SQUARE_SIZE as i32;
        let y = square.rank().flip() as i32 * SQUARE_SIZE as i32;
        (BOARD_X as i32 + x, BOARD_Y as i32 + y)
    }

    fn highlight_square(&self, ctx: &mut Wasm4, square: Square) {
        let outline = match self.game.board().color_on(square) {
            Some(Color::White) | None => BLACK,
            Some(Color::Black) => WHITE
        };
        let (x, y) = self.square_coords(square);
        ctx.rect(x, y, SQUARE_SIZE, SQUARE_SIZE, DrawColor::None, outline);
    }
}

impl Runtime for Glowfish {
    fn init(ctx: &mut Wasm4) -> Self {
        ctx.set_palette([0xDA5630, 0xA22200, 0x000000, 0xFFFFFF]);
        Self {
            rng: Rng::new(),
            menu: Some(MenuState::ModeSelect),
            mode: GameMode::VsPlayer,
            p1_col: Color::White,
            engine_delay_timer: 0,
            game: ChessGame::new(),
            selected_square: START_SQUARE,
            selected_piece: None,
            prev_gamepad_state: [GamepadState::default(); 4]
        }
    }

    fn update(&mut self, ctx: &mut Wasm4) {
        self.rng.next();
        let menu_pad = self.main_player_gamepad(ctx);
        let player_pad = self.active_player_gamepad(ctx);
        match &mut self.menu {
            Some(MenuState::ModeSelect) => {
                let mut shift = 0;
                shift -= menu_pad.up() as usize;
                shift += menu_pad.down() as usize;
                if shift != 0 {
                    let index = (self.mode as usize + shift)
                        .rem_euclid(GameMode::ALL.len());
                    self.mode = GameMode::ALL[index];
                    ctx.tone(sounds::SELECT_MOVE);
                }
                if menu_pad.button_z() {
                    self.menu = Some(MenuState::ColorSelect);
                    ctx.tone(sounds::SELECT_MOVE);
                }
            }
            Some(MenuState::ColorSelect) => {
                let mut shift = 0;
                shift -= menu_pad.up() as usize;
                shift += menu_pad.down() as usize;
                if shift != 0 {
                    let index = (self.p1_col as usize + shift)
                        .rem_euclid(Color::ALL.len());
                    self.p1_col = Color::ALL[index];
                    ctx.tone(sounds::SELECT_MOVE);
                }
                if menu_pad.button_x() {
                    self.menu = Some(MenuState::ModeSelect);
                    ctx.tone(sounds::SELECT_MOVE);
                } else if menu_pad.button_z() {
                    self.menu = None;
                    ctx.tone(sounds::SELECT_MOVE);
                }
            }
            Some(MenuState::PromotionSelect(promotion)) => {
                if self.game.status() != GameStatus::Ongoing {
                    ctx.tone(sounds::CHECKMATE);
                    self.menu = Some(MenuState::GameOver);
                } else { 
                    let mut shift = 0;
                    shift -= player_pad.left() as usize;
                    shift += player_pad.right() as usize;
                    if shift != 0 {
                        let index = (*promotion as usize + shift)
                            .rem_euclid(Promotion::ALL.len());
                        *promotion = Promotion::ALL[index];
                        ctx.tone(sounds::SELECT_MOVE);
                    }
                    if player_pad.button_x() {
                        self.menu = None;
                        ctx.tone(sounds::PIECE_DESELECT);
                    } else if player_pad.button_z() {
                        let mv = Move {
                            from: self.selected_piece.unwrap(),
                            to: self.selected_square,
                            promotion: Some(promotion.piece())
                        };
                        self.menu = None;
                        self.try_play_move(ctx, mv);
                    }
                }
            },
            Some(MenuState::GameOver) => {
                if menu_pad.button_z() {
                    self.menu = Some(MenuState::ModeSelect);
                    self.game = ChessGame::new();
                    self.selected_square = START_SQUARE;
                    self.selected_piece = None;
                }
            }
            None => self.update_game(ctx)
        }
        self.prev_gamepad_state = ctx.gamepad_state();
        self.draw(ctx);
    }
}

wasm4_main!(Glowfish);
