include!(concat!(env!("OUT_DIR"), "/packets.rs"));

#[derive(Debug, Clone, PartialEq)]
pub enum BoardCell {
    None,
    X,
    O,
}

impl BoardCell {
    pub fn from_usize(id: usize) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::X),
            2 => Ok(Self::O),
            _ => Err(format!("Bad BoardCell id {id}").into()),
        }
    }
    pub fn to_usize(&self) -> usize {
        match self {
            Self::None => 0,
            Self::X => 1,
            Self::O => 2,
        }
    }
}

impl Default for BoardCell {
    fn default() -> Self {
        Self::None
    }
}

pub struct Board {
    pub width: usize,
    pub height: usize,
    pub teams: usize,
    pub cells: Vec<Vec<BoardCell>>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let mut res = Self {
            width,
            height,
            teams: 2,
            cells: Vec::with_capacity(height),
        };
        res.cells.fill_with(|| Vec::with_capacity(width));
        res
    }
    pub fn put(
        &mut self,
        x: usize,
        y: usize,
        cell: BoardCell,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if x >= self.width || y >= self.height {
            return Err("Attemped to set cell outside the bounds of board".into());
        }
        let row = match self.cells.get_mut(y) {
            Some(row) => row,
            None => {
                while self.cells.len() < y {
                    self.cells.push(Vec::with_capacity(self.width));
                }
                let row = Vec::with_capacity(self.width);
                self.cells.push(row);
                self.cells.get_mut(y).unwrap()
            }
        };
        match row.get(x) {
            Some(_) => (),
            None => {
                while row.len() < x {
                    row.push(BoardCell::None);
                }
                let column = BoardCell::None;
                row.push(column);
            }
        };

        self.cells[y][x] = cell;

        Ok(())
    }
    pub fn get(&self, x: usize, y: usize) -> BoardCell {
        let row = self.cells.get(y);
        match row {
            Some(row) => match row.get(x) {
                Some(cell) => cell.clone(),
                None => BoardCell::None,
            },
            None => BoardCell::None,
        }
    }
    pub fn occupied(&self, x: usize, y: usize) -> bool {
        self.get(x, y) != BoardCell::None
    }
}
