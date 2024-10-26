mod config;

use std::sync::{Arc, Mutex, RwLock};
use chrono::{Days, Utc};
use iced::widget::{
    button, center, checkbox, column, horizontal_rule, pick_list, progress_bar,
    row, scrollable, slider, text, text_input, toggler, vertical_rule,
    vertical_space,
};
use iced::{Center, Element, Fill, Theme};

use config::Config;
use tokio::runtime::{Builder, Runtime};

fn main() {
    iced::application("Styling - Iced", Calendar::update, Calendar::view)
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

    fn update(&mut self, message: Message) {

    }

    fn view(&self) -> Element<Message> {
        println!("view!");
        /*let mut days = {
            let mut days = vec![];
            for _ in 0..7 {
                days.push(vec![]);
            }
            days
        };*/
        let now = Utc::now();
        for cal in &self.config.calendars {
            println!("got cal!");
            for event in cal.get_events(&self.rt).iter() {
                println!("got events: start {} ende {} name {} loc {} rep {:?}", event.start, event.finish, event.name, event.location, event.repeat/*reqwest::get(&event.name).unwrap().text().unwrap()*/);
                if event.start == 0 {
                    continue;
                }
                println!("passed 0");
                if chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap().checked_sub_days(Days::new(7)).unwrap() > chrono::Utc::now() {
                    continue;
                }
                println!("passed 1");
                if chrono::DateTime::from_timestamp_millis(event.finish as i64).unwrap() < chrono::Utc::now() {
                    continue;
                }
                println!("passed checks");
                let start = chrono::DateTime::from_timestamp_millis(event.start as i64).unwrap();
                for i in 0..6 {
                    if start.clone().checked_add_days(Days::new(i)).unwrap().date_naive() == now.date_naive() {
                        /*days[i as usize].push(row![
                            text(&event.name)
                        ]);*/
                        println!("Got {}", event.name);
                    }
                }
            }
        }
        let choose_theme = column![
            text("Theme:")
        ]
        .spacing(10);
        center(column![choose_theme]).into()
    }

}

#[derive(Debug)]
enum Message {
    RefreshCalendars,
}
