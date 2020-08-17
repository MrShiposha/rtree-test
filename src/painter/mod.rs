use {
    std::path::Path,
    minifb::Window,
    image,
    super::{Rect, Coord, TestCase}
};

mod digit;
use digit::Digit;

pub type ColorRGB = u32;
pub struct ColorHSV(pub u8, pub u8, pub u8);
const COLOR_CHANNEL_SIZE: usize = 8;

const DEFAULT_COLOR: ColorRGB = 0x00FFFFFF;
pub const DATA_COLOR: ColorRGB = 0;
pub const FOUND_COLOR: ColorRGB = 0x0000ff00;
pub const SEARCH_COLOR: ColorRGB = 0x00ff0000;

const DIGIT_RECT_SIZE: Coord = 2;
const DIGIT_ROWS: Coord = 8;
const DIGIT_COLS: Coord = 8;
const DIGIT_WIDTH: Coord = DIGIT_COLS*DIGIT_RECT_SIZE;

pub trait IntoRGB {
    fn into_rgb(self) -> ColorRGB;
}

impl IntoRGB for (u8, u8, u8) {
    fn into_rgb(self) -> ColorRGB {
        let r = (self.0 as ColorRGB) << (COLOR_CHANNEL_SIZE << 1);
        let g = (self.1 as ColorRGB) << COLOR_CHANNEL_SIZE;
        let b = self.2 as ColorRGB;

        r | g | b
    }
}

impl IntoRGB for ColorHSV {
    fn into_rgb(self) -> ColorRGB {
        fn percent_to_value(percent: u8) -> u8 {
            (255.0 * percent as f32 / 100.0) as u8
        }

        let ColorHSV(h, s, v) = self;
        let h_i = (h / 60) % 6;
        let v_min = (((100 - s as u16) * v as u16) / 100) as u8;
        let a = ((v - v_min) as u16 * (h % 60) as u16 / 60) as u8;

        let v_inc = v_min + a;
        let v_dec = v - a;

        let v = percent_to_value(v);
        let v_min = percent_to_value(v_min);
        let v_dec = percent_to_value(v_dec);
        let v_inc = percent_to_value(v_inc);

        match h_i {
            0 => (v, v_inc, v_min).into_rgb(),
            1 => (v_dec, v, v_min).into_rgb(),
            2 => (v_min, v, v_inc).into_rgb(),
            3 => (v_min, v_dec, v).into_rgb(),
            4 => (v_inc, v_min, v).into_rgb(),
            5 => (v, v_min, v_dec).into_rgb(),
            _ => unreachable!()
        }
    }
}

pub trait UnpackRGB {
    fn unpack_rgb(&self) -> (u8, u8, u8);
}

impl UnpackRGB for ColorRGB {
    fn unpack_rgb(&self) -> (u8, u8, u8) {
        let r = ((self >> (COLOR_CHANNEL_SIZE << 1)) & 0xFF) as u8;
        let g = ((self >> COLOR_CHANNEL_SIZE) & 0xFF) as u8;
        let b = (self & 0xFF) as u8;

        (r, g, b)
    }
}

pub struct Painter {
    frame_buffer: Vec<ColorRGB>,
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

    pub fn save_image<P: AsRef<Path>>(&self, path: P) {
        image::save_buffer(
            path,
            self.frame_buffer.iter().map(|rgb_color| {
                let (r, g, b) = rgb_color.unpack_rgb();

                vec![r, g, b]
            }).flatten().collect::<Vec<_>>().as_slice(),
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgb8
        ).unwrap();
    }

    pub fn draw_test_case(&mut self, case: &TestCase) {
        for (i, rect) in case.data_rects.iter().enumerate() {
            let color;

            if case.founded.contains(&i) {
                color = FOUND_COLOR;
            } else {
                color = DATA_COLOR;
            }

            self.draw_indexed_rect(rect, color, i);
        }

        self.draw_hollow_rect(SEARCH_COLOR, &case.search_rect);
    }

    pub fn draw_indexed_rect(&mut self, rect: &Rect, color: ColorRGB, index: usize) {
        let index_x = rect.left + (rect.right - rect.left) / 2;
        let index_y = rect.top + (rect.bottom - rect.top) / 2;

        self.draw_hollow_rect(color, rect);
        self.draw_num(color, index_x, index_y, index);
    }

    pub fn draw_hline(&mut self, color: ColorRGB, x0: Coord, x1: Coord, y: Coord) {
        self.draw_pixels(color, (x0..=x1).map(|x| (x, y)))
    }

    pub fn draw_vline(&mut self, color: ColorRGB, y0: Coord, y1: Coord, x: Coord) {
        self.draw_pixels(color, (y0..=y1).map(|y| (x, y)))
    }

    pub fn draw_pixels<I>(&mut self, color: ColorRGB, coords: I)
    where
        I: Iterator<Item=(Coord, Coord)>
    {
        let width = self.width;
        for i in coords.map(|(x, y)| Self::coords_to_index(width, x, y)) {
            self.frame_buffer[i] = color;
        }
    }

    pub fn draw_hollow_rect(&mut self, color: ColorRGB, rect: &Rect) {
        self.draw_hline(color, rect.left, rect.right, rect.top);
        self.draw_hline(color, rect.left, rect.right, rect.bottom);
        self.draw_vline(color, rect.top, rect.bottom, rect.left);
        self.draw_vline(color, rect.top, rect.bottom, rect.right);
    }

    pub fn draw_num(&mut self, color: ColorRGB, x: Coord, y: Coord, num: usize) {
        let mut digits = num_to_digits(num);

        let mut x = x - (DIGIT_WIDTH*digits.len() as Coord) / 2;
        let y = y - DIGIT_WIDTH / 2;

        while let Some(digit) = digits.pop() {
            self.draw_digit(color, x, y, digit);
            x += DIGIT_WIDTH;
        }
    }

    fn draw_digit(&mut self, color: ColorRGB, x: Coord, y: Coord, digit: u8) {
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

    pub fn draw_filled_rect(&mut self, color: ColorRGB, rect: &Rect) {
        for y in rect.top..rect.bottom {
            self.draw_hline(color, rect.left, rect.right, y);
        }
    }

    pub fn clear_color() -> ColorRGB {
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