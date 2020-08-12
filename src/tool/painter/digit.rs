use {
    typenum::U64,
    bitmaps::Bitmap
};

pub struct Digit(pub(super) Bitmap<U64>);

impl Digit {
    pub fn new(bitmap: u64) -> Self {
        Digit(Bitmap::from_value(bitmap))
    }
}