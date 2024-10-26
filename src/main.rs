mod config;

use std::sync::{Arc, Mutex, RwLock};
use chrono::{Datelike, Days, Utc};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::container::bordered_box;
use iced::widget::{
    button, center, checkbox, column, horizontal_rule, pick_list, progress_bar, row, scrollable, slider, text, text_input, toggler, vertical_rule, vertical_space, Column, Container, Row, Text
};
use iced::{Center, Element, Fill, Length, Renderer, Shadow, Theme};

use config::Config;
use tokio::runtime::{Builder, Runtime};

fn main() {
    iced::application("HCal", Calendar::update, Calendar::view)
    .theme(Calendar::theme)
        .run().unwrap()
}

struct Calendar {
    config: Config,
    // the number of days offset from the current day
    curr_view: isize,
    rt: Runtime,
}

impl Default for Calendar {
    fn default() -> Self {
        Self { config: Config::load(), curr_view: 0, rt: Builder::new_current_thread().enable_time().enable_io().build().unwrap() }
    }
}

impl Calendar {

    fn theme(&self) -> Theme {
        Theme::Dracula
    }

    fn update(&mut self, message: Message) {

    }

    fn view(&self) -> Element<Message> {
        println!("view!");
        let mut days = {
            let mut days: Vec<Vec<Element<'_, Message, Theme, Renderer>>> = vec![];
            for _ in 0..7 {
                days.push(vec![]);
            }
            days
        };
        let now = Utc::now();
        for cal in &self.config.calendars {
            for event in cal.get_events(&self.rt).iter() {
                println!("got events: start {} ende {} name {} loc {} rep {:?}", event.start, event.finish, event.name, event.location, event.repeat/*reqwest::get(&event.name).unwrap().text().unwrap()*/);
                if event.start == 0 {
                    continue;
                }
                if chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap().checked_sub_days(Days::new(7)).unwrap() > chrono::Utc::now() {
                    continue;
                }
                if chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap() < chrono::Utc::now() && event.repeat.is_none() {
                    continue;
                }
                println!("curr millis: {}", Utc::now().timestamp_millis());
                if let Some(rep) = event.repeat.as_ref() {
                    match rep.freq.as_str() {
                        "WEEKLY" => {
                            /*let origin = chrono::DateTime::from_timestamp_millis(start as i64).unwrap();
                            let day = origin.weekday();
                            let time = origin.time();
                            let diff = Utc::now().weekday().number_from_monday();
                            let mut new = Utc::now().checked_sub_days(Days::new(diff as u64)).unwrap().checked_add_days(Days::new(day.num_days_from_monday() as u64)).unwrap().with_time(time).unwrap();
                            new*/
                            let start = chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap();
                            let end = chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap();
                            let name = event.name.replace("\\, ", "\n");
                            days[start.weekday().num_days_from_monday() as usize].push(
                                Container::new(Text::new(format!("({}-{})\n{}\n{}", cut_off_end(&start.time().to_string(), 3), cut_off_end(&end.time().to_string(), 3), name, &event.location))).style(|theme| bordered_box(theme).shadow(Shadow::default())).into()
                            );
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    let start = chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap();
                    for i in 0..6 {
                        if start.clone().checked_add_days(Days::new(i)).unwrap().date_naive() == now.date_naive() {
                            /*days[i as usize].push(row![
                                text(&event.name)
                            ]);*/
                            println!("Got {i}: {}", event.name);
                        }
                    }
                }
            }
        }
        /*let days = days.into_iter().map(|events| column(events.into()).into()).collect::<Vec<_>>();
        center(column(days)).into()*/
        // let days = days.into_iter().map(|events| column(events.into()).into()).collect::<Vec<_>>();

        let mut days_row = Row::new();
        let mut day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].iter();

        for events in days {
            let day = day_names.next().unwrap();
            let day_box = Container::new(Text::new(*day))
                .width(Length::Fill)
                .height(Length::Fill);
            let mut day_col = Column::new().push(day_box);
            day_col = day_col.push(column(events));
            days_row = days_row.push(day_col);
        }

        days_row.into()
    }

}

fn cut_off_end(val: &str, cut: usize) -> &str {
    &val[0..(val.len() - cut)]
}

#[derive(Debug)]
enum Message {
    RefreshCalendars,
}
