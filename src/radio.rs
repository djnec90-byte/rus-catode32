//! Refcounted lifecycle for the shared WiFi/ESP-NOW radio.
//!
//! esp-radio 0.18 does not expose a `WifiController::stop` — the only way
//! to truly power the WiFi MAC down is to drop the controller. This
//! module owns that drop/recreate dance so the rest of the codebase can
//! request the radio without thinking about it.
//!
//! At rest, the WiFi peripheral handle sits in `ctx.wifi_peripheral` and
//! the radio is fully off. The first call to [`acquire`] consumes the
//! peripheral via `esp_radio::wifi::new`, parks the new controller into
//! `ctx.wifi`, and binds the freshly-minted ESP-NOW handle onto
//! `ctx.espnow`. Each subsequent caller just bumps the refcount.
//! [`release`] decrements; on the last release the controller and
//! ESP-NOW handle are dropped (which de-inits the WiFi MAC) and the
//! peripheral is recovered via `unsafe { WIFI::steal() }` for the next
//! cycle.
//!
//! Two consumers refcount the radio:
//!
//! * [`crate::wifi_tracker::scan_now`] — brackets a single scan with
//!   acquire/release. Scans are infrequent and short.
//! * [`crate::espnow_manager::start_session`] /
//!   [`crate::espnow_manager::stop_session`] — bracket the entire
//!   duration a scene wants ESP-NOW traffic.
//!
//! If a scheduled scan fires during an active social session the
//! refcount stays at 2 and the radio is never cycled mid-session.

use esp_hal::peripherals::WIFI;
use esp_println::println;

use crate::context::GameContext;

/// Bring the radio up (or just bump the refcount). Returns `true` if
/// the radio is usable on exit, `false` if init failed.
///
/// On the 0→1 transition this both initializes the wifi controller
/// and **activates** the ESP-NOW manager (registers the broadcast
/// peer, flips `active`). Subsequent acquires only bump the count —
/// the manager stays activated until the matching final release.
pub fn acquire(ctx: &mut GameContext) -> bool {
    if ctx.radio_users == 0 {
        if !init_radio(ctx) {
            return false;
        }
        if let Some(espnow) = ctx.espnow.as_mut() {
            espnow.activate();
        }
    }
    ctx.radio_users = ctx.radio_users.saturating_add(1);
    true
}

/// Decrement the refcount and tear the radio down on the last release.
/// Tolerant of being called when the count is already zero (logs a
/// warning) so a scene that failed to acquire can still call this
/// unconditionally on exit.
///
/// On the 1→0 transition this **deactivates** the ESP-NOW manager
/// before dropping the wifi controller. Intermediate releases leave
/// the manager active so a scene that hands off to a visit (Social →
/// Inside) doesn't kill the radio session it just established.
pub fn release(ctx: &mut GameContext) {
    if ctx.radio_users == 0 {
        println!("[radio] unbalanced release");
        return;
    }
    ctx.radio_users -= 1;
    if ctx.radio_users == 0 {
        if let Some(espnow) = ctx.espnow.as_mut() {
            espnow.deactivate();
        }
        teardown_radio(ctx);
    }
}

fn init_radio(ctx: &mut GameContext) -> bool {
    let Some(peripheral) = ctx.wifi_peripheral.take() else {
        println!("[radio] no wifi peripheral available");
        return false;
    };
    match esp_radio::wifi::new(peripheral, Default::default()) {
        Ok((controller, interfaces)) => {
            let mac = interfaces.station.mac_address();
            ctx.wifi = Some(controller);
            if let Some(espnow) = ctx.espnow.as_mut() {
                espnow.attach(interfaces.esp_now, mac);
            }
            true
        }
        Err(e) => {
            println!("[radio] wifi::new failed: {:?}", e);
            // `wifi::new` consumed the peripheral; steal it back so a
            // future acquire can try again.
            ctx.wifi_peripheral = Some(unsafe { WIFI::steal() });
            false
        }
    }
}

fn teardown_radio(ctx: &mut GameContext) {
    // Detach ESP-NOW first so the inner handle is dropped before the
    // controller — order shouldn't matter to the driver but the
    // ownership story is easier to reason about this way.
    if let Some(espnow) = ctx.espnow.as_mut() {
        drop(espnow.detach());
    }
    drop(ctx.wifi.take());
    // The controller and ESP-NOW handle have both been dropped now, so
    // the underlying ESP-IDF wifi stack has de-init'd. Steal the
    // peripheral back into the stash for the next acquire.
    ctx.wifi_peripheral = Some(unsafe { WIFI::steal() });
}
