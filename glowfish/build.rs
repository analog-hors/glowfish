use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use cozy_chess::*;

fn square_to_string(square: Square) -> String {
    let mut square = format!("{}", square);
    square.make_ascii_uppercase();
    format!("Square::{square}")
}

fn move_to_string(mv: Move) -> String {
    let from = square_to_string(mv.from);
    let to = square_to_string(mv.to);
    let promotion = match mv.promotion {
        Some(Piece::Queen) => "Some(Piece::Queen)",
        Some(Piece::Knight) => "Some(Piece::Knight)",
        Some(Piece::Rook) => "Some(Piece::Rook)",
        Some(Piece::Bishop) => "Some(Piece::Bishop)",
        None => "None",
        _ => unreachable!()
    };
    format!("Move{{from:{from},to:{to},promotion:{promotion}}}")
}

fn main() {
    let mut book: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    book.push("book.rs");
    let mut book = BufWriter::new(File::create(book).unwrap());

    writeln!(&mut book, "fn book_entry(board: &Board) -> &[Move] {{").unwrap();
    writeln!(&mut book, "    match board.hash() {{").unwrap();
    for line in include_str!("book.txt").lines() {
        let (board, moves) = line.trim().split_once('|').unwrap();
        let board = board.parse::<Board>().unwrap();
        let moves = moves.split(',')
            .map(|mv| mv.parse::<Move>().unwrap());
        write!(&mut book, "        {} => &[", board.hash()).unwrap();
        for mv in moves {
            let mv = move_to_string(mv);
            write!(&mut book, "{mv},").unwrap();
        }
        writeln!(&mut book, "],").unwrap();
    }
    writeln!(&mut book, "        _ => &[]").unwrap();
    writeln!(&mut book, "    }}").unwrap();
    writeln!(&mut book, "}}").unwrap();
}
