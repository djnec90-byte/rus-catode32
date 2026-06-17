# boot.py - Auto-run game unless A+B buttons held during boot
#
# Hold A+B during the 1-second startup window to enter REPL mode.
# The same window also lets mpremote interrupt via Ctrl+C.
#
# After the game exits for any reason, all game modules are cleared
# from sys.modules so that a subsequent `mpremote mount ... exec`
# re-imports clean modules from the mounted filesystem.

from machine import Pin
import time
import gc

# Button pins (must match config.py)
BTN_A = 1
BTN_B = 0

btn_a = Pin(BTN_A, Pin.IN, Pin.PULL_UP)
btn_b = Pin(BTN_B, Pin.IN, Pin.PULL_UP)

# Wait 0.25 seconds, then sample A+B once at the end.
# Sampling at the END (not during) avoids false positives from GPIO
# boot transients on GPIO 0/1 that settle within the first ~100ms.
# The sleep is interruptible by Ctrl+C so mpremote can still break in.
time.sleep_ms(250)
_skip = btn_a.value() == 0 and btn_b.value() == 0

if _skip:
    print("[boot] A+B held - REPL mode")
else:
    print("[boot] Starting game...")
    try:
        import main
        main.main()
    except Exception as e:
        import sys
        print("[boot] Error:")
        sys.print_exception(e)
    finally:
        # Clear all game modules from sys.modules.
        # This ensures that a subsequent `mpremote mount <dir> exec "import main; main.main()"`
        # re-imports everything fresh from the mounted filesystem rather than
        # getting the stale cached versions from device flash.
        import sys
        _keep = frozenset(('micropython', 'gc', 'sys', 'machine', 'time', 'builtins', 'uos'))
        for _k in list(sys.modules):
            if _k not in _keep:
                try:
                    del sys.modules[_k]
                except Exception:
                    pass
        gc.collect()
        print("[boot] Module cache cleared")
