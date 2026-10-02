use tauri::AppHandle;

use crate::panel;

enum Command {
    Start,
    Toggle,
    Show,
    Hide,
    Quit,
}

fn parse(argv: &[String]) -> Option<Command> {
    match argv.get(1).map(String::as_str) {
        None => Some(Command::Start),
        Some("toggle") => Some(Command::Toggle),
        Some("show") => Some(Command::Show),
        Some("hide") => Some(Command::Hide),
        Some("quit") => Some(Command::Quit),
        Some(_) => None,
    }
}

pub fn run(app: &AppHandle, argv: &[String]) {
    match parse(argv) {
        Some(Command::Toggle) => panel::toggle(app),
        Some(Command::Show) => panel::show(app),
        Some(Command::Hide) => panel::hide(app, false),
        Some(Command::Quit) => app.exit(0),
        Some(Command::Start) | None => {}
    }
}

pub fn cold_start(app: &AppHandle, argv: &[String]) {
    match parse(argv) {
        Some(Command::Toggle) => panel::toggle(app),
        Some(Command::Show) => panel::show(app),
        Some(Command::Quit) => app.exit(0),
        Some(Command::Start) | Some(Command::Hide) | None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(rest: &[&str]) -> Vec<String> {
        std::iter::once("slopchat".to_string())
            .chain(rest.iter().map(|value| value.to_string()))
            .collect()
    }

    #[test]
    fn maps_arguments_to_commands() {
        assert!(matches!(parse(&args(&[])), Some(Command::Start)));
        assert!(parse(&args(&["start"])).is_none());
        assert!(matches!(parse(&args(&["toggle"])), Some(Command::Toggle)));
        assert!(matches!(parse(&args(&["show"])), Some(Command::Show)));
        assert!(matches!(parse(&args(&["hide"])), Some(Command::Hide)));
        assert!(matches!(parse(&args(&["quit"])), Some(Command::Quit)));
        assert!(parse(&args(&["bogus"])).is_none());
    }
}
