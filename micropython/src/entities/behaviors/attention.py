"""Attention behavior for psst and point_bird interactions."""

import random
from entities.behaviors.base import BaseBehavior
from assets.icons import EXCLAIM
from ui import draw_bubble

_REJECTION_DURATION = 5.0
_REJECTION_STAT_MULTIPLIER = 0.5


_PHASE1_DURATION = 1.5  # question mark
_PHASE2_DURATION = 1.5  # exclamation mark
_PHASE3_DURATION = 2.0  # happy

_EXCLAIM_RISE_DURATION = 1.0  # seconds for exclaim to rise
_EXCLAIM_RISE_AMOUNT = 15     # pixels risen at peak


# Variant configurations
VARIANTS = {
    "psst": {
        "stats": {
            "curiosity": 3,
            "playfulness": 1.5,
            "focus": 3,
            "courage": 0.05,
            "intelligence": 0.5,
        },
    },
    "point_bird": {
        "stats": {
            "curiosity": 5,
            "playfulness": 2.5,
            "focus": 2,
            "courage": 0.05,
            "intelligence": 0.5,
        },
    },
}


class AttentionBehavior(BaseBehavior):
    """Handles psst and point_bird interactions.

    Phases:
    1. noticing  - Question mark bubble, sitting_silly.side.neutral
    2. realizing - Exclamation mark rises, sitting_silly.side.aloof
    3. happy     - No bubble, sitting_silly.side.happy

    Rejection (psst only):
    - Cat holds a disinterested pose for ~5s, half stat rewards, then meanders.
    """

    NAME = "attention"

    REJECTION_POSES = (
        "standing.side.neutral_looking_down",
        "sitting.side.looking_down",
        "laying.side.neutral2",
        "laying.side.bored",
        "sitting_silly.side.neutral",
        "standing.side.annoyed",
        "laying.side.annoyed",
        "laying.side.content",
        "sitting_licking.side.licking_leg",
    )

    _REJECTION_THRESHOLDS = {
        "affection":   25,
        "comfort":     30,
        "sociability": 25,
        "courage":     20,
    }

    @classmethod
    def _rejection_chance(cls, context):
        complement = 1.0
        for stat, threshold in cls._REJECTION_THRESHOLDS.items():
            val = getattr(context, stat, 100)
            if val < threshold:
                deficit = (threshold - val) / threshold
                complement *= (1.0 - deficit)
        return 1.0 - complement

    def __init__(self, character):
        super().__init__(character)
        self._variant = "psst"
        self._rejecting = False

    def get_completion_bonus(self, context):
        bonus = dict(VARIANTS[self._variant].get("stats", {}))
        if self._rejecting:
            bonus = {k: v * _REJECTION_STAT_MULTIPLIER for k, v in bonus.items()}
        return bonus

    def next(self, context):
        if self._rejecting:
            return 'meandering'
        if self._variant == "point_bird" and context:
            chance = 0.25 * ((context.playfulness + context.curiosity) / 100)
            if random.random() < chance:
                return 'chattering'
        return None

    def start(self, variant=None, on_complete=None):
        if self._active:
            return
        self._variant = variant if variant in VARIANTS else "psst"
        super().start(on_complete)

        context = self._character.context
        if context and self._variant == "psst":
            self._rejecting = random.random() < self._rejection_chance(context)
        else:
            self._rejecting = False

        if self._rejecting:
            self._phase = "rejecting"
            self._character.set_pose(random.choice(self.REJECTION_POSES))
            return

        self._phase = "noticing"
        self._character.set_pose("sitting_silly.side.neutral")

    def update(self, dt):
        """Update the reaction.

        Args:
            dt: Delta time in seconds.
        """
        if not self._active:
            return

        self._phase_timer += dt

        if self._phase == "rejecting":
            if self._phase_timer >= _REJECTION_DURATION:
                self.stop(completed=True)
            return

        if self._phase == "noticing":
            self._progress = min(1.0, self._phase_timer / _PHASE1_DURATION)
            if self._phase_timer >= _PHASE1_DURATION:
                self._phase = "realizing"
                self._phase_timer = 0.0
                self._progress = 0.0
                self._character.set_pose("sitting_silly.side.aloof")

        elif self._phase == "realizing":
            self._progress = min(1.0, self._phase_timer / _PHASE2_DURATION)
            if self._phase_timer >= _PHASE2_DURATION:
                self._phase = "happy"
                self._phase_timer = 0.0
                self._character.set_pose("sitting_silly.side.happy")

        elif self._phase == "happy":
            if self._phase_timer >= _PHASE3_DURATION:
                self._character.play_bursts()
                self.stop(completed=True)

    def draw(self, renderer, char_x, char_y, mirror=False):
        """Draw the speech bubble or exclamation mark.

        Args:
            renderer: The renderer to draw with.
            char_x: Character's x position on screen.
            char_y: Character's y position.
            mirror: If True, character is facing right.
        """
        if not self._active:
            return

        if self._phase == "noticing":
            draw_bubble(renderer, "question", char_x, char_y, self._progress, mirror)

        elif self._phase == "realizing":
            rise_t = min(1.0, self._phase_timer / _EXCLAIM_RISE_DURATION)
            rise_offset = int(rise_t * _EXCLAIM_RISE_AMOUNT)
            exclaim_y = char_y - 40 - rise_offset

            if mirror:
                exclaim_x = char_x + 16
            else:
                exclaim_x = char_x - EXCLAIM["width"] - 16

            renderer.draw_sprite_obj(EXCLAIM, exclaim_x, exclaim_y)
