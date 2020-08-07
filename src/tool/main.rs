mod painter;
mod app;

use {
    std::env,
    app::App
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <test-case-file>", env!("CARGO_PKG_NAME"));
        std::process::exit(-1);
    }

    let test_case_file = &args[1];
    let mut app = App::new(test_case_file);

    app.render_loop();
}

