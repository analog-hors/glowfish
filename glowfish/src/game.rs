use heapless::Vec;
use cozy_chess::*;

pub struct ChessGame {
    board: Board,
    history: Vec<u64, 150>
}

impl ChessGame {
    pub fn new() -> Self {
        let board = Board::default();
        let mut history = Vec::new();
        history.push(board.hash()).unwrap();
        Self {
            history,
            board
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn history(&self) -> &[u64] {
        &self.history
    }

    pub fn try_play(&mut self, mv: Move) -> bool {
        if self.status() != GameStatus::Ongoing {
            return false;
        }
        if !self.board.try_play(mv).unwrap() {
            return false;
        }
        if self.board.halfmove_clock() == 0 {
            self.history.clear();
        }
        self.history.push(self.board.hash()).unwrap();
        true
    }

    pub fn status(&self) -> GameStatus {
        let bishops = self.board.pieces(Piece::Bishop);
        let knights = self.board.pieces(Piece::Knight);
        match self.board.occupied().popcnt() {
            2 => return GameStatus::Drawn,
            3 => if !(bishops | knights).is_empty() {
                return GameStatus::Drawn
            }
            _ => {}
        }
        let current = *self.history.last().unwrap();
        let repetitions = self.history.iter()
            .filter(|&&h| h == current)
            .count();
        if repetitions >= 3 {
            return GameStatus::Drawn;
        }
        self.board.status()
    }
}

