import config
from scenes.vacation_scene import VacationScene
from environment import Environment, LAYER_BACKGROUND, LAYER_MIDGROUND, LAYER_FOREGROUND
from entities.character import CharacterEntity
from sky import SkyRenderer
from assets.nature import (BEACH_HILL, BEACH_MOUNTAIN, BEACH_TREES,
                            WAVE_CHUNKS, WAVE_CHUNKS2,
                            BEACH_WAVE_BG, BEACH_WAVE_MG_SM, BEACH_WAVE_MG_LG, BEACH_WAVE_FG,
                            BEACH_SHIMMERS_BG, BEACH_SHIMMERS_MG)

_WORLD_WIDTH = 234
_GROUND_Y    = 58

# Sprite dimensions
_HILL_W  = BEACH_HILL["width"]       # 36
_HILL_H  = BEACH_HILL["height"]      # 4
_MTN_W   = BEACH_MOUNTAIN["width"]
_MTN_H   = BEACH_MOUNTAIN["height"]
_TREE_W  = BEACH_TREES["width"]      # 16
_TREE_H  = BEACH_TREES["height"]     # 11
_WC_W    = WAVE_CHUNKS["width"]      # 8
_WC2_W   = WAVE_CHUNKS2["width"]     # 4

# World-x of the BEACH_HILL sprite (background parallax layer)
_HILL_WORLD_X = 54
_HILL_Y       = 38   # top of hill sprite

# Horizon lines
_WATER_HORIZON_Y = _HILL_Y + _HILL_H   # 1px below hill bottom
_SAND_HORIZON_Y  = _HILL_Y - 1         # 1px above hill top

# BEACH_MOUNTAIN on the background parallax layer
_MTN_WORLD_X = config.DISPLAY_WIDTH - _MTN_W + 31
_MTN_Y       = _HILL_Y - _MTN_H - 1

# BEACH_TREES on the background parallax layer, overlapping the hill
_TREES_WORLD_X = _HILL_WORLD_X + 14
_TREES_Y       = _HILL_Y - _TREE_H + 1

# Shore world-x: same as the hill left edge so lines are vertical at camera_x=0
_SHORE_WORLD_X = _HILL_WORLD_X

# Sand texture — small pixel clusters in world coordinates (packed x,y bytes).
_SAND_MG_CLUSTERS = bytes([
     98, 44, 101, 46,
    115, 41, 118, 43, 120, 41,
    128, 48, 131, 50,
    140, 43, 143, 45,
    152, 50, 155, 48, 158, 51,
    163, 45, 167, 40,
    174, 39, 177, 42, 180, 40,
    185, 47, 180, 43,
])

_SAND_FG_CLUSTERS = bytes([
     80, 55,  88, 57,
     99, 60, 102, 62, 106, 60,
    118, 53, 121, 55,
    135, 58, 139, 60,
    146, 62, 150, 60, 153, 63,
    158, 55, 162, 57, 165, 55,
    168, 60, 172, 58,
    199, 54, 201, 56,
    210, 58, 215, 60,
])

# Shared animation timer — all waves and shimmers use this single period.
_TIMER_LENGTH           = 3.0
_WAVE_CREST_DURATION    = 0.6   # seconds to hold at shore
_WAVE_RECEDE_DURATION   = 1.2   # seconds to drift back to water
_WAVE_INCOMING_DURATION = _TIMER_LENGTH - _WAVE_CREST_DURATION - _WAVE_RECEDE_DURATION

_SHIMMER_STEP_DURATION = 0.18   # seconds per pixel of drift

# Pixels each wave group travels from water to shoreline.
_WAVE_TRAVEL_BG    = 4
_WAVE_TRAVEL_MG_SM = 6
_WAVE_TRAVEL_MG_LG = 8
_WAVE_TRAVEL_FG    = 12

# Scale factors for unpacking phase and shore-parallax from bytes.
_TIMER_SCALE    = _TIMER_LENGTH / 256   # phase_256 → seconds offset
_SHORE_PAR_SCALE = 1.0 / 128           # shore_par_128 → parallax factor


def _draw_wave_group(renderer, camera_x, timer, chunks, sprite, sprite_w, num_frames, travel_px):
    half_w = sprite_w // 2
    last_frame = num_frames - 1
    frame_interval = _WAVE_INCOMING_DURATION / num_frames
    for i in range(0, len(chunks), 3):
        t = (timer + chunks[i + 1] * _TIMER_SCALE) % _TIMER_LENGTH
        shore_x = int(_SHORE_WORLD_X - camera_x * chunks[i + 2] * _SHORE_PAR_SCALE) - half_w
        if t < _WAVE_INCOMING_DURATION:
            frame = int(t / frame_interval)
            if frame > last_frame:
                frame = last_frame
            travel = travel_px * frame // last_frame
        elif t < _WAVE_INCOMING_DURATION + _WAVE_CREST_DURATION:
            frame = last_frame
            travel = travel_px
        else:
            frame = last_frame
            recede_elapsed = t - (_WAVE_INCOMING_DURATION + _WAVE_CREST_DURATION)
            progress = max(0.0, 1.0 - recede_elapsed / _WAVE_RECEDE_DURATION)
            travel = int(travel_px * progress)
        renderer.draw_sprite_obj(sprite, shore_x - (travel_px - travel), chunks[i], frame=frame)


def _draw_shimmers(renderer, camera_x, parallax, timer, shimmers):
    sw = config.DISPLAY_WIDTH
    offset = int(camera_x * parallax)
    for i in range(0, len(shimmers), 5):
        max_drift = shimmers[i + 4]
        dark_dur = _TIMER_LENGTH - max_drift * _SHIMMER_STEP_DURATION
        t = (timer + shimmers[i + 2] * _TIMER_SCALE) % _TIMER_LENGTH
        if t < dark_dur:
            continue
        step = int((t - dark_dur) / _SHIMMER_STEP_DURATION)
        if step >= max_drift:
            continue
        direction = 1 if shimmers[i + 3] else -1
        sx = shimmers[i] + step * direction - offset
        if 0 <= sx < sw:
            renderer.draw_pixel(sx, shimmers[i + 1])


class VacationBeachScene(VacationScene):
    SCENE_NAME     = 'vacation_beach'
    ENTRY_X        = 110
    ENJOY_DURATION = 750.0
    GRACE_DURATION = 120.0
    STAT_ACCRUAL   = {'serenity': 8.0, 'fulfillment': 8.0}

    def __init__(self, context, renderer, input):
        super().__init__(context, renderer, input)
        self.sky = SkyRenderer()
        self._wave_timer = 0.0

    def setup_scene(self):
        self.environment = Environment(world_width=_WORLD_WIDTH)
        self.context.scene_x_min = 100
        self.context.scene_x_max = _WORLD_WIDTH - 10
        self.character = CharacterEntity(self.ENTRY_X, _GROUND_Y, context=self.context)

    def on_enter(self):
        env_settings = getattr(self.context, 'environment', {})
        sky_settings = dict(env_settings)
        sky_settings['weather'] = 'Clear'
        self.sky.configure(
            sky_settings,
            world_width=self.environment.world_width,
            seed=self.context.pet_seed
        )
        self.sky._render_rect = (0, 0, config.DISPLAY_WIDTH, _SAND_HORIZON_Y - 1)
        self.sky.add_to_environment(self.environment, LAYER_BACKGROUND)
        self.environment.add_custom_draw(LAYER_BACKGROUND, self._draw_background)
        self.environment.add_custom_draw(LAYER_MIDGROUND, self._draw_midground)
        self.environment.add_custom_draw(LAYER_FOREGROUND, self._draw_foreground)

    def on_exit(self):
        self.sky.remove_from_environment(self.environment, LAYER_BACKGROUND)

    def on_update(self, dt):
        env = self.context.environment
        self.sky.set_time(env.get('time_hours', 12), env.get('time_minutes', 0))
        self.sky.update(dt)
        self.character.update(dt)
        self.environment.update(dt)
        self._wave_timer += dt
        if self._wave_timer >= _TIMER_LENGTH:
            self._wave_timer -= _TIMER_LENGTH
        self._tick_vacation(dt)

    # ------------------------------------------------------------------
    # Draw passes
    # ------------------------------------------------------------------

    def _draw_background(self, renderer, camera_x, parallax):
        offset = int(camera_x * parallax)
        sw = config.DISPLAY_WIDTH
        hill_sx = _HILL_WORLD_X - offset

        # Water horizon: left edge to left side of hill
        water_right = max(0, min(sw, hill_sx))
        if water_right > 0:
            renderer.draw_line(0, _WATER_HORIZON_Y, water_right - 1, _WATER_HORIZON_Y, color=1)

        # Sand horizon: right side of hill to right edge
        sand_left = max(0, min(sw, hill_sx + _HILL_W))
        if sand_left < sw:
            renderer.draw_line(sand_left, _SAND_HORIZON_Y, sw - 1, _SAND_HORIZON_Y, color=1)

        # Hill, mountain, trees
        if hill_sx + _HILL_W >= 0 and hill_sx < sw:
            renderer.draw_sprite_obj(BEACH_HILL, hill_sx, _HILL_Y)

        mtn_sx = _MTN_WORLD_X - offset
        if mtn_sx + _MTN_W >= 0 and mtn_sx < sw:
            renderer.draw_sprite_obj(BEACH_MOUNTAIN, mtn_sx, _MTN_Y)

        tree_sx = _TREES_WORLD_X - offset
        if tree_sx + _TREE_W >= 0 and tree_sx < sw:
            renderer.draw_sprite_obj(BEACH_TREES, tree_sx, _TREES_Y)

        _draw_shimmers(renderer, camera_x, parallax, self._wave_timer, BEACH_SHIMMERS_BG)
        _draw_wave_group(renderer, camera_x, self._wave_timer, BEACH_WAVE_BG,
                         WAVE_CHUNKS2, _WC2_W, 4, _WAVE_TRAVEL_BG)

    def _draw_midground(self, renderer, camera_x, parallax):
        offset = int(camera_x * parallax)
        sw = config.DISPLAY_WIDTH
        b = _SAND_MG_CLUSTERS
        for i in range(0, len(b), 2):
            sx = b[i] - offset
            if 0 <= sx < sw:
                renderer.draw_pixel(sx, b[i + 1])

        _draw_shimmers(renderer, camera_x, parallax, self._wave_timer, BEACH_SHIMMERS_MG)
        _draw_wave_group(renderer, camera_x, self._wave_timer, BEACH_WAVE_MG_SM,
                         WAVE_CHUNKS2, _WC2_W, 4, _WAVE_TRAVEL_MG_SM)
        _draw_wave_group(renderer, camera_x, self._wave_timer, BEACH_WAVE_MG_LG,
                         WAVE_CHUNKS,  _WC_W,  8, _WAVE_TRAVEL_MG_LG)

    def _draw_foreground(self, renderer, camera_x, parallax):
        offset = int(camera_x * parallax)
        sw = config.DISPLAY_WIDTH
        b = _SAND_FG_CLUSTERS
        for i in range(0, len(b), 2):
            sx = b[i] - offset
            if 0 <= sx < sw:
                renderer.draw_pixel(sx, b[i + 1])

        _draw_wave_group(renderer, camera_x, self._wave_timer, BEACH_WAVE_FG,
                         WAVE_CHUNKS, _WC_W, 8, _WAVE_TRAVEL_FG)
