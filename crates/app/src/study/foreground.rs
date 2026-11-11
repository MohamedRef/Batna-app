//! Browser foreground bridge. The active question and scroll remain mounted;
//! only the answer's elapsed time resets after returning from another app.
//! A killed Android process needs a separate, durable resume feature.

use dioxus::prelude::*;
use web_time::Instant;

use super::{PHASE, Phase, STARTED};

const FOREGROUND_JS: &str = include_str!("foreground.js");

/// Install once at the app shell, not once per Study mount. This avoids
/// stale eval senders after the user switches between Study and other tabs.
pub(crate) fn use_foreground_reset() {
    use_future(|| async move {
        let mut eval = document::eval(FOREGROUND_JS);
        while let Ok(message) = eval.recv::<String>().await {
            if message == "visible" && *PHASE.peek() == Phase::Running {
                *STARTED.write() = Instant::now();
            }
        }
    });
}
