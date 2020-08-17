use {
    std::{
        collections::HashSet,
        fs::OpenOptions,
        path::Path
    },
    serde::{Serialize, Deserialize},
};

mod rect;
pub use rect::Rect;
pub mod painter;

pub type Coord = i64;

#[derive(Serialize, Deserialize)]
pub struct TestCase {
    pub data_rects: Vec<Rect>,
    pub search_rect: Rect,
    pub founded: HashSet<usize>,
}

impl TestCase {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let file = OpenOptions::new().read(true).open(path).unwrap();

        serde_json::from_reader(file).unwrap()
    }

    pub fn save<P: AsRef<Path>>(self, path: P) {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .unwrap();

        serde_json::to_writer_pretty(
            file,
            &self
        ).unwrap();
    }
}