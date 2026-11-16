//! Settings › Sync › the verdicts read key (#1352): Mohamed makes a key
//! that lets `batna-tools verdicts --cloud` read his ✓ ✎ ✗ marks from Batna
//! Cloud, so the fix list is current without a file. Shown once; a new key
//! replaces the old; removing it stops it at once (`batna_server::read_key`).

use dioxus::prelude::*;

use super::ui::{Cloud, problem_text};
use super::{Command, KeyOp, SyncState};
use crate::i18n::cloud as tx;

#[component]
pub fn ReadKeyCard() -> Element {
    let cloud = use_context::<Cloud>();
    let key = cloud.read_key.read().clone();
    let state = use_memo(move || cloud.status.read().sync);
    use_effect(move || {
        if matches!(state(), SyncState::Live | SyncState::CatchingUp)
            && cloud.read_key.peek().created_ms.is_none()
        {
            cloud.send(Command::ReadKey(KeyOp::Load));
        }
    });
    let mut confirm = use_signal(|| false);
    let mut copied = use_signal(|| false);
    let mut ask = move |op: KeyOp| {
        let mut signal = cloud.read_key;
        signal.with_mut(|k| {
            k.busy = true;
            k.problem = None;
            k.new_key = None;
        });
        confirm.set(false);
        copied.set(false);
        cloud.send(Command::ReadKey(op));
    };
    let has_key = matches!(key.created_ms, Some(Some(_)));

    rsx! {
        crate::ui::Card { class: "read-key",
            h2 { {tx::KEY_TITLE} }
            p { class: "small muted", {tx::KEY_ABOUT} }
            match key.created_ms {
                None if key.problem.is_none() => rsx! { p { class: "small muted", {tx::KEY_LOADING} } },
                Some(Some(ms)) => {
                    let (_, month, day) = crate::store::zone().calendar_day(ms as i64).ymd();
                    rsx! { p { class: "small", {tx::key_since(day, crate::i18n::plan::MONTHS[month as usize - 1])} } }
                }
                Some(None) => rsx! { p { class: "small", {tx::KEY_NONE} } },
                None => rsx! {},
            }
            if let Some(new) = key.new_key.clone() {
                p { class: "small", role: "status", {tx::KEY_NEW} }
                textarea { class: "export-text read-key-text", dir: "ltr", readonly: true, rows: 2, value: "{new}" }
                button {
                    class: "btn btn-secondary read-key-copy",
                    onclick: move |_| {
                        let text = serde_json::to_string(&new).unwrap_or_default();
                        document::eval(&format!("navigator.clipboard.writeText({text})"));
                        copied.set(true);
                    },
                    {if copied() { tx::KEY_COPIED } else { tx::KEY_COPY }}
                }
            }
            if key.busy {
                p { class: "small muted", aria_live: "polite", {tx::KEY_BUSY} }
            } else if confirm() {
                p { class: "small", role: "alert", {tx::KEY_REMOVE_SURE} }
                div { class: "sync-actions",
                    button { class: "btn btn-ghost read-key-remove-yes", onclick: move |_| ask(KeyOp::Remove), {tx::KEY_REMOVE_YES} }
                    button { class: "btn btn-primary read-key-keep", onclick: move |_| confirm.set(false), {tx::KEY_REMOVE_NO} }
                }
            } else if key.created_ms.is_some() {
                if has_key {
                    p { class: "small muted", {tx::KEY_REMAKE_NOTE} }
                }
                div { class: "sync-actions",
                    button { class: "btn btn-primary read-key-make", onclick: move |_| ask(KeyOp::Make),
                        {if has_key { tx::KEY_REMAKE } else { tx::KEY_MAKE }}
                    }
                    if has_key {
                        button { class: "btn btn-secondary read-key-remove", onclick: move |_| confirm.set(true), {tx::KEY_REMOVE} }
                    }
                }
            }
            if let Some(p) = key.problem {
                p { class: "small", role: "status", {problem_text(p)} }
            }
        }
    }
}

/// The read key from end to end: Settings makes it, and the tools' request
/// reads the synced marks with it. Runs against the in-process hub in every
/// `cargo test` and against the real Worker in `e2e.rs`.
#[cfg(test)]
pub(super) fn round_trip(url: &str, phone: &mut super::sync_tests::Device) {
    use batna_core::review::verify::{Judgement, Verdict};
    use batna_protocol::cloud::VerdictRows;

    use super::Event;

    let key_event = |phone: &mut super::sync_tests::Device, op| {
        phone.send(Command::ReadKey(op));
        phone.wait("read key", |e| match e {
            Event::ReadKey(k) => Some(k.clone()),
            _ => None,
        })
    };
    let read = |key: &str| -> Result<VerdictRows, u16> {
        let answer = ureq::get(&format!("{url}/v1/verdicts"))
            .header("authorization", &format!("Bearer {key}"))
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .unwrap();
        let status = answer.status().as_u16();
        let mut body = answer.into_body();
        if status == 200 {
            Ok(serde_json::from_str(&body.read_to_string().unwrap()).unwrap())
        } else {
            Err(status)
        }
    };

    let wrong = Judgement {
        verdict: Verdict::Wrong,
        edit: Default::default(),
        note: Some("page 14 says 7-9".into()),
        fingerprint: "0123456789abcdef".into(),
        at_ms: crate::store::now_ms(),
    };
    phone.store.set_judgement("hb.L01.004", Some(&wrong)).unwrap();
    phone.send(Command::Kick);
    phone.wait_status("verdict up", |s| s.pending == 0);

    assert_eq!(key_event(phone, KeyOp::Load).created_ms, Some(None), "no key yet");
    let made = key_event(phone, KeyOp::Make);
    assert!(matches!(made.created_ms, Some(Some(_))) && made.problem.is_none(), "{made:?}");
    let key = made.new_key.clone().expect("the new key, once");
    assert!(!format!("{made:?}").contains(&key), "the key stays out of logs");
    let rows = read(&key).unwrap().rows;
    let row = rows.iter().find(|r| r.item_id == "hb.L01.004").expect("the synced mark");
    assert_eq!(serde_json::from_str::<Judgement>(&row.value).unwrap(), wrong);
    assert_eq!(read("bvk1.nope.x").unwrap_err(), 401);

    // A new key replaces the old; removing it stops it.
    let again = key_event(phone, KeyOp::Make).new_key.unwrap();
    assert_eq!(read(&key).unwrap_err(), 401);
    assert!(read(&again).is_ok());
    let removed = key_event(phone, KeyOp::Remove);
    assert_eq!(removed.created_ms, Some(None));
    assert_eq!(removed.new_key, None);
    assert_eq!(read(&again).unwrap_err(), 401);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use super::super::{CloudStatus, Problem, ReadKeyStatus};
    use super::*;
    use crate::tap_dom::TestDom;

    thread_local! {
        static CASE: RefCell<Option<(ReadKeyStatus, tokio::sync::mpsc::UnboundedSender<Command>)>> =
            const { RefCell::new(None) };
    }

    fn app() -> Element {
        let (key, sent) = CASE.with(|c| c.borrow().clone()).unwrap();
        let mut status = CloudStatus::not_configured("phone-1".into());
        status.configured = true;
        status.email = Some("mohamed@example.com".into());
        status.sync = SyncState::Live;
        use_context_provider(|| super::super::ui::test_cloud(status, key, sent));
        rsx! { ReadKeyCard {} }
    }

    fn mount(key: ReadKeyStatus) -> (TestDom, UnboundedReceiver<Command>) {
        let (tx, rx) = unbounded_channel();
        CASE.with(|c| *c.borrow_mut() = Some((key, tx)));
        let mut t = TestDom::of(app);
        t.settle();
        (t, rx)
    }

    fn sent(rx: &mut UnboundedReceiver<Command>) -> Vec<String> {
        std::iter::from_fn(|| rx.try_recv().ok()).map(|c| format!("{c:?}")).collect()
    }

    #[test]
    fn it_asks_once_live_then_offers_to_make_a_key() {
        let (t, mut rx) = mount(ReadKeyStatus::default());
        assert!(t.html().contains(tx::KEY_LOADING));
        assert_eq!(sent(&mut rx), ["ReadKey(Load)"]);
        let (mut t, mut rx) = mount(ReadKeyStatus { created_ms: Some(None), ..Default::default() });
        assert!(t.html().contains(tx::KEY_NONE) && !t.html().contains(tx::KEY_REMOVE), "{}", t.html());
        t.click(tx::KEY_MAKE);
        assert_eq!(sent(&mut rx), ["ReadKey(Make)"]);
        assert!(t.html().contains(tx::KEY_BUSY));
    }

    #[test]
    fn a_new_key_is_shown_once_and_removing_asks_first() {
        let made = ReadKeyStatus {
            created_ms: Some(Some(1_794_000_000_000)),
            new_key: Some("bvk1.g0123456789abcdef01234567.secret".into()),
            ..Default::default()
        };
        let (mut t, mut rx) = mount(made);
        let html = t.html();
        assert!(
            html.contains(tx::KEY_NEW) && html.contains("bvk1.g0123456789abcdef01234567.secret"),
            "{html}"
        );
        assert!(html.contains(tx::KEY_REMAKE) && html.contains(tx::KEY_REMAKE_NOTE));
        t.click("read-key-copy");
        assert!(t.html().contains(tx::KEY_COPIED));
        t.click("read-key-remove");
        assert!(t.html().contains(tx::KEY_REMOVE_SURE));
        t.click("read-key-keep");
        assert!(!t.html().contains(tx::KEY_REMOVE_SURE));
        assert!(sent(&mut rx).is_empty(), "nothing asked yet");
        t.click("read-key-remove");
        t.click("read-key-remove-yes");
        assert_eq!(sent(&mut rx), ["ReadKey(Remove)"]);
        // Making a new one hides the shown key at once.
        assert!(!t.html().contains("bvk1."));
    }

    #[test]
    fn a_problem_is_said() {
        let (t, _rx) = mount(ReadKeyStatus { problem: Some(Problem::Network), ..Default::default() });
        assert!(t.html().contains(super::super::ui::problem_text(Problem::Network)));
        assert!(!t.html().contains(tx::KEY_LOADING));
    }
}
