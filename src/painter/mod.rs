use {
    minifb::Window,
    super::{Rect, Coord}
};

mod digit;
use digit::Digit;

pub type Color = u32;
const COLOR_CHANNEL_SIZE: usize = 8;

const DEFAULT_COLOR: Color = 0x00FFFFFF;

const DIGIT_RECT_SIZE: Coord = 2;
const DIGIT_ROWS: Coord = 8;
const DIGIT_COLS: Coord = 8;
const DIGIT_WIDTH: Coord = DIGIT_COLS*DIGIT_RECT_SIZE;

pub trait IntoRGB {
    fn into_rgb(self) -> Color;
}

impl IntoRGB for (u8, u8, u8) {
    fn into_rgb(self) -> Color {
        let r = (self.0 as Color) << (COLOR_CHANNEL_SIZE << 1);
        let g = (self.1 as Color) << COLOR_CHANNEL_SIZE;
        let b = self.2 as Color;

        r | g | b
    }
}

pub struct Painter {
    frame_buffer: Vec<Color>,
    width: usize,
    height: usize,

    digits: Vec<Digit>,
}

impl Painter {
    pub fn new(width: usize, height: usize) -> Self {
        let digits = vec![
            Digit::new(0x3c4242424242423c),
            Digit::new(0x7820202020203830),
            Digit::new(0x7c0c183060606438),
            Digit::new(0x3c2220301830223c),
            Digit::new(0x7020203e22242830),
            Digit::new(0x1e304040301e027e),
            Digit::new(0x3c46463e060c1830),
            Digit::new(0x04040c183060627e),
            Digit::new(0x3c4242423c42423c),
            Digit::new(0x0c1830607c62623c),
        ];

        Painter {
            frame_buffer: vec![DEFAULT_COLOR; width * height],
            width,
            height,
            digits
        }
    }

    pub fn render(&self, window: &mut Window) {
        window.update_with_buffer(&self.frame_buffer, self.width, self.height).unwrap();
    }

    pub fn draw_hline(&mut self, color: Color, x0: Coord, x1: Coord, y: Coord) {
        self.draw_pixels(color, (x0..=x1).map(|x| (x, y)))
    }

    pub fn draw_vline(&mut self, color: Color, y0: Coord, y1: Coord, x: Coord) {
        self.draw_pixels(color, (y0..=y1).map(|y| (x, y)))
    }

    pub fn draw_pixels<I>(&mut self, color: Color, coords: I)
    where
        I: Iterator<Item=(Coord, Coord)>
    {
        let width = self.width;
        for i in coords.map(|(x, y)| Self::coords_to_index(width, x, y)) {
            self.frame_buffer[i] = color;
        }
    }

    pub fn draw_hollow_rect(&mut self, color: Color, rect: &Rect) {
        self.draw_hline(color, rect.left, rect.right, rect.top);
        self.draw_hline(color, rect.left, rect.right, rect.bottom);
        self.draw_vline(color, rect.top, rect.bottom, rect.left);
        self.draw_vline(color, rect.top, rect.bottom, rect.right);
    }

    pub fn draw_num(&mut self, color: Color, x: Coord, y: Coord, num: usize) {
        let mut digits = num_to_digits(num);

        let mut x = x - (DIGIT_WIDTH*digits.len() as Coord) / 2;
        let y = y - DIGIT_WIDTH / 2;

        while let Some(digit) = digits.pop() {
            self.draw_digit(color, x, y, digit);
            x += DIGIT_WIDTH;
        }
    }

    fn draw_digit(&mut self, color: Color, x: Coord, y: Coord, digit: u8) {
        assert!(digit < 10);

        for row in 0..DIGIT_ROWS {
            for col in 0..DIGIT_COLS {
                let index = (row*DIGIT_COLS + col) as usize;
                if self.digits[digit as usize].0.get(index) {
                    let top = y + DIGIT_RECT_SIZE*row;
                    let bottom = top + DIGIT_RECT_SIZE;
                    let left = x + DIGIT_RECT_SIZE*col;
                    let right = left + DIGIT_RECT_SIZE;

                    self.draw_filled_rect(color, &Rect {
                        top,
                        bottom,
                        left,
                        right
                    });
                }
            }
        }
    }

    pub fn draw_filled_rect(&mut self, color: Color, rect: &Rect) {
        for y in rect.top..rect.bottom {
            self.draw_hline(color, rect.left, rect.right, y);
        }
    }

    pub fn clear_color() -> Color {
        0x00ffffff
    }

    fn coords_to_index(width: usize, x: Coord, y: Coord) -> usize {
        y as usize * width + x as usize
    }
}

fn num_to_digits(mut num: usize) -> Vec<u8> {
    if num < 10 {
        return vec![num as u8];
    }

    let mut digits = vec![];
    let radix = 10;

    while num != 0 {
        digits.push((num % radix) as u8);
        num /= radix;
    }

    digits
}