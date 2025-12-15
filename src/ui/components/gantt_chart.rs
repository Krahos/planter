use chrono::{DateTime, Duration, Utc};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Text, mouse};
use iced::alignment::Horizontal;
use iced::{Border, Color, Element, Length, Point, Rectangle, Size};
use planter_core::project::Project;

use crate::ui::style::{BORDER_RADIUS_MD, PADDING_MD, ROW_HEIGHT, THEME};

pub struct GanttChart<'a> {
    project: &'a Project,
}

impl<'a> GanttChart<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self { project }
    }
}

pub fn gantt_chart(project: &Project) -> GanttChart<'_> {
    GanttChart::new(project)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for GanttChart<'a>
where
    Renderer: advanced::text::Renderer + renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::new(600.0, 400.0));
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let name_width = 220.0;
        let chart_width = bounds.width - name_width - PADDING_MD;

        // Find date range
        let (min_date, max_date) =
            self.project.tasks().fold((None, None), |(min, max), task| {
                match (task.start(), task.finish()) {
                    (Some(s), Some(f)) => (
                        Some(min.map_or(s, |m: DateTime<Utc>| m.min(s))),
                        Some(max.map_or(f, |m: DateTime<Utc>| m.max(f))),
                    ),
                    _ => (min, max),
                }
            });

        // Add padding to date range
        let (min, max) = match (min_date, max_date) {
            (Some(mn), Some(mx)) => {
                let pad = Duration::hours((mx - mn).num_hours().max(24) / 10);
                (mn - pad, mx + pad)
            }
            _ => return,
        };

        let total_seconds = (max - min).num_seconds() as f32;
        if total_seconds <= 0.0 {
            return;
        }

        // Draw background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            THEME.background,
        );

        // Draw vertical grid lines
        let num_grid_lines = 8;
        for i in 0..=num_grid_lines {
            let x = bounds.x
                + name_width
                + PADDING_MD
                + (i as f32 / num_grid_lines as f32) * chart_width;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: bounds.y,
                        width: 1.0,
                        height: bounds.height,
                    },
                    border: Border::default(),
                    ..renderer::Quad::default()
                },
                THEME.border,
            );
        }

        // Draw tasks
        for (i, task) in self.project.tasks().enumerate() {
            let y = bounds.y + (i as f32 * ROW_HEIGHT);

            // Alternating row background
            if i % 2 == 1 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y,
                            width: bounds.width,
                            height: ROW_HEIGHT,
                        },
                        border: Border::default(),
                        ..renderer::Quad::default()
                    },
                    THEME.background_alt,
                );
            }

            // Task name
            let name = Text {
                content: task.name().to_string(),
                bounds: Size::new(name_width, ROW_HEIGHT),
                size: renderer.default_size().into(),
                line_height: advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: Horizontal::Left.into(),
                align_y: iced::alignment::Vertical::Center,
                shaping: advanced::text::Shaping::Advanced,
                wrapping: advanced::text::Wrapping::default(),
            };
            renderer.fill_text(
                name,
                Point::new(bounds.x + PADDING_MD, y),
                THEME.text,
                *viewport,
            );

            // Task bar
            if let (Some(start), Some(finish)) = (task.start(), task.finish()) {
                let x_offset = (start - min).num_seconds() as f32 / total_seconds;
                let width = (finish - start).num_seconds() as f32 / total_seconds;

                let bar_x = bounds.x + name_width + PADDING_MD + (x_offset * chart_width);
                let bar_width = (width * chart_width).max(4.0);
                let bar_height = 24.0;
                let bar_y = y + (ROW_HEIGHT - bar_height) / 2.0;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bar_x,
                            y: bar_y,
                            width: bar_width,
                            height: bar_height,
                        },
                        border: Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: BORDER_RADIUS_MD.into(),
                        },
                        ..renderer::Quad::default()
                    },
                    if task.completed() {
                        THEME.success
                    } else {
                        THEME.primary
                    },
                );
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<GanttChart<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer + renderer::Renderer + 'a,
{
    fn from(gantt: GanttChart<'a>) -> Self {
        Self::new(gantt)
    }
}
