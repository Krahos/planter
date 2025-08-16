use ui::{personnel_page, tasks_page};

mod ui;

fn main() -> iced::Result {
    iced::application(
        personnel_page::State::default,
        personnel_page::update,
        personnel_page::view,
    )
    .run()
}
