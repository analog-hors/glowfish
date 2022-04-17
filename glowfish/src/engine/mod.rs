use cozy_chess::*;

use heapless::Vec;
use crate::game::ChessGame;

mod eval;

type History = Vec::<u64, 150>;

include!(concat!(env!("OUT_DIR"), "/book.rs"));

pub fn best_move(game: &ChessGame, random: u64) -> Move {
    let moves = book_entry(game.board());
    if moves.len() > 0 {
        return moves[(random % moves.len() as u64) as usize];
    }
    let mut history = History::from_slice(game.history()).unwrap();
    history.pop();
    search(&mut history, game.board(), 2, 0, -15_000, 15_000).0.unwrap()
}

fn search(history: &mut History, board: &Board, depth: u8, ply_index: u8, mut alpha: i16, beta: i16) -> (Option<Move>, i16) {
    match board.status() {
        GameStatus::Won => return (None, -10_000 + ply_index as i16),
        GameStatus::Drawn => return (None, 0),
        GameStatus::Ongoing => {}
    }
    let repetitions = history.iter()
        .filter(|&&h| h == board.hash())
        .count();
    if repetitions >= 2 {
        return (None, 0);
    }
    if depth == 0 {
        return (None, qsearch(board, ply_index + 1, -beta, -alpha));
    }
    history.push(board.hash()).unwrap();
    let mut best_move = None;
    let mut value = i16::MIN;
    board.generate_moves(|moves| {
        for mv in moves {
            let mut child = board.clone();
            child.play_unchecked(mv);
            let (_, mut child_value) = search(
                history,
                &child,
                depth - 1, 
                ply_index + 1,
                -beta,
                -alpha
            );
            child_value *= -1;
            if child_value > value {
                value = child_value;
                best_move = Some(mv);
                if value > alpha {
                    alpha = value;
                    if alpha >= beta {
                        return true;
                    }
                }
            }
        }
        false
    });
    history.pop();
    (best_move, value)
}

fn qsearch(
    board: &Board,
    ply_index: u8,
    mut alpha: i16,
    beta: i16
) -> i16 {
    match board.status() {
        GameStatus::Won => return -10_000 + ply_index as i16,
        GameStatus::Drawn => return 0,
        GameStatus::Ongoing => {}
    }
    let mut best_eval = eval::evaluate(board);
    if ply_index >= 20 {
        return best_eval;
    }
    alpha = alpha.max(best_eval);
    if alpha >= beta {
        return best_eval;
    }

    let victims = board.colors(!board.side_to_move());
    'search: for &victim in Piece::ALL.iter().rev() {
        if victim == Piece::King {
            continue;
        }
        let victims = victims & board.pieces(victim);
        for &attacker in &Piece::ALL {
            let attackers = board.pieces(attacker);
            let stop = board.generate_moves_for(attackers, |mut moves| {
                moves.to &= victims;
                for mv in moves {
                    let mut child = board.clone();
                    child.play_unchecked(mv);
                    let eval = -qsearch(
                        &child,
                        ply_index + 1,
                        -beta,
                        -alpha
                    );
            
                    if eval > best_eval {
                        best_eval = eval;
                    }
            
                    alpha = alpha.max(best_eval);
                    if alpha >= beta {
                        return true;
                    }
                }
                false
            });
            if stop {
                break 'search;
            }
        }
    }
    best_eval
}
