//! Storybook - Component Preview Tool for script-kit-gpui
//!
//! A standalone binary for previewing and testing Script Kit components.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin storybook
//! cargo run --bin storybook -- --story "button"
//! ```

use gpui::*;
use script_kit_gpui::storybook::StoryBrowser;

fn main() {
    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    let mut initial_story: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--story" | "-s" => {
                if i + 1 < args.len() {
                    initial_story = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                eprintln!("Script Kit Storybook - Component Preview Tool");
                eprintln!();
                eprintln!("Usage: storybook [OPTIONS]");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  -s, --story <ID>     Open a specific story by ID");
                eprintln!("  -h, --help           Show this help message");
                eprintln!();
                eprintln!("Available stories:");
                eprintln!("  button           - Button component variants");
                eprintln!("  toast            - Toast notification component");
                eprintln!("  form-fields      - Form input components");
                eprintln!("  list-item        - List item component");
                eprintln!("  scrollbar        - Scrollbar component");
                eprintln!("  design-tokens    - Design system tokens");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    Application::new().run(move |cx| {
        // Create window options
        let window_size = size(px(1200.), px(800.));
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                window_size,
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Script Kit Storybook".into()),
                appears_transparent: false,
                ..Default::default()
            }),
            window_min_size: Some(size(px(800.), px(600.))),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| {
            cx.new(|cx| {
                let mut browser = StoryBrowser::new(cx);

                // Select initial story if specified
                if let Some(ref story_id) = initial_story {
                    browser.select_story(story_id);
                }

                browser
            })
        })
        .expect("Failed to open storybook window");
    });
}
