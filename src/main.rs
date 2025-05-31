use ui::tasks_page;

mod ui;

fn main() -> iced::Result {
    iced::application("Planter", tasks_page::update, tasks_page::view).run()
}
