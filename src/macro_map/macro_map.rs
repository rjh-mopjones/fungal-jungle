
pub enum Tile {
    Sea,
    Ice,
    Jungle,
    Mountain,
    Forest,
    Plains,
    Desert,
    Blank,
}

pub struct MacroMapTile {
    pub(crate) tile: Tile,
    pub(crate) temperature: f64,
    pub(crate) height: f64
}

pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) border_value: f64,
    pub(crate) map: Vec<MacroMapTile>,
}
