use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Scheduled appointment slot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentSlot {
    /// Slot start time.
    pub start_time: NaiveDateTime,
    /// Slot end time.
    pub end_time: NaiveDateTime,
    /// Host sales rep / advisor user ID.
    pub host_user: String,
    /// Booked attendee email.
    pub attendee_email: Option<String>,
    /// Booked attendee name.
    pub attendee_name: Option<String>,
    /// Appointment title/subject.
    pub subject: Option<String>,
    /// Booking status.
    pub is_booked: bool,
}

/// Calendar iCalendar / ICS payload model for automated email invites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarInvitePayload {
    /// Unique event UID.
    pub uid: String,
    /// Event summary / title.
    pub summary: String,
    /// Start ISO string.
    pub dt_start: String,
    /// End ISO string.
    pub dt_end: String,
    /// Organizer email.
    pub organizer: String,
    /// Attendee email.
    pub attendee: String,
}

/// Appointment Booking Engine preventing double-booking and managing agent calendars.
#[derive(Debug, Default, Clone)]
pub struct AppointmentBookingEngine {
    /// All registered slots.
    pub slots: Vec<AppointmentSlot>,
}

impl AppointmentBookingEngine {
    /// Creates a new empty appointment booking engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an available time slot for a host user.
    pub fn add_available_slot(
        &mut self,
        host_user: &str,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) {
        self.slots.push(AppointmentSlot {
            start_time,
            end_time,
            host_user: host_user.to_string(),
            attendee_email: None,
            attendee_name: None,
            subject: None,
            is_booked: false,
        });
    }

    /// Attempts to book an available slot, returning calendar invite payload on success.
    pub fn book_slot(
        &mut self,
        host_user: &str,
        start_time: NaiveDateTime,
        attendee_name: &str,
        attendee_email: &str,
        subject: &str,
    ) -> Result<CalendarInvitePayload, String> {
        let slot = self
            .slots
            .iter_mut()
            .find(|s| s.host_user == host_user && s.start_time == start_time);

        match slot {
            Some(s) if !s.is_booked => {
                s.is_booked = true;
                s.attendee_name = Some(attendee_name.to_string());
                s.attendee_email = Some(attendee_email.to_string());
                s.subject = Some(subject.to_string());

                let uid = format!("{}-{}-{}", host_user, start_time.and_utc().timestamp(), attendee_email);
                Ok(CalendarInvitePayload {
                    uid,
                    summary: subject.to_string(),
                    dt_start: s.start_time.to_string(),
                    dt_end: s.end_time.to_string(),
                    organizer: host_user.to_string(),
                    attendee: attendee_email.to_string(),
                })
            }
            Some(_) => Err("Slot is already booked".into()),
            None => Err("Requested slot not found".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_appointment_booking_flow() {
        let mut engine = AppointmentBookingEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let start = date.and_hms_opt(10, 0, 0).unwrap();
        let end = date.and_hms_opt(11, 0, 0).unwrap();

        engine.add_available_slot("sales@acme.com", start, end);

        let invite = engine
            .book_slot(
                "sales@acme.com",
                start,
                "John Prospect",
                "john@prospect.com",
                "ERPNext Demo",
            )
            .expect("Booking failed");

        assert_eq!(invite.summary, "ERPNext Demo");
        assert_eq!(invite.attendee, "john@prospect.com");

        // Attempting to book the same slot again should fail
        assert!(
            engine
                .book_slot(
                    "sales@acme.com",
                    start,
                    "Jane Prospect",
                    "jane@prospect.com",
                    "Second Booking",
                )
                .is_err()
        );
    }
}
