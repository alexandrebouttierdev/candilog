//! Calendrier natif partagé par les formulaires et les filtres.

use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::overlay::{self, Size};
use crate::ui::components::{layout, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use chrono::{Datelike, NaiveDate};
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Element, Length};

pub fn layer(app: &App) -> Element<'_, Message> {
    let Some(picker) = app.date_picker else {
        return Space::with_height(0).into();
    };
    let Some(first) = NaiveDate::from_ymd_opt(picker.year, picker.month, 1) else {
        return Space::with_height(0).into();
    };
    let next = if picker.month == 12 {
        NaiveDate::from_ymd_opt(picker.year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(picker.year, picker.month + 1, 1)
    }
    .unwrap_or(first);
    let days_in_month = (next - chrono::Duration::days(1)).day();
    let offset = first.weekday().num_days_from_monday();

    let weekdays = row![
        day_label("Lun"),
        day_label("Mar"),
        day_label("Mer"),
        day_label("Jeu"),
        day_label("Ven"),
        day_label("Sam"),
        day_label("Dim"),
    ]
    .spacing(space::XS);

    let mut calendar = column![weekdays].spacing(space::XS);
    for week in 0..6_u32 {
        let mut line = row![].spacing(space::XS);
        for weekday in 0..7_u32 {
            let slot = week * 7 + weekday;
            if slot < offset || slot >= offset + days_in_month {
                line = line.push(Space::new(38.0, 34.0));
                continue;
            }
            let day = slot - offset + 1;
            let date = NaiveDate::from_ymd_opt(picker.year, picker.month, day).unwrap_or(first);
            line = line.push(day_cell(
                day,
                date == chrono::Local::now().date_naive(),
                date,
            ));
        }
        calendar = calendar.push(line);
    }

    let heading = format!(
        "{} {}",
        crate::ui::format::month_name(picker.month),
        picker.year
    );
    let body = column![
        row![
            controls::icon_action(
                Icon::ArrowLeft,
                "Mois précédent",
                Message::DatePickerPreviousMonth,
            ),
            layout::spacer(),
            typo::label(heading),
            layout::spacer(),
            controls::icon_action(
                Icon::ArrowRight,
                "Mois suivant",
                Message::DatePickerNextMonth,
            ),
        ]
        .align_y(Alignment::Center),
        container(calendar)
            .padding(space::MD)
            .width(Length::Shrink)
            .style(crate::ui::theme::styles::sunken),
    ]
    .spacing(space::LG)
    .align_x(Alignment::Center);

    overlay::modal(
        "Choisir une date",
        body,
        overlay::footer([
            controls::ghost("Fermer", None)
                .on_press(Message::CloseDatePicker)
                .into(),
            controls::secondary("Aujourd'hui", Some(Icon::Calendar))
                .on_press(Message::DatePickerSelected(
                    chrono::Local::now().date_naive(),
                ))
                .into(),
        ]),
        Size::Confirm,
        Message::CloseDatePicker,
    )
}

fn day_label<'a>(label: &'a str) -> Element<'a, Message> {
    container(
        typo::caption(label)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    )
    .width(38.0)
    .height(24.0)
    .center_x(38.0)
    .center_y(24.0)
    .into()
}

/// Cellule à dimensions fixes : le chiffre et l'en-tête partagent la même colonne.
fn day_cell(day: u32, active: bool, date: NaiveDate) -> Element<'static, Message> {
    button(
        container(typo::body(day.to_string()))
            .width(Length::Fill)
            .height(34.0)
            .center_x(Length::Fill)
            .center_y(34.0),
    )
    .width(38.0)
    .height(34.0)
    .padding(0)
    .style(if active {
        styles::selected_inverse
    } else {
        styles::ghost
    })
    .on_press(Message::DatePickerSelected(date))
    .into()
}
