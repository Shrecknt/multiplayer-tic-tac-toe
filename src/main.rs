use std::io;
use std::io::Write;

fn main() {
    let mut board = Board::new();
    let main_player = Player::new("X");
    let other_player = Player::new("O");
    loop {
        board.clone().draw();
        let mut yes = &mut board;
        main_player_turn(&main_player.id, yes);
        yes = &mut board;
    }
}
fn read_line() -> Option<String> {
    let mut input = String::new();
    let unwrap = io::stdin().read_line(&mut input);
    if unwrap.is_ok() {
        return Some(input);
    }
    None
}

fn python_input(input: &str) -> Option<String>{
    print!("{input}");
    io::stdout().flush().unwrap();
    read_line()
}


fn main_player_turn(id: &str, mut board: &mut Board) {
    let input = python_input("What's your choice: ").unwrap();
    match input.trim().parse::<i32>() {
        Ok(mut spot) => {
            spot = spot - 1;
            if !is_valid_turn(&board, &spot) {
                return;
            };
            board.spots[spot as usize] = id.to_string();
        }
        Err(_) => {
            println!("You put bad number! :angry:")
        }
    }
}
fn is_valid_turn(board: &Board, &spot: &i32) -> bool {
    if !board.clone().valid_spot(spot) {
        println!("Invalid spot!");
        return false;
    }
    if board.clone().spot_taken(spot) {
        println!("Spot taken!");
        return false;
    }
    true
}
#[derive(Clone)]
struct Board {
    spots: Vec<String>
}

impl Board {
    fn new() -> Board {
        return Board {spots: vec![" ".to_string(); 10] }
    }
    fn spot_taken(self, spot: i32) -> bool {
        self.spots.get(spot as usize) != Some(&" ".to_string())
    }
    fn valid_spot(self, spot: i32) -> bool {
        spot >= 0 && spot < (self.spots.len() - 1) as i32
    }
    fn draw(self) {
        println!("-------");
        println!("|{}|{}|{}|", self.spots[0], self.spots[1], self.spots[2]);
        println!("-------");
        println!("|{}|{}|{}|", self.spots[3], self.spots[4], self.spots[5]);
        println!("-------");
        println!("|{}|{}|{}|", self.spots[6], self.spots[7], self.spots[8]);
        println!("-------");
    }
}
struct Player {
    id: String
}

impl Player {
    fn new(id: &str) -> Player {
        Player {id: id.to_string()}
    }
}