import math
from assets.items import CLOCKFACE

# Hand lengths in pixels from center (clockface center is at pixel 8,8 within the 17x17 sprite)
_HOUR_LEN = 4
_MINUTE_LEN = 6
_CENTER_OX = 8  # offset within sprite to center pixel
_CENTER_OY = 8


class ClockWidget:
    """Draws a wall clock sprite with animated hour/minute hands."""

    def __init__(self, world_x, world_y):
        self.world_x = world_x
        self.world_y = world_y
        self._hour_angle = 0.0
        self._minute_angle = 0.0

    def set_time(self, hours, minutes):
        # hour hand moves with both hours and minutes
        hour_fraction = (hours % 12 + minutes / 60.0) / 12.0
        self._hour_angle = hour_fraction * 2 * math.pi

        # minute hand: full rotation every 60 min
        self._minute_angle = (minutes / 60.0) * 2 * math.pi

    def draw(self, renderer, camera_x, parallax):
        sx = self.world_x - int(camera_x * parallax)
        sy = self.world_y

        renderer.draw_sprite_obj(CLOCKFACE, sx, sy)

        cx = sx + _CENTER_OX
        cy = sy + _CENTER_OY

        _draw_hand(renderer, cx, cy, self._hour_angle, _HOUR_LEN)
        _draw_hand(renderer, cx, cy, self._minute_angle, _MINUTE_LEN)


def _draw_hand(renderer, cx, cy, angle, length):
    """Draw a clock hand from (cx, cy) at clock-angle (0 = 12 o'clock, CW)."""
    ex = cx + int(length * math.sin(angle) + 0.5)
    ey = cy - int(length * math.cos(angle) + 0.5)
    renderer.draw_line(cx, cy, ex, ey)
