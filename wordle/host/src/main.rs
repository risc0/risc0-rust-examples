use std::io;

use methods::{WORDLE_ID, WORDLE_PATH};
use risc0_zkvm::host::{Prover, Receipt};
use risc0_zkvm::serde::to_vec;
use wordle_core::WORD_LENGTH;

use crate::wordlist::words::pick_word;

mod wordlist;

// The "server" is an agent in the Wordle game that checks the player's guesses.
struct Server<'a> {
    // The server chooses the secret word, and remembers it until the end of the game. It is private
    // because the player shouldn't know the word until the game is over.
    secret_word: &'a str,
}

impl Server<'_> {
    pub fn new() -> Self {
        Self {
            secret_word: pick_word(),
        }
    }

    pub fn get_secret_word_hash(&self) -> Vec<u32> {
        let receipt = self.check_round("_____");
        let journal = receipt.get_journal_vec().unwrap();
        journal[..16].to_owned()
    }

    pub fn check_round(&self, guess_word: &str) -> Receipt {
        let method_code = std::fs::read(WORDLE_PATH).expect("failed to load method code");
        let mut prover = Prover::new(&method_code, WORDLE_ID).expect("failed to construct prover");

        prover
            .add_input(to_vec(self.secret_word).unwrap().as_slice())
            .unwrap();
        prover
            .add_input(to_vec(&guess_word).unwrap().as_slice())
            .unwrap();

        return prover.run().unwrap();
    }
}

// The "player" is an agent in the Wordle game that tries to guess the server's secret word.
struct Player {
    // The player remembers the hash of the secret word that the server commits to at the beginning
    // of the game. By comparing the hash after each guess, the player knows if the server cheated
    // by changing the word.
    pub hash: Vec<u32>,
}

impl Player {
    pub fn check_receipt(&self, receipt: Receipt) -> Vec<u32> {
        receipt
            .verify(WORDLE_ID)
            .expect("receipt verification failed");

        let journal = receipt.get_journal_vec().unwrap();
        let hash = &journal[..16];

        if hash != self.hash {
            panic!("The hash mismatched, so the server cheated!");
        }

        let score = &journal[16..];
        return score.to_owned();
    }
}

fn read_stdin_guess() -> String {
    let mut guess = String::new();
    loop {
        io::stdin().read_line(&mut guess).unwrap();
        guess.pop(); // remove trailing newline

        if guess.chars().count() == WORD_LENGTH {
            break;
        } else {
            println!("Your guess must have 5 letters!");
            guess.clear();
        }
    }
    guess
}

fn print_wordle_feedback(guess_word: &str, score: &Vec<u32>) {
    for i in 0..WORD_LENGTH {
        match score[i] {
            0 => print!("\x1b[41m"), // correct: green
            1 => print!("\x1b[43m"), // present: yellow
            _ => print!("\x1b[40m"), // miss: black
        }
        print!("{:}", guess_word.chars().nth(i).unwrap());
    }
    println!("\x1b[0m");
}

fn game_is_won(score: Vec<u32>) -> bool {
    return score.iter().all(|x| *x == 0u32);
}

fn main() {
    println!("Welcome to fair wordle!");

    let server = Server::new();
    let player = Player {
        hash: server.get_secret_word_hash(),
    };

    let mut game_won = false;

    for _ in 0..6 {
        let guess_word = read_stdin_guess();
        let receipt = server.check_round(guess_word.as_str());
        let score = player.check_receipt(receipt);
        print_wordle_feedback(guess_word.as_str(), &score);
        if game_is_won(score) {
            game_won = true;
            break;
        }
    }

    if game_won {
        println!("You won!");
    } else {
        println!("Game over");
    }
}
