use chrono::*;
use uuid::Uuid;

use std::{collections::BTreeMap, fmt, mem};

use crate::properties::*;
use date_time::{format_utc_date_time, naive_date_to_property, parse_utc_date_time};

pub(crate) mod date_time;
mod event;
mod other;
mod todo;
mod venue;

pub use date_time::{CalendarDateTime, DatePerhapsTime};
pub use event::*;
pub use other::*;
pub use todo::*;
pub use venue::*;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct InnerComponent {
    pub properties: BTreeMap<String, Property>,
    pub multi_properties: Vec<Property>,
}

//impl<'a> Into<InnerComponent> for parser::Component<'a> {
//    fn into(self) -> InnerComponent {
//        unimplemented!()
//    }
//}

impl InnerComponent {
    /// End of builder pattern.
    /// copies over everything
    pub fn done(&mut self) -> Self {
        InnerComponent {
            properties: mem::take(&mut self.properties),
            multi_properties: mem::take(&mut self.multi_properties),
        }
    }
}

/// Implemented by everything that goes into a `Calendar`
pub trait Component {
    /// Returns kind of component.
    ///
    ///
    /// Must be ALL CAPS
    /// These are used in the `BEGIN` and `END` line of the component.
    fn component_kind(&self) -> String;

    /// Allows access to the inner properties map.
    fn properties(&self) -> &BTreeMap<String, Property>;

    /// Read-only access to `multi_properties`
    fn multi_properties(&self) -> &Vec<Property>;

    /// Gets the value of a property.
    fn property_value(&self, key: &str) -> Option<&str> {
        Some(self.properties().get(key)?.value())
    }

    /// Writes [`Component`] using [`std::fmt`].
    fn fmt_write<W: fmt::Write>(&self, out: &mut W) -> Result<(), fmt::Error> {
        write_crlf!(out, "BEGIN:{}", self.component_kind())?;

        if !self.properties().contains_key("DTSTAMP") {
            let now = Utc::now();
            write_crlf!(out, "DTSTAMP:{}", format_utc_date_time(now))?;
        }

        for property in self.properties().values() {
            property.fmt_write(out)?;
        }

        if !self.properties().contains_key("UID") {
            write_crlf!(out, "UID:{}", Uuid::new_v4())?;
        }

        for property in self.multi_properties() {
            property.fmt_write(out)?;
        }

        write_crlf!(out, "END:{}", self.component_kind())?;
        Ok(())
    }

    /// Serializes this component into [`rfc5545`](http://tools.ietf.org/html/rfc5545) again
    ///
    /// # Panic
    /// this can panic if [`std::fmt::write`] returns an Error
    /// use [`Component::try_into_string()`] if you don't like panicking
    fn to_string(&self) -> String {
        self.try_into_string().unwrap()
    }

    /// Serializes this component into [`rfc5545`](http://tools.ietf.org/html/rfc5545) again
    fn try_into_string(&self) -> Result<String, std::fmt::Error> {
        let mut out_string = String::new();
        self.fmt_write(&mut out_string)?;
        Ok(out_string)
    }

    /// Append a given [`Property`]
    fn append_property(&mut self, property: Property) -> &mut Self;

    /// Adds a [`Property`] of which there may be many
    fn append_multi_property(&mut self, property: Property) -> &mut Self;

    /// Construct and append a [`Property`]
    fn add_property(&mut self, key: &str, val: &str) -> &mut Self {
        self.append_property(Property::new(key, val))
    }

    /// Construct and append a [`Property`]
    fn add_multi_property(&mut self, key: &str, val: &str) -> &mut Self {
        self.append_multi_property(Property::new(key, val))
    }

    /// Set the [`DTSTAMP`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.2) [`Property`]
    ///
    /// This must be a UTC date-time value.
    fn timestamp(&mut self, dt: DateTime<Utc>) -> &mut Self {
        self.add_property("DTSTAMP", &format_utc_date_time(dt))
    }

    /// Gets the [`DTSTAMP`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.2) property.
    fn get_timestamp(&self) -> Option<DateTime<Utc>> {
        parse_utc_date_time(self.property_value("DTSTAMP")?)
    }

    /// Gets the [`DTSTART`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.4) [`Property`]
    fn get_start(&self) -> Option<DatePerhapsTime> {
        DatePerhapsTime::from_property(self.properties().get("DTSTART")?)
    }

    /// Gets the [`DTEND`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.2) [`Property`]
    fn get_end(&self) -> Option<DatePerhapsTime> {
        DatePerhapsTime::from_property(self.properties().get("DTEND")?)
    }

    /// Gets the [`RECURRENCE-ID`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.4) [`Property`]
    fn get_recurrence_id(&self) -> Option<DatePerhapsTime> {
        DatePerhapsTime::from_property(self.properties().get("RECURRENCE-ID")?)
    }

    /// Gets the [`EXDATE`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.1) [`Property`]
    fn get_exdates(&self) -> Vec<DatePerhapsTime> {
        self.multi_properties()
            .iter()
            .filter(|prop| prop.key == "EXDATE")
            .filter_map(|prop| DatePerhapsTime::from_property(prop))
            .collect()
    }

    /// Set the [`DTSTART`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.4) [`Property`]
    ///
    /// See [`DatePerhapsTime`] for info how are different [`chrono`] types converted automatically.
    fn starts<T: Into<DatePerhapsTime>>(&mut self, dt: T) -> &mut Self {
        let calendar_dt = dt.into();
        self.append_property(calendar_dt.to_property("DTSTART"))
    }

    /// Set the [`DTEND`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.2) [`Property`]
    ///
    /// See [`DatePerhapsTime`] for info how are different [`chrono`] types converted automatically.
    fn ends<T: Into<DatePerhapsTime>>(&mut self, dt: T) -> &mut Self {
        let calendar_dt = dt.into();
        self.append_property(calendar_dt.to_property("DTEND"))
    }

    /// Set the [`RECURRENCE-ID`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.4) [`Property`]
    ///
    /// See [`DatePerhapsTime`] for info how are different [`chrono`] types converted automatically.
    fn recurrence_id<T: Into<DatePerhapsTime>>(&mut self, dt: T) -> &mut Self {
        let calendar_dt = dt.into();
        self.append_property(calendar_dt.to_property("RECURRENCE-ID"))
    }

    /// Add an [`EXDATE`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.1) [`Property`]
    ///
    /// See [`DatePerhapsTime`] for info how are different [`chrono`] types converted automatically.
    fn exdate<T: Into<DatePerhapsTime>>(&mut self, dt: T) -> &mut Self {
        let calendar_dt = dt.into();
        self.append_multi_property(calendar_dt.to_property("EXDATE"))
    }

    /// Set the [`DTSTART`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.4) [`Property`]
    /// and [`DTEND`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.2) [`Property`],
    /// date only
    fn all_day(&mut self, date: NaiveDate) -> &mut Self {
        self.append_property(naive_date_to_property(date, "DTSTART"))
            .append_property(naive_date_to_property(date, "DTEND"))
    }

    /// Defines the relative priority.
    ///
    /// Ranges from 0 to 10, larger values will be truncated
    fn priority(&mut self, priority: u32) -> &mut Self {
        let priority = ::std::cmp::min(priority, 10);
        self.add_property("PRIORITY", &priority.to_string())
    }

    /// Gets the relative priority.
    ///
    /// Ranges from 0 to 10.
    fn get_priority(&self) -> Option<u32> {
        let priority = self.property_value("PRIORITY")?.parse().ok()?;
        if priority <= 10 {
            Some(priority)
        } else {
            None
        }
    }

    /// Prints to stdout
    fn print(&self) -> Result<(), fmt::Error> {
        let mut out = String::new();
        self.fmt_write(&mut out)?;
        print_crlf!("{}", out);
        Ok(())
    }

    /// Set the summary
    fn summary(&mut self, desc: &str) -> &mut Self {
        self.add_property("SUMMARY", desc)
    }

    /// Gets the summary
    fn get_summary(&self) -> Option<&str> {
        self.property_value("SUMMARY")
    }

    /// Set the description
    fn description(&mut self, desc: &str) -> &mut Self {
        self.add_property("DESCRIPTION", desc)
    }

    /// Gets the description
    fn get_description(&self) -> Option<&str> {
        self.property_value("DESCRIPTION")
    }

    ///// Set the description
    ///// TODO `Attendee` needs to be its own type
    //fn attendee(&mut self, desc: &str) -> &mut Self {
    //    self.add_multi_property("ATTENDEE", desc) // multi_properties should be a multi-map
    //}

    /// Set the LOCATION
    /// 3.8.1.7.  Location
    fn location(&mut self, location: &str) -> &mut Self {
        self.add_property("LOCATION", location)
    }

    /// Gets the location
    fn get_location(&self) -> Option<&str> {
        self.property_value("LOCATION")
    }

    /// Set the LOCATION with a VVENUE UID
    /// iCalender venue draft
    fn venue(&mut self, location: &str, venue_uid: &str) -> &mut Self {
        self.append_property(
            Property::new("LOCATION", location)
                .append_parameter(Parameter::new("VVENUE", venue_uid))
                .done(),
        );
        self
    }

    /// Set the UID
    fn uid(&mut self, uid: &str) -> &mut Self {
        self.add_property("UID", uid)
    }

    /// Gets the UID
    fn get_uid(&self) -> Option<&str> {
        self.property_value("UID")
    }

    /// Set the visibility class
    fn class(&mut self, class: Class) -> &mut Self {
        self.append_property(class.into())
    }

    /// Gets the visibility class
    fn get_class(&self) -> Option<Class> {
        Class::from_str(self.property_value("CLASS")?)
    }

    /// Sets the URL.
    fn url(&mut self, url: &str) -> &mut Self {
        self.add_property("URL", url)
    }

    /// Gets the URL.
    fn get_url(&self) -> Option<&str> {
        self.property_value("URL")
    }
}

macro_rules! component_impl {
    ($t:ty, $kind:expr) => {
        impl Component for $t {
            /// Tells you what kind of [`Component`] this is
            ///
            /// Might be [`VEVENT`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.1), [`VTODO`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.2), [`VALARM`](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.6) etc
            fn component_kind(&self) -> String {
                $kind
            }

            /// Read-only access to `properties`
            fn properties(&self) -> &BTreeMap<String, Property> {
                &self.inner.properties
            }

            /// Read-only access to `multi_properties`
            fn multi_properties(&self) -> &Vec<Property> {
                &self.inner.multi_properties
            }

            /// Adds a [`Property`]
            fn append_property(&mut self, property: Property) -> &mut Self {
                self.inner
                    .properties
                    .insert(property.key().to_owned(), property);
                self
            }

            /// Adds a [`Property`] of which there may be many
            fn append_multi_property(&mut self, property: Property) -> &mut Self {
                self.inner.multi_properties.push(property);
                self
            }
        }

        impl From<InnerComponent> for $t {
            fn from(inner: InnerComponent) -> $t {
                Self { inner }
            }
        }
    };
}

component_impl! { Event, String::from("VEVENT") }
component_impl! { Todo , String::from("VTODO")}
component_impl! { Venue , String::from("VVENUE")}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_properties_unset() {
        let event = Event::new();
        assert_eq!(event.get_priority(), None);
        assert_eq!(event.get_summary(), None);
        assert_eq!(event.get_description(), None);
        assert_eq!(event.get_location(), None);
        assert_eq!(event.get_uid(), None);
        assert_eq!(event.get_class(), None);
        assert_eq!(event.get_timestamp(), None);
        assert_eq!(event.get_url(), None);
    }

    #[test]
    fn get_properties_set() {
        let event = Event::new()
            .priority(5)
            .summary("summary")
            .description("description")
            .location("location")
            .uid("uid")
            .class(Class::Private)
            .url("http://some.test/url")
            .done();
        assert_eq!(event.get_priority(), Some(5));
        assert_eq!(event.get_summary(), Some("summary"));
        assert_eq!(event.get_description(), Some("description"));
        assert_eq!(event.get_location(), Some("location"));
        assert_eq!(event.get_uid(), Some("uid"));
        assert_eq!(event.get_class(), Some(Class::Private));
        assert_eq!(event.get_url(), Some("http://some.test/url"));
    }

    #[test]
    fn get_date_times_naive() {
        let naive_date_time = NaiveDate::from_ymd(2001, 3, 13).and_hms(14, 15, 16);
        let event = Event::new()
            .starts(naive_date_time)
            .ends(naive_date_time)
            .done();
        assert_eq!(event.get_start(), Some(naive_date_time.into()));
        assert_eq!(event.get_end(), Some(naive_date_time.into()));
    }

    #[test]
    fn get_date_times_utc() {
        let utc_date_time = Utc.ymd(2001, 3, 13).and_hms(14, 15, 16);
        let event = Event::new()
            .timestamp(utc_date_time)
            .starts(utc_date_time)
            .ends(utc_date_time)
            .done();
        assert_eq!(event.get_timestamp(), Some(utc_date_time));
        assert_eq!(event.get_start(), Some(utc_date_time.into()));
        assert_eq!(event.get_end(), Some(utc_date_time.into()));
    }

    #[test]
    fn get_date_times_tzid() {
        let date_time = NaiveDate::from_ymd(2001, 3, 13).and_hms(14, 15, 16);
        let date_time_tzid = CalendarDateTime::WithTimezone {
            date_time,
            tzid: "Pacific/Auckland".to_string(),
        };
        let event = Event::new()
            .starts(date_time_tzid.clone())
            .ends(date_time_tzid.clone())
            .done();
        assert_eq!(event.get_start(), Some(date_time_tzid.clone().into()));
        assert_eq!(event.get_end(), Some(date_time_tzid.into()));
    }

    #[test]
    fn get_dates_naive() {
        let naive_date = NaiveDate::from_ymd(2001, 3, 13);
        let event = Event::new().starts(naive_date).ends(naive_date).done();
        assert_eq!(event.get_start(), Some(naive_date.into()));
        assert_eq!(event.get_end(), Some(naive_date.into()));
    }
}
