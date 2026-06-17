"""Kneading biscuits behavior - rhythmic pawing when content after a stretch."""

import random
from entities.behaviors.base import BaseBehavior


class KneadingBehavior(BaseBehavior):
    """Pet kneads rhythmically after a satisfying stretch.

    A calm, self-soothing behavior. Chains back to stretching,
    which has a 50% chance of returning here — creating a short
    but naturally-terminating comfort loop.

    Phases:
    1. kneading - Rhythmic pawing
    2. settling  - Pet winds down and stills
    """

    NAME = "kneading"

    COMPLETION_BONUS = {
        # Rapid changers
        "comfort": 2,
        "focus": -0.25,

        # Medium changers
        "cleanliness": -0.1,

        # Slow changers
        "serenity": 0.05,
    }

    def __init__(self, character):
        super().__init__(character)

        self.knead_duration = random.uniform(10.0, 45.0)
        self.settle_duration = random.uniform(1.0, 4.0)

    def get_completion_bonus(self, context):
        bonus = dict(super().get_completion_bonus(context))
        fed_factor = max(0.0, (context.fullness - 90) / 10.0)
        if fed_factor > 0:
            bonus["comfort"] = bonus.get("comfort", 0) + 1 * fed_factor
            bonus["serenity"] = bonus.get("serenity", 0) + 0.1 * fed_factor
        return self.apply_location_bonus(context, bonus)

    def apply_location_bonus(self, context, bonus):
        if getattr(context, 'in_familiar_location', False):
            bonus['comfort'] = bonus.get('comfort', 0) + 1
            bonus['serenity'] = bonus.get('serenity', 0) + 0.15  # deep comfort in known territory
        else:
            bonus['comfort'] = bonus.get('comfort', 0) * 0.85    # can't fully settle elsewhere
        return bonus

    def next(self, context):
        if random.random() < 0.5:
            return 'stretching'
        return 'lounging'

    def start(self, on_complete=None):
        if self._active:
            return
        super().start(on_complete)
        self._phase = "kneading"
        self._character.set_pose("kneading.side.neutral")

    def update(self, dt):
        if not self._active:
            return

        self._phase_timer += dt

        if self._phase == "kneading":
            self._progress = min(1.0, self._phase_timer / self.knead_duration)

            if self._phase_timer >= self.knead_duration:
                self._phase = "settling"
                self._phase_timer = 0.0
                self._character.set_pose("leaning_forward.side.neutral")

        elif self._phase == "settling":
            if self._phase_timer >= self.settle_duration:
                self.stop(completed=True)
