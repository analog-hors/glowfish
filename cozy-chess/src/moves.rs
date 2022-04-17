use crate::*;

const fn sliding_moves(square: Square, mut blockers: BitBoard, deltas: &[(i8, i8); 4]) -> BitBoard {
    blockers.0 &= !square.bitboard().0;
    let mut moves = BitBoard::EMPTY;
    let mut i = 0;
    while i < deltas.len() {
        let (df, dr) = deltas[i];
        let mut square = square;
        while !blockers.has(square) {
            if let Some(sq) = square.try_offset(df, dr) {
                square = sq;
                moves.0 |= square.bitboard().0;
            } else {
                break;
            }
        }
        i += 1;
    }
    moves
}

/// Get the moves for a rook on some square.
/// ```
/// # use cozy_chess::*;
/// let blockers = bitboard! {
///     . . . X . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . X
///     . . . . . X . .
///     . . . . . . . .
/// };
/// let moves = get_rook_moves(Square::D3, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// });
/// ```
pub const fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    sliding_moves(square, blockers, &[
        (1, 0),
        (0, -1),
        (-1, 0),
        (0, 1)
    ])
}

/// Get the moves for a bishop on some square.
/// ```
/// # use cozy_chess::*;
/// let blockers = bitboard! {
///     . . . . . . . .
///     . . . . . . . X
///     . . X . . . . .
///     . . . . . X . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . X . .
/// };
/// let moves = get_bishop_moves(Square::D3, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     X . . . . . . .
///     . X . . . X . .
///     . . X . X . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
/// });
/// ```
pub const fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    sliding_moves(square, blockers, &[
        (1, 1),
        (1, -1),
        (-1, -1),
        (-1, 1)
    ])
}

/// Get the rays for a rook on some square.
/// ```
/// # use cozy_chess::*;
/// let rays = get_rook_rays(Square::D3);
/// assert_eq!(rays, bitboard! {
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// });
/// ```
pub const fn get_rook_rays(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let square = Square::index_const(i);
            table[i].0 = square.rank().bitboard().0 ^ square.file().bitboard().0;
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the rays for a bishop on some square.
/// ```
/// # use cozy_chess::*;
/// let rays = get_bishop_rays(Square::D3);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . X
///     X . . . . . X .
///     . X . . . X . .
///     . . X . X . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
/// });
/// ```
pub const fn get_bishop_rays(square: Square) -> BitBoard {
    const fn get_bishop_rays(square: Square) -> BitBoard {
        let mut rays = BitBoard::EMPTY.0;
        let mut i = 0;
        while i < Square::NUM {
            let target = Square::index_const(i);
            let rd = (square.rank() as i8 - target.rank() as i8).abs();
            let fd = (square.file() as i8 - target.file() as i8).abs();
            if rd == fd && rd != 0 {
                rays |= 1 << i;
            }
            i += 1;
        }
        BitBoard(rays)
    }
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            table[i] = get_bishop_rays(Square::index_const(i));
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get all squares between two squares, if reachable via a ray.
/// ```
/// # use cozy_chess::*;
/// let rays = get_between_rays(Square::B4, Square::G4);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X X X X . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
pub const fn get_between_rays(from: Square, to: Square) -> BitBoard {
    let blockers = BitBoard(from.bitboard().0 ^ to.bitboard().0);
    let bishop_ray = get_bishop_moves(from, blockers);
    if bishop_ray.has(to) {
        return BitBoard(bishop_ray.0 & get_bishop_moves(to, blockers).0);
    }
    let rook_ray = get_rook_moves(from, blockers);
    if rook_ray.has(to) {
        return BitBoard(rook_ray.0 & get_rook_moves(to, blockers).0);
    }
    BitBoard::EMPTY
}

/// Get a ray on the board that passes through both squares, if it exists.
/// ```
/// # use cozy_chess::*;
/// let rays = get_line_rays(Square::D2, Square::G5);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . X
///     . . . . . . X .
///     . . . . . X . .
///     . . . . X . . .
///     . . . X . . . .
///     . . X . . . . .
/// });
/// ```
pub const fn get_line_rays(from: Square, to: Square) -> BitBoard {
    let rays = get_bishop_rays(from);
    if rays.has(to) {
        return BitBoard((rays.0 | from.bitboard().0) & (get_bishop_rays(to).0 | to.bitboard().0));
    }
    let rays = get_rook_rays(from);
    if rays.has(to) {
        return BitBoard((rays.0 | from.bitboard().0) & (get_rook_rays(to).0 | to.bitboard().0));
    }
    BitBoard::EMPTY
}

/// Get the knight moves for a knight on some square.
/// ```
/// # use cozy_chess::*;
/// let moves = get_knight_moves(Square::D3);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
///     . . . . . . . .
///     . X . . . X . .
///     . . X . X . . .
/// });
/// ```
pub const fn get_knight_moves(square: Square) -> BitBoard {
    const fn get_knight_moves(square: Square) -> BitBoard {
        const KNIGHT_DELTAS: [(i8, i8); 8] = [
            (-1, 2),
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1)
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < KNIGHT_DELTAS.len() {
            let (df, dr) = KNIGHT_DELTAS[i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            table[i] = get_knight_moves(Square::index_const(i));
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the king moves for a king on some square.
/// ```
/// # use cozy_chess::*;
/// let moves = get_king_moves(Square::D3);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X X X . . .
///     . . X . X . . .
///     . . X X X . . .
///     . . . . . . . .
/// });
/// ```
pub const fn get_king_moves(square: Square) -> BitBoard {
    const fn get_king_moves(square: Square) -> BitBoard {
        const KING_DELTAS: [(i8, i8); 8] = [
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1)
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < KING_DELTAS.len() {
            let (df, dr) = KING_DELTAS[i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            table[i] = get_king_moves(Square::index_const(i));
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the pawn attacks for a pawn on some square.
/// ```
/// # use cozy_chess::*;
/// let attacks = get_pawn_attacks(Square::D3, Color::White);
/// assert_eq!(attacks, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X . X . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
pub const fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
    const fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
        const PAWN_DELTAS: [[(i8, i8); 2]; Color::NUM] = [
            [(1, 1), (-1, 1)],
            [(1, -1), (-1, -1)]
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < PAWN_DELTAS[color as usize].len() {
            let (df, dr) = PAWN_DELTAS[color as usize][i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut c = 0;
        while c < table.len() {
            let mut i = 0;
            while i < table[c].len() {
                table[c][i] = get_pawn_attacks(
                    Square::index_const(i),
                    Color::index_const(c)
                );
                i += 1;
            }
            c += 1;
        }
        table
    };
    TABLE[color as usize][square as usize]
}

/// Get the pawn forward moves/non-captures for a pawn of some color on some square.
/// ```
/// # use cozy_chess::*;
/// let moves = get_pawn_quiets(Square::D2, Color::White, BitBoard::EMPTY);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// 
/// let blockers = bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// };
/// let moves = get_pawn_quiets(Square::D7, Color::Black, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
pub const fn get_pawn_quiets(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    let square_bb = square.bitboard();
    let mut moves = BitBoard(if let Color::White = color {
        square_bb.0 << File::NUM
    } else {
        square_bb.0 >> File::NUM
    });
    moves.0 &= !blockers.0;
    if !moves.is_empty() && Rank::Second.relative_to(color).bitboard().has(square) {
        moves.0 |= if let Color::White = color {
            moves.0 << File::NUM
        } else {
            moves.0 >> File::NUM
        };
        moves.0 &= !blockers.0;
    }
    moves
}
