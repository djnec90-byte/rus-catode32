"""
sleep_manager.py - Manages device sleep modes for power saving.

Modes (set via config.SLEEP_MODE):
  None    - sleep disabled
  "basic" - screen off, reduced game tick rate, CPU still running full power
  "deep"  - (future) true deep sleep with hardware wake-up via external pull-ups
"""

import time
import machine
import config


class SleepManager:
    """Tracks inactivity and enters the configured sleep mode when the timeout elapses.

    Pin IRQs are registered on every button at init so a button press can
    instantly set the wake flag even while the CPU is in machine.idle().

    Activity tracking (resetting the inactivity timer) is done from the main
    loop via notify_activity() rather than inside the IRQ handler, keeping the
    handler minimal and safe for interrupt context.
    """

    def __init__(self, input_handler, renderer):
        self._input = input_handler
        self._renderer = renderer
        self._sleeping = False
        self._wake_flag = False
        self._last_activity = time.ticks_ms()
        self._register_irqs()

    # ------------------------------------------------------------------
    # IRQ
    # ------------------------------------------------------------------

    def _register_irqs(self):
        """Register a falling-edge IRQ on every button for instant wake detection."""
        for pin in self._input.buttons.values():
            pin.irq(trigger=machine.Pin.IRQ_FALLING, handler=self._on_button_irq)

    def _on_button_irq(self, pin):
        """IRQ handler — runs in interrupt context, must be minimal."""
        self._wake_flag = True

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    @property
    def is_sleeping(self):
        return self._sleeping

    def notify_activity(self):
        """Reset the inactivity timer.  Call from the main loop on any button press."""
        self._last_activity = time.ticks_ms()

    def should_sleep(self):
        """Return True when the inactivity timeout has elapsed and sleep is appropriate."""
        if self._sleeping:
            return False
        elapsed = time.ticks_diff(time.ticks_ms(), self._last_activity)
        return elapsed >= config.SLEEP_TIMEOUT_SEC * 1000

    def enter_sleep(self, update_fn):
        """Block in basic sleep until a button press wakes the device.

        The display must already be off before calling (caller handles the
        transition-out so the screen fade happens in the normal render path).

        update_fn(dt) is called approximately SLEEP_FPS times per second so
        that pet needs, behaviors, and time continue to advance while invisible.
        """
        print("[Sleep] Entering basic sleep")
        self._sleeping = True
        self._wake_flag = False
        self._renderer.power_off()

        last_update = time.ticks_ms()
        while not self._wake_flag:
            machine.idle()
            now = time.ticks_ms()
            elapsed = time.ticks_diff(now, last_update)
            if elapsed >= config.SLEEP_FRAME_TIME_MS:
                update_fn(elapsed / 1000.0)
                last_update = now

        print("[Sleep] Waking from basic sleep")
        self._sleeping = False
        self._wake_flag = False
        self._last_activity = time.ticks_ms()
        self._renderer.power_on()
        # Consume the wake press so it doesn't trigger a game action
        self._input.consume_all()
