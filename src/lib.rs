#![allow(unused_variables)]

extern crate chrono;
extern crate uuid;

use chrono::*;
use uuid::Uuid;

pub mod period;
use period::Period;

#[derive(Debug,Clone)]
pub enum Repeats {
    Annually,
    Years(usize),
    Months(usize),
    Weeks(usize),
    Days(usize),
    Daily,
    Not,
}

pub enum Numeration {
    First,
    Second,
    Third,
    Forth,
    Fifth,
    Sixth,
    Seventh,
    Last,
}

impl Numeration {
    pub fn monday() -> Repeats {
        Repeats::Not
    }
}

pub trait Repeatable {
    fn repeats(&mut self, repeat: Repeats) -> &mut Self;
    fn every(&mut self, repeat: Repeats) -> &mut Self;
    fn on(&mut self, repeat: Repeats) -> &mut Self;
}

#[derive(Debug,Clone)]
pub struct Event {
    summary: String,
    description: String,
    start: DateTime<Local>,
    time_stamp: DateTime<Local>,
    uuid: Uuid,
    repeats: Repeats,
}

impl Event {
    pub fn new(summary: &str) -> Event {
        Event {
            summary: String::from(summary),
            description: String::new(),
            uuid: Uuid::new_v4(),
            time_stamp: Local::now(),
            start: Local::now(),
            repeats: Repeats::Not,
        }
    }

    pub fn starts<D: Datelike>(&mut self, date: D) -> &mut Self {
        self
    }

    pub fn done(&mut self) -> Self {
        self.clone()
    }
}

impl Repeatable for Event {
    fn repeats(&mut self, repeat: Repeats) -> &mut Self {
        self.repeats = repeat;
        self
    }

    fn every(&mut self, repeat: Repeats) -> &mut Self {
        Self::repeats(self, repeat)
    }

    fn on(&mut self, repeat: Repeats) -> &mut Self {
        Self::repeats(self, repeat)
    }
}

#[test]
fn it_works() {
    use Repeats::*;
    let birthday = Event::new("My Birthday").repeats(Annually).done();
    let birthday = Event::new("My Birthday").every(15.days()).done();
    println!("{:#?}", birthday);
}
