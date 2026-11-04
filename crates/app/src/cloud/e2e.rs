//! The app's sync engine against the real Batna Cloud Worker on local
//! workerd, two devices. Ignored by `cargo test`: `crates/server/tests/e2e.mjs`
//! runs it with `BATNA_E2E_URL` (the Worker) and `BATNA_E2E_TOKEN_URL` (its
//! fake Google, which hands out ID tokens). Nothing leaves the machine.
//! The same scenario also runs in every `cargo test` against the in-process
//! hub (`fake.rs`); `sync_tests.rs` covers more there.

use batna_protocol::cloud::{DeviceInfo, GoogleSignIn, Session};

use super::sync_tests::{Device, answer, eventually};
use super::{Command, Problem, SyncState};

fn sign_in(url: &str, id_token: String, device: &DeviceInfo) -> Session {
    let api = super::api::Api::new(url);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(api.sign_in_google(&GoogleSignIn { id_token, device: device.clone() })).unwrap()
}

#[test]
#[ignore = "needs a local Batna Cloud: run crates/server/tests/e2e.mjs"]
fn phone_and_tablet_sync_through_the_worker() {
    let (Ok(url), Ok(token_url)) = (std::env::var("BATNA_E2E_URL"), std::env::var("BATNA_E2E_TOKEN_URL"))
    else {
        panic!("set BATNA_E2E_URL and BATNA_E2E_TOKEN_URL");
    };
    three_devices(&url, || ureq::get(&token_url).call().unwrap().body_mut().read_to_string().unwrap());
}

#[test]
fn phone_and_tablet_sync_through_the_hub() {
    let cloud = super::fake::FakeCloud::start();
    three_devices(&cloud.url, || super::fake::id_token(super::fake::EMAIL));
}

/// Phone, tablet and laptop on one account: an answer travels, the newest
/// setting wins, a new device catches up, and a remote sign-out lands.
fn three_devices(url: &str, id_token: impl Fn() -> String) {
    let url = url.to_string();
    let session = |d: &DeviceInfo| Some(sign_in(&url, id_token(), d));
    let mut phone = Device::open(&url, "Galaxy S24 Ultra", None, session);
    let mut tablet = Device::open(&url, "Galaxy Tab S11", None, session);
    phone.wait_live();
    tablet.wait_live();

    // An answer on the phone shows on the tablet.
    let started = std::time::Instant::now();
    phone.store.append_review(answer("hb.L01.001")).unwrap();
    phone.send(Command::Kick);
    tablet.wait_changed("review_events");
    let reviews = tablet.store.reviews().unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].event.item_id, "hb.L01.001");
    crate::diag::report!("app e2e: phone → tablet in {} ms", started.elapsed().as_millis());

    // Last writer wins: the tablet's later choice ends up on both.
    phone.store.put_setting("daily_new", &15).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    tablet.store.put_setting("daily_new", &30).unwrap();
    phone.send(Command::Kick);
    tablet.send(Command::Kick);
    eventually("both agree", || {
        phone.store.setting::<u32>("daily_new") == Some(30)
            && tablet.store.setting::<u32>("daily_new") == Some(30)
    });

    // A third device joins: its answers go up, and it catches up on the rest.
    let mut laptop = Device::open(&url, "MacBook Air", None, session);
    laptop.store.append_review(answer("hb.L01.002")).unwrap();
    laptop.store.append_review(answer("hb.L01.003")).unwrap();
    laptop.send(Command::Kick);
    laptop.wait_status("flushed", |s| s.sync == SyncState::Live && s.pending == 0);
    eventually("tablet has all three", || tablet.store.reviews().unwrap().len() == 3);
    assert_eq!(laptop.store.setting::<u32>("daily_new"), Some(30), "and it caught up");

    // The Devices card, then a remote sign-out of the tablet.
    let list = phone.devices().unwrap();
    assert_eq!(list.len(), 3);
    assert!(list.iter().any(|d| d.current && d.name == "Galaxy S24 Ultra"));
    phone.send(Command::RevokeDevice(tablet.id()));
    let s = tablet.wait_status("tablet out", |s| s.sync == SyncState::SignedOut);
    assert_eq!(s.problem, Some(Problem::SignedOutRemotely));
    phone.send(Command::SignOut);
    phone.wait_status("phone out", |s| s.sync == SyncState::SignedOut);
}
