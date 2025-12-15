use iced::Element;
use iced::widget::scrollable;
use planter_core::project::Project;

use super::components::gantt_chart::gantt_chart;

#[derive(Debug, Default)]
pub struct GanttState {}

#[derive(Debug, Clone)]
pub enum GanttMessage {}

pub fn update(_state: &mut GanttState, _project: &mut Project, _message: GanttMessage) {}

pub fn view(project: &Project) -> Element<'_, GanttMessage> {
    scrollable(gantt_chart(project)).into()
}
