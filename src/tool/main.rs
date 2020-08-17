mod app;

use {
    std::env,
    app::App
};

fn main() {
    let original_test_case_file;
    let new_test_case_file;

    let args = env::args().collect::<Vec<_>>();
    match args.as_slice() {
        [_, original] => {
            original_test_case_file = original;
            new_test_case_file = original_test_case_file;
        },
        [_, original, new_file] => {
            original_test_case_file = original;
            new_test_case_file = new_file;
        },
        _ => {
            eprintln!("Usage: {} <test-case-file> [new-test-case-file]", env!("CARGO_PKG_NAME"));
            std::process::exit(-1);
        }
    }

    let mut app = App::new(original_test_case_file, new_test_case_file);

    app.render_loop();
}

