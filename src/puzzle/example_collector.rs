use crate::go::{GoGame, GoPlayer};
use std::{fs::File, io::Write, sync::mpsc::Sender};

pub trait ExampleCollector {
    fn collect_example(&mut self, node: GoGame, player_won: GoPlayer);
}

pub struct NullExampleCollector;

impl ExampleCollector for NullExampleCollector {
    fn collect_example(&mut self, _node: GoGame, _player_won: GoPlayer) {}
}

pub struct FileExampleCollector {
    file: File,
    sample_index: u32,
    sample_every: u32,
}

impl FileExampleCollector {
    pub fn new(file: File, sample_every: u32) -> Self {
        Self {
            file,
            sample_index: 0,
            sample_every,
        }
    }

    fn write_example(&mut self, node: GoGame, player_won: GoPlayer) {
        // Always make it black to play
        let board = if node.current_player == GoPlayer::Black {
            node.board
        } else {
            node.board.invert_colours()
        };

        let mut bytes: [u8; 49] = [0; 49];

        bytes[0..16].copy_from_slice(&board.get_bitboard_for_player(GoPlayer::Black).serialise());
        bytes[16..32].copy_from_slice(&board.get_bitboard_for_player(GoPlayer::White).serialise());
        bytes[32..48].copy_from_slice(&(!board.out_of_bounds()).serialise());
        bytes[48] = if player_won == node.current_player {
            1
        } else {
            0
        };

        self.file.write_all(&bytes).unwrap();
    }
}

impl ExampleCollector for FileExampleCollector {
    fn collect_example(&mut self, node: GoGame, player_won: GoPlayer) {
        if self.sample_index == 0 {
            self.write_example(node, player_won);
        }

        self.sample_index = (self.sample_index + 1) % self.sample_every;
    }
}

#[derive(Clone)]
pub struct ChannelExampleCollector {
    tx: Sender<(GoGame, GoPlayer)>,
}

impl ChannelExampleCollector {
    pub fn new(tx: Sender<(GoGame, GoPlayer)>) -> Self {
        Self { tx }
    }
}

impl ExampleCollector for ChannelExampleCollector {
    fn collect_example(&mut self, node: GoGame, player_won: GoPlayer) {
        self.tx.send((node, player_won)).unwrap();
    }
}
