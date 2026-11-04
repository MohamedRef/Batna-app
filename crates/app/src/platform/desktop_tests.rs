//! What the platform calls do on a desktop without the phone's or the
//! Mac's services: each says "not here" plainly, so the screens fall back
//! instead of waiting or failing.

use std::time::Duration;

use dioxus::prelude::*;

use super::*;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap()
}

#[test]
fn timers_wait_time_out_and_run_blocking_work() {
    runtime().block_on(async {
        let started = std::time::Instant::now();
        time::sleep(Duration::from_millis(20)).await;
        assert!(started.elapsed() >= Duration::from_millis(20));
        assert_eq!(time::timeout(Duration::from_secs(1), async { 7 }).await, Ok(7));
        let slow = time::timeout(Duration::from_millis(10), time::sleep(Duration::from_secs(5)));
        assert_eq!(slow.await, Err(time::Elapsed));
        assert_eq!(time::blocking(|| 2 + 2).await, Ok(4));
    });
}

#[test]
fn phone_only_services_say_they_are_not_here() {
    assert!(!haptics::available());
    haptics::vibrate(haptics::Haptic::Right);
    assert!(!network::metered(), "a desktop is never on mobile data");
    assert_eq!(documents::save("/tmp/x", "x"), Err("not on Android".into()));
    assert_eq!(documents::open(), Err("not on Android".into()));
    assert_eq!(documents::state(), documents::DocState::Failed);
    assert_eq!(runtime().block_on(documents::finished()), documents::DocState::Failed);
    assert_eq!(system_bars::device_dark(), None);
    system_bars::apply("#ffffff", "#000000");
    assert_eq!(background::take(), None);
    background::start(&background::Labels {
        show: "",
        again: "",
        good: "",
        end: "",
        channel: "",
        channel_hint: "",
    });
    background::update("t", "x", false);
    background::stop();
    webview_guard::install();
}

#[test]
fn reminders_and_update_notices_arm_nothing_here() {
    assert!(!reminders::local());
    assert_eq!(reminders::enabled(), None);
    assert_eq!(reminders::battery_unrestricted(), None);
    reminders::schedule("[]");
    reminders::repeat("{}");
    reminders::request_permission();
    reminders::open_notification_settings();
    reminders::open_app_settings();
    assert_eq!(update_notice::take_tap(), None);
    update_notice::schedule(true);
    update_notice::clear();
}

#[test]
fn speech_without_a_voice_reports_it_so_the_text_shows() {
    runtime().block_on(async {
        assert_eq!(tts::say("   ", 1.0).await, Ok(()), "nothing to say");
    });
    assert!(tts::longest("one two three") > tts::longest("one"));
    // No web view here: the voice can't start, and the session goes on.
    fn app() -> Element {
        let mut said = use_signal(|| None::<Result<(), tts::SpeakError>>);
        use_future(move || async move {
            said.set(Some(tts::say("Hepatic fibrosis", 1.0).await));
        });
        tts::stop();
        rsx! { "{said:?}" }
    }
    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..20 {
            if tokio::time::timeout(Duration::from_millis(500), dom.wait_for_work()).await.is_err() {
                break;
            }
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            if !dioxus::ssr::render(&dom).contains("None") {
                break;
            }
        }
        let html = dioxus::ssr::render(&dom);
        assert!(html.contains("Some("), "{html}");
    });
}
