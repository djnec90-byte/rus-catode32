"""Stretching behavior for comfort and vigor."""

import random
from entities.behaviors.base import BaseBehavior


class StretchingBehavior(BaseBehavior):
    """Pet stretches for comfort and vigor.

    Phases:
    1. preparing - Pet prepares to stretch
    2. stretching - Main stretch
    3. relaxing - Pet relaxes after stretch
    """

    NAME = "stretching"

    COMPLETION_BONUS = {
        "comfort": 3,
    }

    def next(self, context):
        if random.random() < 0.2:
            return 'kneading'
        return None

    def __init__(self, character):
        """Initialize the stretching behavior.

        Args:
            character: The CharacterEntity this behavior belongs to.
        """
        super().__init__(character)

        # Phase durations
        self.prepare_duration = random.uniform(0.5, 2.0)
        self.stretch_duration = random.uniform(3.0, 12.0)
        self.relax_duration = random.uniform(5.0, 15.0)

    def start(self, on_complete=None):
        if self._active:
            return
        super().start(on_complete)
        self._phase = "preparing"
        self._character.set_pose("standing.side.neutral")

    def update(self, dt):
        """Update stretch phases.

        Args:
            dt: Delta time in seconds.
        """
        if not self._active:
            return

        self._phase_timer += dt

        if self._phase == "preparing":
            if self._phase_timer >= self.prepare_duration:
                self._phase = "stretching"
                self._phase_timer = 0.0
                self._character.set_pose("leaning_forward.side.stretch")

        elif self._phase == "stretching":
            self._progress = min(1.0, self._phase_timer / self.stretch_duration)

            if self._phase_timer >= self.stretch_duration:
                self._phase = "relaxing"
                self._phase_timer = 0.0
                self._character.set_pose("standing.side.neutral")

        elif self._phase == "relaxing":
            if self._phase_timer >= self.relax_duration:
                self.stop(completed=True)
