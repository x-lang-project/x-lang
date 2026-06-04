use std::fmt::{Debug, Display};

use crate::Error;

pub fn missing_return_slot_layout_tracking_record() -> String {
    Error::new("missing return-slot layout").render_detail()
}
