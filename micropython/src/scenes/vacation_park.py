import random
import config
from scenes.vacation_scene import VacationScene
from environment import Environment, LAYER_BACKGROUND, LAYER_MIDGROUND, LAYER_FOREGROUND
from entities.character import CharacterEntity
from entities.flyer import FlyerEntity
from sky import SkyRenderer
from assets.furniture import PARK_BENCH, STREET_LAMP
from assets.nature import BUSH
from assets.plants import TINY_FLOWER, GRASS_YOUNG, GRASS_GROWING

_WORLD_WIDTH = 300
_GROUND_Y    = 63
_PATH_Y_TOP  = 44
_PATH_Y_BOT  = 48

_LAMP_POSITIONS  = [20, 90, 160]
_BENCH_POSITIONS = [55, 125]

# (world_x, y_bottom) — bottoms 2-3px above _PATH_Y_TOP (44), drawn before benches/lamps
_BUSH_POSITIONS = [
    (8,  42),
    (42,  41),
    (88, 42),
    (120, 41),
]

# Predefined grass blade positions along the top edge of the path (y = _PATH_Y_TOP - 1)
_PATH_GRASS_X = [5, 12, 28, 33, 51, 67, 72, 89, 98, 119, 138, 142, 161]

# (world_x, sprite, y_bottom)
_SCATTER_FOREGROUND = [
    (15,  GRASS_YOUNG,    63),
    (55,  GRASS_GROWING,  63),
    (105, GRASS_YOUNG,    63),
    (140, GRASS_GROWING,  63),
    (185, GRASS_YOUNG,    63),
    (220, GRASS_GROWING,  63),
    (255, GRASS_YOUNG,    63),
    (285, GRASS_GROWING,  63),
]

_SCATTER_MIDGROUND = [
    (42,  TINY_FLOWER,  58),
    (90,  TINY_FLOWER,  55),
    (125, GRASS_YOUNG,  60),
    (165, TINY_FLOWER,  57),
    (175, TINY_FLOWER,  53),
    (230, TINY_FLOWER,  59),
    (240, GRASS_YOUNG,  56),
    (275, TINY_FLOWER,  54),
]


class VacationParkScene(VacationScene):
    SCENE_NAME     = 'vacation_park'
    ENJOY_DURATION = 750.0   # ~12.5 minutes
    GRACE_DURATION = 120.0
    STAT_ACCRUAL   = {'fulfillment': 8.0, 'playfulness': 8.0}

    def __init__(self, context, renderer, input):
        super().__init__(context, renderer, input)
        self.sky = SkyRenderer()

    def setup_scene(self):
        self.environment = Environment(world_width=_WORLD_WIDTH)
        self.context.scene_x_min = 10
        self.context.scene_x_max = _WORLD_WIDTH - 10
        self.character = CharacterEntity(self.ENTRY_X, _GROUND_Y, context=self.context)

        for x, y_bot in _BUSH_POSITIONS:
            self.environment.add_object(
                LAYER_BACKGROUND, BUSH,
                x, y_bot - BUSH["height"]
            )
        for x in _BENCH_POSITIONS:
            self.environment.add_object(
                LAYER_BACKGROUND, PARK_BENCH,
                x, _PATH_Y_TOP - PARK_BENCH["height"]
            )
        for x in _LAMP_POSITIONS:
            self.environment.add_object(
                LAYER_BACKGROUND, STREET_LAMP,
                x, _PATH_Y_TOP - STREET_LAMP["height"]
            )

        for _ in range(random.randint(2, 3)):
            b = FlyerEntity(
                'butterfly',
                random.randint(20, _WORLD_WIDTH - 20),
                random.randint(5, 35),
            )
            b.anim_speed   = random.randint(7, 12)
            b.bounds_left  = 10
            b.bounds_right = _WORLD_WIDTH - 10
            b.bounds_top   = 5
            b.bounds_bottom = _GROUND_Y - 5
            self.environment.add_entity(b)

    def on_enter(self):
        env_settings = getattr(self.context, 'environment', {})
        sky_settings = dict(env_settings)
        sky_settings['weather'] = 'Clear'
        self.sky.configure(
            sky_settings,
            world_width=self.environment.world_width,
            seed=self.context.pet_seed
        )
        self.sky.add_to_environment(self.environment, LAYER_BACKGROUND)
        self.environment.add_custom_draw(LAYER_BACKGROUND, self._draw_path)
        self.environment.add_custom_draw(LAYER_MIDGROUND, self._draw_scatter_midground)
        self.environment.add_custom_draw(LAYER_FOREGROUND, self._draw_scatter_foreground)

    def on_exit(self):
        self.sky.remove_from_environment(self.environment, LAYER_BACKGROUND)

    def on_update(self, dt):
        env = self.context.environment
        self.sky.set_time(env.get('time_hours', 12), env.get('time_minutes', 0))
        self.sky.update(dt)
        self.character.update(dt)
        self.environment.update(dt)
        self._tick_vacation(dt)

    def _draw_path(self, renderer, camera_x, parallax):
        camera_offset = int(camera_x * parallax)

        # Occasional grass blades peeking above the top path line
        for wx in _PATH_GRASS_X:
            sx = wx - camera_offset
            if 0 <= sx < config.DISPLAY_WIDTH:
                renderer.draw_pixel(sx, _PATH_Y_TOP - 1)

        for sx in range(config.DISPLAY_WIDTH):
            wx = sx + camera_offset
            # Slow component controls gap zones; fast component feathers edges.
            # Top line
            slow_t = (wx * 3) % 53
            fast_t = (wx * 11) % 7
            if slow_t < 11:          # gap zone: sparse feather pixels
                if fast_t == 0:
                    renderer.draw_pixel(sx, _PATH_Y_TOP)
            else:                    # solid zone: mostly draw, occasional skip
                if fast_t != 2:
                    renderer.draw_pixel(sx, _PATH_Y_TOP)
            # Bottom line — different constants so the two lines look independent
            slow_b = (wx * 5) % 61
            fast_b = (wx * 9) % 7
            if slow_b < 13:
                if fast_b == 0:
                    renderer.draw_pixel(sx, _PATH_Y_BOT)
            else:
                if fast_b != 3:
                    renderer.draw_pixel(sx, _PATH_Y_BOT)

    def _draw_scatter_foreground(self, renderer, camera_x, parallax):
        self._draw_scatter(renderer, camera_x, parallax, _SCATTER_FOREGROUND)

    def _draw_scatter_midground(self, renderer, camera_x, parallax):
        self._draw_scatter(renderer, camera_x, parallax, _SCATTER_MIDGROUND)

    def _draw_scatter(self, renderer, camera_x, parallax, items):
        camera_offset = int(camera_x * parallax)
        screen_width = config.DISPLAY_WIDTH
        for world_x, sprite, y_bottom in items:
            sx = world_x - camera_offset
            if sx < -sprite["width"] or sx > screen_width + sprite["width"]:
                continue
            renderer.draw_sprite(
                sprite["frames"][0],
                sprite["width"],
                sprite["height"],
                sx, y_bottom - sprite["height"],
                transparent=True,
                transparent_color=0
            )
