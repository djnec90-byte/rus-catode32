//! ESP-NOW transport layer.
//!
//! Wraps [`esp_radio::esp_now::EspNow`] with a small in-process inbox
//! and a fixed-size set of registered unicast peers. The manager lives
//! on [`GameContext`] for the program's lifetime, but its inner
//! `EspNow` handle is only bound while the radio is acquired by
//! [`crate::radio`] — operations are silent no-ops otherwise. This
//! keeps the manager addressable from any scene without having to
//! propagate radio-up/down state through every method signature.
//!
//! "Clean break" Rust protocol: messages are framed as a 4-byte ASCII
//! tag (e.g. `b"vst "`, `b"hi  "`) followed by a binary payload
//! encoded by the message-specific module. We do not interop with the
//! legacy MicroPython JSON wire format.
//!
//! Most of the public API is unused until later porting phases wire the
//! social scene and the playdate runtime; the module-level
//! `allow(dead_code)` keeps the build clean in the meantime.

#![allow(dead_code)]

use esp_println::println;
use heapless::Vec;

use esp_radio::esp_now::{
    EspNow, EspNowError, EspNowWifiInterface, PeerInfo, ReceivedData, BROADCAST_ADDRESS,
    ESP_NOW_MAX_DATA_LEN,
};

use crate::{
    context::GameContext,
    espnow_msg::{encode_empty, TAG_VBYE},
    radio,
};

/// MAC address newtype. Stored as raw 6 bytes; formatted as
/// `aa:bb:cc:dd:ee:ff` for logs and save files via [`format_mac`].
pub type MacAddr = [u8; 6];

/// Broadcast destination — re-exported for callers.
pub const BROADCAST: MacAddr = BROADCAST_ADDRESS;

/// Max bytes we will ever buffer for a single inbound or outbound message.
/// Sized to the ESP-NOW MTU so we never silently truncate.
pub const MAX_PAYLOAD: usize = ESP_NOW_MAX_DATA_LEN;

/// How many inbound messages we will hold between polls before dropping
/// the oldest. Sized to match the legacy MicroPython queue depth.
pub const INBOX_CAPACITY: usize = 8;

/// How many distinct unicast peers we will register at once. The
/// broadcast peer is always registered separately and does not count
/// against this.
pub const MAX_UNICAST_PEERS: usize = 4;

/// One received frame. The payload's first 4 bytes are the message tag
/// (e.g. `b"vst "`); the rest is the message-specific binary body.
pub struct InboxItem {
    pub src: MacAddr,
    pub data: Vec<u8, MAX_PAYLOAD>,
}

/// ESP-NOW transport state. The inner driver handle is only bound while
/// [`crate::radio::acquire`] has the WiFi MAC up; calls made while
/// detached are no-ops.
pub struct EspNowManager {
    inner: Option<EspNow<'static>>,
    /// Cached hardware MAC. Populated on the first `attach` and
    /// retained across power cycles so the debug scene can keep
    /// showing it even when the radio is down.
    own_mac: Option<MacAddr>,
    inbox: Vec<InboxItem, INBOX_CAPACITY>,
    peers: Vec<MacAddr, MAX_UNICAST_PEERS>,
    /// True between [`activate`] and [`deactivate`]. Send paths require
    /// active state; receive path requires the inner handle too.
    active: bool,
}

impl EspNowManager {
    /// Empty manager. The inner handle and MAC are populated lazily by
    /// `attach` once the radio is brought up.
    pub fn new() -> Self {
        Self {
            inner: None,
            own_mac: None,
            inbox: Vec::new(),
            peers: Vec::new(),
            active: false,
        }
    }

    /// Returns the cached hardware MAC, if the radio has been brought
    /// up at least once. `None` only on a cold boot before any acquire.
    pub fn own_mac(&self) -> Option<MacAddr> {
        self.own_mac
    }

    /// Called by [`crate::radio::acquire`] right after a fresh
    /// `esp_radio::wifi::new` succeeds. Hands the driver handle to the
    /// manager and caches the MAC if this is the first attach.
    pub fn attach(&mut self, inner: EspNow<'static>, own_mac: MacAddr) {
        self.inner = Some(inner);
        if self.own_mac.is_none() {
            self.own_mac = Some(own_mac);
        }
    }

    /// Called by [`crate::radio::release`] right before dropping the
    /// wifi controller. Returns the inner handle so the caller can
    /// drop it; the manager is left detached (no-op until reattached).
    pub fn detach(&mut self) -> Option<EspNow<'static>> {
        self.active = false;
        self.inbox.clear();
        self.peers.clear();
        self.inner.take()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Register the broadcast peer and flip into "listening" mode.
    /// Caller must have called [`crate::radio::acquire`] first so the
    /// inner handle is bound.
    pub fn activate(&mut self) {
        if self.active {
            return;
        }
        let Some(inner) = self.inner.as_mut() else {
            println!("[EspNow] activate without radio");
            return;
        };
        let peer = PeerInfo {
            interface: EspNowWifiInterface::Station,
            peer_address: BROADCAST,
            lmk: None,
            channel: None,
            encrypt: false,
        };
        if let Err(e) = inner.add_peer(peer) {
            if !matches!(e, EspNowError::Error(_)) {
                println!("[EspNow] add broadcast peer failed: {:?}", e);
            }
        }
        self.active = true;
    }

    /// Deregister every peer we added and drop pending inbox frames.
    /// The driver handle stays bound — only [`crate::radio::release`]
    /// detaches it. Subsequent calls until the next `activate` are
    /// no-ops.
    pub fn deactivate(&mut self) {
        if !self.active {
            return;
        }
        if let Some(inner) = self.inner.as_mut() {
            for mac in self.peers.iter() {
                let _ = inner.remove_peer(mac);
            }
            let _ = inner.remove_peer(&BROADCAST);
        }
        self.peers.clear();
        self.inbox.clear();
        self.active = false;
    }

    /// Register a unicast peer if we have not already. Quietly drops
    /// when the peer table is full or the radio is detached.
    pub fn add_peer(&mut self, mac: MacAddr) {
        if !self.active {
            return;
        }
        if self.peers.iter().any(|m| *m == mac) {
            return;
        }
        if self.peers.is_full() {
            return;
        }
        let Some(inner) = self.inner.as_mut() else {
            return;
        };
        let info = PeerInfo {
            interface: EspNowWifiInterface::Station,
            peer_address: mac,
            lmk: None,
            channel: None,
            encrypt: false,
        };
        match inner.add_peer(info) {
            Ok(_) => {
                let _ = self.peers.push(mac);
            }
            Err(e) => println!("[EspNow] add_peer({:?}) failed: {:?}", mac, e),
        }
    }

    /// Broadcast `data` to every device in range. `data` must already
    /// include the 4-byte message tag at offset 0. Silently drops if
    /// the payload is too large, the manager is inactive, or the
    /// driver handle is detached.
    pub fn send_broadcast(&mut self, data: &[u8]) {
        if !self.active || data.len() > MAX_PAYLOAD {
            return;
        }
        let Some(inner) = self.inner.as_mut() else {
            return;
        };
        let _ = inner.send(&BROADCAST, data);
    }

    /// Unicast `data` to a previously-registered peer. Same framing
    /// rules as `send_broadcast`; silently drops if the peer is not
    /// registered or the manager is detached.
    pub fn send_to(&mut self, mac: MacAddr, data: &[u8]) {
        if !self.active || data.len() > MAX_PAYLOAD {
            return;
        }
        if !self.peers.iter().any(|m| *m == mac) {
            return;
        }
        let Some(inner) = self.inner.as_mut() else {
            return;
        };
        let _ = inner.send(&mac, data);
    }

    /// Pull every pending packet from the driver into our inbox.
    /// Called once per game-loop tick by `Game`. Self-broadcasts are
    /// dropped; the oldest item is evicted if the inbox fills.
    pub fn poll(&mut self) {
        if !self.active {
            return;
        }
        let own = self.own_mac;
        let Some(inner) = self.inner.as_mut() else {
            return;
        };
        loop {
            let recv: Option<ReceivedData> = inner.receive();
            let Some(frame) = recv else { break };
            let src = frame.info.src_address;
            if Some(src) == own {
                continue;
            }
            let mut buf: Vec<u8, MAX_PAYLOAD> = Vec::new();
            for b in frame.data() {
                if buf.push(*b).is_err() {
                    break;
                }
            }
            if self.inbox.is_full() {
                let _ = self.inbox.remove(0);
            }
            let _ = self.inbox.push(InboxItem { src, data: buf });
        }
    }

    /// Drain the inbox. Caller (per-frame dispatch in `Game`) iterates,
    /// classifies, and forwards messages to the appropriate
    /// scene/handler.
    pub fn drain(&mut self) -> Vec<InboxItem, INBOX_CAPACITY> {
        core::mem::take(&mut self.inbox)
    }
}

/// Acquire the radio for ESP-NOW traffic. Scenes call this on
/// `enter`. The first acquire (refcount 0→1) brings the wifi
/// controller up and activates the ESP-NOW manager; subsequent
/// acquires only bump the refcount. Returns `true` if the manager
/// is ready to send/receive.
pub fn start_session(ctx: &mut GameContext) -> bool {
    radio::acquire(ctx)
}

/// Counterpart to [`start_session`]. Tolerant of being called even if
/// `start_session` failed earlier. Only the **last** matching release
/// (refcount 1→0) actually deactivates the manager and drops the
/// controller — intermediate releases just decrement.
pub fn stop_session(ctx: &mut GameContext) {
    radio::release(ctx);
}

/// Begin a playdate by taking a dedicated radio refcount on behalf of
/// the visit. The caller (the social scene) has already established
/// the visit metadata in `ctx.visit`; this exists separately so the
/// visit's lifetime is independent of whichever scene happens to own
/// the radio at the moment.
///
/// On success the broadcast peer registration that
/// [`start_session`] put in place is reused — we do not re-activate
/// the manager. Returns `true` if the radio came up.
pub fn acquire_visit_radio(ctx: &mut GameContext) -> bool {
    radio::acquire(ctx)
}

/// End the active playdate. Optionally tells the peer (`notify_peer =
/// true`) by sending a `vbye` frame, then drops the visit's radio
/// refcount and clears `ctx.visit`. Tolerant of being called when no
/// visit is active (no-op).
pub fn end_visit(ctx: &mut GameContext, notify_peer: bool) {
    let Some(visit) = ctx.visit.take() else {
        return;
    };
    if notify_peer {
        if let Some(espnow) = ctx.espnow.as_mut() {
            let frame = encode_empty(TAG_VBYE);
            espnow.send_to(visit.peer_mac, &frame);
        }
    }
    radio::release(ctx);
}

/// Format a MAC as `aa:bb:cc:dd:ee:ff`. Used for the debug scene and
/// any future persistence (friends list).
pub fn format_mac(m: &MacAddr) -> heapless::String<17> {
    use core::fmt::Write;
    let mut s: heapless::String<17> = heapless::String::new();
    for (i, b) in m.iter().enumerate() {
        if i > 0 {
            let _ = s.push(':');
        }
        let _ = write!(s, "{:02x}", b);
    }
    s
}
