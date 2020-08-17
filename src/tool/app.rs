use {
    std::{
        collections::HashSet,
        time::Duration,
        path::{Path, PathBuf},
    },
    minifb::{Window, WindowOptions, Menu, Key, MENU_KEY_CTRL, MouseMode, MouseButton},
    rtree_test::{TestCase, Rect, Coord, painter::*},
};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const FPS: u64 = 60;

const SAVE_ID: usize = 0;
const UNDO_ID: usize = 1;

const DATA_COLOR: ColorRGB = 0;
const FOUND_COLOR: ColorRGB = 0x0000ff00;
const SEARCH_COLOR: ColorRGB = 0x00ff0000;
const BEGIN_EDIT_COLOR: ColorRGB = 0x000000ff;


pub struct App {
    case_file: PathBuf,
    window: Window,
    painter: Painter,
    data_rects: Vec<Rect>,
    search_rect: Option<Rect>,
    founded: HashSet<usize>,
    begin_coords: Option<(Coord, Coord)>,
    is_edit: bool,
    edit_mode: EditMode
}

impl App {
    pub fn new<P: AsRef<Path>>(original_case_file: P, case_file: P) -> App {
        let mut window = Window::new(
            env!("CARGO_PKG_NAME"),
            WIDTH,
            HEIGHT,
            WindowOptions {
                borderless: true,
                resize: false,
                ..WindowOptions::default()
            }
        ).unwrap();

        let mut file_menu = Menu::new("File").unwrap();
        file_menu.add_item("Save", SAVE_ID)
            .shortcut(Key::S, MENU_KEY_CTRL)
            .build();

        let mut edit_menu = Menu::new("Edit").unwrap();
        edit_menu.add_item("Undo", UNDO_ID)
            .shortcut(Key::Z, MENU_KEY_CTRL)
            .build();

        window.add_menu(&file_menu);
        window.add_menu(&edit_menu);

        window.limit_update_rate(Some(Duration::from_nanos(1000000000 / FPS)));

        let data_rects;
        let search_rect;
        let founded;

        if original_case_file.as_ref().exists() {
            let test_case = TestCase::load(&original_case_file);

            data_rects = test_case.data_rects;
            search_rect = Some(test_case.search_rect);
            founded = test_case.founded;
        } else {
            data_rects = vec![];
            search_rect = None;
            founded = HashSet::new();
        }

        let mut app = Self {
            case_file: case_file.as_ref().to_path_buf(),
            window,
            painter: Painter::new(WIDTH, HEIGHT),
            data_rects,
            search_rect,
            founded,
            begin_coords: None,
            is_edit: false,
            edit_mode: EditMode::Data,
        };

        app.redraw();

        app
    }

    pub fn render_loop(&mut self) {
        while self.window.is_open() {
            self.handle_input();
            self.painter.render(&mut self.window);
        }
    }

    fn handle_input(&mut self) {
        self.window.get_mouse_pos(MouseMode::Clamp).map(|(x, y)| {
            if self.window.get_mouse_down(MouseButton::Left)
            || self.window.get_mouse_down(MouseButton::Right) {
                if self.is_edit {
                    return;
                } {
                    self.is_edit = true;
                }

                if self.window.get_mouse_down(MouseButton::Right) {
                    self.edit_mode = EditMode::Search;
                }

                let x = x as Coord;
                let y = y as Coord;

                match self.begin_coords {
                    Some((bx, by)) => self.finish_edit((bx, x), (by, y)),
                    None => self.begin_edit(x, y)
                }
            } else {
                self.is_edit = false;
            }
        });

        self.window.is_menu_pressed().map(|id| {
            match id {
                SAVE_ID => self.save(),
                UNDO_ID => self.undo(),
                _ => {}
            }
        });
    }

    fn save(&self) {
        if self.search_rect.is_none() {
            eprintln!("Search area is mandatory");
            return;
        }

        TestCase {
            data_rects: self.data_rects.clone(),
            search_rect: self.search_rect.clone().unwrap(),
            founded: self.founded.clone()
        }.save(&self.case_file);
    }

    fn begin_edit(&mut self, x: Coord, y: Coord) {
        self.draw_edit_signal_rect(BEGIN_EDIT_COLOR, x, y);
        self.begin_coords = Some((x, y))
    }

    fn finish_edit(&mut self, xs: (Coord, Coord), ys: (Coord, Coord)) {
        self.clear_begin_edit(xs.0, ys.0);

        match self.edit_mode {
            EditMode::Data => self.add_new_rect(xs, ys),
            EditMode::Search => self.set_search_rect(xs, ys)
        }
        self.edit_mode = EditMode::Data;
    }

    fn clear_begin_edit(&mut self, x: Coord, y: Coord) {
        let finish_color = Painter::clear_color();

        self.draw_edit_signal_rect(finish_color, x, y);
        self.begin_coords = None;
    }

    fn undo(&mut self) {
        match self.begin_coords {
            Some((x, y)) => self.clear_begin_edit(x, y),
            None => if !self.data_rects.is_empty() {
                let index = self.data_rects.len() - 1;
                self.draw_rect_by_index(Painter::clear_color(), index);
                self.data_rects.swap_remove(index);
                self.founded.remove(&index);
            }
        }

        self.redraw();

        self.is_edit = false;
        self.edit_mode = EditMode::Data;
    }

    fn set_search_rect(&mut self, xs: (Coord, Coord), ys: (Coord, Coord)) {
        if let Some(ref src_rect) = self.search_rect {
            self.painter.draw_hollow_rect(Painter::clear_color(), src_rect);
        }

        let rect = Self::make_rect(xs, ys);
        self.painter.draw_hollow_rect(SEARCH_COLOR, &rect);

        self.founded.clear();
        for i in 0..self.data_rects.len() {
            if self.data_rects[i].intersects_with(&rect) {
                self.founded.insert(i);
                self.draw_rect_by_index(FOUND_COLOR, i);
            } else {
                self.draw_rect_by_index(DATA_COLOR, i);
            }
        }

        self.search_rect = Some(rect);
    }

    fn redraw(&mut self) {
        for i in 0..self.data_rects.len() {
            let color;

            if self.founded.contains(&i) {
                color = FOUND_COLOR;
            } else {
                color = DATA_COLOR;
            }

            self.draw_rect_by_index(color, i);
        }

        if let Some(ref search_rect) = self.search_rect {
            self.painter.draw_hollow_rect(SEARCH_COLOR, search_rect);
        }
    }

    fn draw_edit_signal_rect(&mut self, color: ColorRGB, x: Coord, y: Coord) {
        let begin_size = 5;
        self.painter.draw_filled_rect(color, &Rect {
            top: y,
            bottom: y + begin_size,
            left: x,
            right: x + begin_size
        });
    }

    fn add_new_rect(&mut self, xs: (Coord, Coord), ys: (Coord, Coord)) {
        let rect = Self::make_rect(xs, ys);
        let index = self.data_rects.len();

        let color = match self.search_rect {
            Some(ref search_rect) if rect.intersects_with(search_rect) => {
                self.founded.insert(index);

                FOUND_COLOR
            }
            _ => DATA_COLOR
        };

        self.data_rects.push(rect);
        self.draw_rect_by_index(color, index);
    }

    fn make_rect(xs: (Coord, Coord), ys: (Coord, Coord)) -> Rect {
        let (mut left, mut right) = xs;
        let (mut top, mut bottom) = ys;

        if left > right {
            std::mem::swap(&mut left, &mut right);
        }

        if top > bottom {
            std::mem::swap(&mut top, &mut bottom);
        }

        Rect {
            top,
            bottom,
            left,
            right
        }
    }

    fn draw_rect_by_index(&mut self, color: ColorRGB, index: usize) {
        let rect = &self.data_rects[index];
        let index_x = rect.left + (rect.right - rect.left) / 2;
        let index_y = rect.top + (rect.bottom - rect.top) / 2;

        self.painter.draw_hollow_rect(color, rect);
        self.painter.draw_num(color, index_x, index_y, index);
    }
}

enum EditMode {
    Data,
    Search
}
