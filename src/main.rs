mod config;

use chrono::{Datelike, Days, NaiveTime, Timelike, Utc};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::container::bordered_box;
use iced::widget::text::Wrapping;
use iced::widget::{
    button, center, checkbox, column, horizontal_rule, pick_list, progress_bar, row, scrollable,
    slider, text, text_input, toggler, vertical_rule, vertical_space, Column, Container, Row,
    Space, Text,
};
use iced::{Center, Element, Fill, Length, Renderer, Shadow, Theme};
use std::sync::{Arc, Mutex, RwLock};

use config::Config;
use tokio::runtime::{Builder, Runtime};

fn main() {
    iced::application("HCal", Calendar::update, Calendar::view)
        .theme(Calendar::theme)
        .run()
        .unwrap()
}

struct Calendar {
    config: Config,
    // the number of days offset from the current day
    curr_view: isize,
    rt: Runtime,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            config: Config::load(),
            curr_view: 0,
            rt: Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()
                .unwrap(),
        }
    }
}

impl Calendar {
    fn theme(&self) -> Theme {
        Theme::Dracula
    }

    fn update(&mut self, message: Message) {}

    fn view(&self) -> Element<Message> {
        println!("view!");
        let mut days = {
            let mut days = vec![];
            for _ in 0..7 {
                days.push(vec![]);
            }
            days
        };
        let now = Utc::now();
        let mut earliest = u64::MAX;
        let mut latest = 0;
        for cal in &self.config.calendars {
            for event in cal.get_events(&self.rt).iter() {
                println!(
                    "got events: start {} ende {} name {} loc {} rep {:?}",
                    event.start,
                    event.finish,
                    event.name,
                    event.location,
                    event.repeat /*reqwest::get(&event.name).unwrap().text().unwrap()*/
                );
                if event.start == 0 {
                    continue;
                }
                if chrono::DateTime::from_timestamp_millis(event.start as i64)
                    .unwrap()
                    .checked_sub_days(Days::new(7))
                    .unwrap()
                    > chrono::Utc::now()
                {
                    continue;
                }
                if chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap()
                    < chrono::Utc::now()
                    && event.repeat.is_none()
                {
                    continue;
                }
                println!("curr millis: {}", Utc::now().timestamp_millis());
                if let Some(rep) = event.repeat.as_ref() {
                    match rep.freq.as_str() {
                        "WEEKLY" => {
                            // FIXME: support events spanning multiple days!
                            let (new_start, days_off) = {
                                let origin =
                                    chrono::DateTime::from_timestamp_millis(event.start as i64)
                                        .unwrap();
                                let day = origin.weekday();
                                let time = origin.time();
                                if time.num_seconds_from_midnight() as u64 * 1000 < earliest {
                                    earliest = time.num_seconds_from_midnight() as u64 * 1000;
                                }
                                let diff = Utc::now().weekday().number_from_monday();
                                (
                                    Utc::now()
                                        .checked_sub_days(Days::new(diff as u64))
                                        .unwrap()
                                        .checked_add_days(Days::new(
                                            day.num_days_from_monday() as u64
                                        ))
                                        .unwrap()
                                        .with_time(time)
                                        .unwrap(),
                                    day.num_days_from_monday(),
                                )
                            };

                            let new_end = {
                                let origin =
                                    chrono::DateTime::from_timestamp_millis(event.finish as i64)
                                        .unwrap();
                                let day = origin.weekday();
                                let time = origin.time();
                                if time.num_seconds_from_midnight() as u64 * 1000 > latest {
                                    latest = time.num_seconds_from_midnight() as u64 * 1000;
                                }
                                let diff = Utc::now().weekday().number_from_monday();
                                Utc::now()
                                    .checked_sub_days(Days::new(diff as u64))
                                    .unwrap()
                                    .checked_add_days(Days::new(day.num_days_from_monday() as u64))
                                    .unwrap()
                                    .with_time(time)
                                    .unwrap()
                            };
                            let mut new_ev = event.clone();
                            new_ev.start = new_start.timestamp_millis() as u64;
                            new_ev.finish = new_end.timestamp_millis() as u64;
                            days[days_off as usize].push(new_ev);
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    let start =
                        chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap();
                    let end = chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap();
                    if start.time().num_seconds_from_midnight() as u64 * 1000 < earliest {
                        earliest = start.time().num_seconds_from_midnight() as u64 * 1000;
                    }
                    if end.time().num_seconds_from_midnight() as u64 * 1000 > latest {
                        latest = end.time().num_seconds_from_midnight() as u64 * 1000;
                    }
                    for i in 0..6 {
                        if start
                            .clone()
                            .checked_add_days(Days::new(i))
                            .unwrap()
                            .date_naive()
                            == now.date_naive()
                        {
                            days[i as usize].push(event.clone());
                            println!("Got {i}: {}", event.name);
                        }
                    }
                }
            }
        }

        let mut days_row = Row::new();
        let mut day_names = [
            "Montag",
            "Dienstag",
            "Mittwoch",
            "Donnerstag",
            "Freitag",
            "Samstag",
            "Sonntag",
        ]
        .iter();

        println!("midnight dist: {}", latest);

        for events in days {
            let day = day_names.next().unwrap();
            let day_box = Container::new(Text::new(*day))
                .width(Length::Fill)
                .height(Length::Fixed(70.0));
            let mut day_col = Column::new().push(day_box);
            let mut last_event_end = 0;
            let mut overall_space = 0;
            for event in events {
                println!("fixed up {} to {}", event.start, event.finish);
                let start = chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap();
                let end = chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap();
                // dirty fix for handling overlapping events
                if last_event_end
                    > (start.time().num_seconds_from_midnight() as u64 * 1000 - earliest)
                {
                    println!("skip!");
                    continue;
                }
                let gap = (start.time().num_seconds_from_midnight() as u64 * 1000 - earliest)
                    - last_event_end;
                last_event_end = end.time().num_seconds_from_midnight() as u64 * 1000 - earliest;
                println!("raw gap: {}", gap / 1000 / 60 / 10 * 9 * 2);
                if gap > 0 {
                    let gap_box =
                        Container::new(Space::with_height(Length::Fill).width(Length::Fill))
                            .height(Length::FillPortion((gap / 1000 / 60 / 10 * 9 * 2) as u16))
                            .width(Length::Fill);
                    day_col = day_col.push(gap_box);
                }
                // FIXME: support events that are happening at the same time (or overlapping)
                let name = event.name.replace("\\, ", "\n");
                day_col = day_col.push(
                    Container::new(Text::new(format!(
                        "({}-{})\n{}\n{}",
                        cut_off_end(&start.time().to_string(), 3),
                        cut_off_end(&end.time().to_string(), 3),
                        name,
                        &event.location
                    )))
                    .width(Length::Fill)
                    .height(Length::FillPortion(
                        ((event.finish - event.start) / 1000 / 60 / 10 * 9 * 2) as u16,
                    ))
                    .style(|theme| bordered_box(theme).shadow(Shadow::default())),
                );
                println!(
                    "raw day {}",
                    (event.finish - event.start) / 1000 / 60 / 10 * 9 * 2
                );
                overall_space += (event.finish - event.start) / 1000 / 60 / 10 * 9 * 2;
                overall_space += gap / 1000 / 60 / 10 * 9 * 2;
            }
            if last_event_end > 0 {
                let day_off = (latest - earliest) - last_event_end;
                println!("day off: {}", day_off / 1000 / 60 / 10 * 9 * 2);
                if day_off > 0 {
                    let gap_box =
                        Container::new(Space::with_height(Length::Fill).width(Length::Fill))
                            .height(Length::FillPortion(
                                (day_off / 1000 / 60 / 10 * 9 * 2) as u16,
                            ))
                            .width(Length::Fill);
                    day_col = day_col.push(gap_box);
                    overall_space += day_off / 1000 / 60 / 10 * 9 * 2;
                }
            }
            println!("overall used space: {}", overall_space);
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
