use {
    minifb::Window,
    rtree_test::{Rect, Coord}
};

pub type Color = u32;
const COLOR_CHANNEL_SIZE: usize = 8;

const DEFAULT_COLOR: Color = 0x00FFFFFF;

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
    height: usize
}

impl Painter {
    pub fn new(width: usize, height: usize) -> Self {
        Painter {
            frame_buffer: vec![DEFAULT_COLOR; width * height],
            width,
            height
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

    pub fn draw_num(&mut self, color: Color, x: Coord, y: Coord, mut num: usize) {
        let rect_size = 5;
        let rect_gap = 3;
        let rects_in_row = 5;

        let rows = (num as f32 / rects_in_row as f32).ceil() as Coord;
        let cols = if rows == 1 {
            (num % (rects_in_row + 1)) as Coord
        } else {
            rects_in_row as Coord
        };

        let top = y - (rect_size*rows + rect_gap*(rows - 1)) / 2;
        let left = x - (rect_size*cols + rect_gap*(cols - 1)) / 2;

        for row in 0..rows {
            for col in 0..cols {
                let local_top = top + row*(rect_size + rect_gap);
                let local_bottom = local_top + rect_size;
                let local_left = left + col*(rect_size + rect_gap);
                let local_right = local_left + rect_size;

                self.draw_filled_rect(color, &Rect {
                    top: local_top,
                    bottom: local_bottom,
                    left: local_left,
                    right: local_right
                });

                num -= 1;
                if num == 0 {
                    return;
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