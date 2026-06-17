"""visitor_cat.py - Lightweight remote cat entity driven by ESP-NOW state updates.

No behavior manager, no context. Pose/position/mirror are set externally via
apply_state() whenever a 'vst' message arrives from the peer device.
Animation counters run locally so the sprite doesn't freeze between updates.
"""

from entities.entity import Entity


class VisitorCatEntity(Entity):

    def __init__(self, x, y):
        super().__init__(x, y)
        self.mirror = False
        self.vx = 0.0            # pixels/second received from peer; used to extrapolate between vst packets
        self.pose_name = 'sitting.side.neutral'
        self._pose = None
        self._anim_body = 0.0
        self._anim_head = 0.0
        self._anim_eyes = 0.0
        self._anim_tail = 0.0
        self._mirror_cache = {}
        self._inv_fill_cache = {}
        self._load_pose(self.pose_name)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def apply_state(self, x, pose_name, mirror, vx=0):
        """Update position, pose, facing direction, and velocity from a received vst packet."""
        self.x = x
        self.vx = vx
        self.mirror = bool(mirror)
        if pose_name != self.pose_name:
            self._load_pose(pose_name)

    # ------------------------------------------------------------------
    # Entity overrides
    # ------------------------------------------------------------------

    def update(self, dt):
        # Extrapolate position between network updates using last-known velocity
        if self.vx != 0:
            self.x += self.vx * dt
        if self._pose is None:
            return
        p = self._pose
        self._anim_body = (self._anim_body + dt * p['body'].get('speed', 1)) % self._total_frames(p['body'])
        self._anim_head = (self._anim_head + dt * p['head'].get('speed', 1)) % self._total_frames(p['head'])
        self._anim_eyes = (self._anim_eyes + dt * p['eyes'].get('speed', 1)) % self._total_frames(p['eyes'])
        self._anim_tail = (self._anim_tail + dt * p['tail'].get('speed', 1)) % self._total_frames(p['tail'])

    def draw(self, renderer, camera_offset=0):
        if not self.visible or self._pose is None:
            return

        p = self._pose
        x = int(self.x) - camera_offset
        y = int(self.y)

        body = p['body']
        bf = self._frame_idx(body, self._anim_body)
        bx = x - self._anchor_x(body)
        by = y - body['anchor_y']

        head = p['head']
        hf = self._frame_idx(head, self._anim_head)
        hx = bx + self._point(body, 'head_x', bf) - self._anchor_x(head)
        hy = by + self._point(body, 'head_y', bf) - head['anchor_y']

        eyes = p['eyes']
        ef = self._frame_idx(eyes, self._anim_eyes)
        ex = hx + self._point(head, 'eye_x', hf) - self._anchor_x(eyes)
        ey = hy + self._point(head, 'eye_y', hf) - eyes['anchor_y']

        tail = p['tail']
        tf = self._frame_idx(tail, self._anim_tail)
        tx = bx + self._point(body, 'tail_x', bf) - self._anchor_x(tail)
        ty = by + self._point(body, 'tail_y', bf) - tail['anchor_y']

        self._draw_part(renderer, tail, tx, ty, tf)
        if p.get('head_first'):
            self._draw_part(renderer, head, hx, hy, hf)
            self._draw_part(renderer, body, bx, by, bf)
        else:
            self._draw_part(renderer, body, bx, by, bf)
            self._draw_part(renderer, head, hx, hy, hf)
        self._draw_part(renderer, eyes, ex, ey, ef)

    # ------------------------------------------------------------------
    # Internal helpers (mirrors CharacterEntity sprite logic)
    # ------------------------------------------------------------------

    def _load_pose(self, pose_name):
        from assets.character import POSES
        parts = pose_name.split('.')
        try:
            self._pose = POSES[parts[0]][parts[1]][parts[2]]
            self.pose_name = pose_name
            self._mirror_cache = {}
            self._inv_fill_cache = {}
        except (KeyError, IndexError):
            print('[VisitorCat] Unknown pose: ' + pose_name)

    def _total_frames(self, sprite):
        return len(sprite['frames']) + sprite.get('extra_frames', 0)

    def _frame_idx(self, sprite, counter):
        n = len(sprite['frames'])
        total = n + sprite.get('extra_frames', 0)
        i = int(counter) % total
        return i if i < n else 0

    def _anchor_x(self, sprite):
        ax = sprite['anchor_x']
        return sprite['width'] - ax if self.mirror else ax

    def _point(self, sprite, key, frame):
        v = sprite[key]
        result = v[frame] if isinstance(v, list) else v
        if self.mirror and key.endswith('_x'):
            return sprite['width'] - result
        return result

    def _ensure_mirrored(self, sprite):
        from sprite_transform import mirror_sprite_h
        sid = id(sprite)
        if sid not in self._mirror_cache:
            w, h = sprite['width'], sprite['height']
            entry = {'frames': [mirror_sprite_h(f, w, h) for f in sprite['frames']]}
            if 'fill_frames' in sprite:
                mf = [mirror_sprite_h(f, w, h) for f in sprite['fill_frames']]
                entry['inv_fill_frames'] = [bytearray(b ^ 0xFF for b in f) for f in mf]
            self._mirror_cache[sid] = entry
        return self._mirror_cache[sid]

    def _ensure_inv_fill(self, sprite):
        sid = id(sprite)
        if sid not in self._inv_fill_cache:
            self._inv_fill_cache[sid] = [bytearray(b ^ 0xFF for b in f) for f in sprite['fill_frames']]
        return self._inv_fill_cache[sid]

    def _draw_part(self, renderer, sprite, x, y, frame):
        if self.mirror:
            cached = self._ensure_mirrored(sprite)
            if 'inv_fill_frames' in cached:
                renderer.draw_sprite(cached['inv_fill_frames'][frame], sprite['width'], sprite['height'],
                                     x, y, transparent=True, transparent_color=1)
            renderer.draw_sprite(cached['frames'][frame], sprite['width'], sprite['height'], x, y)
        else:
            if 'fill_frames' in sprite:
                inv = self._ensure_inv_fill(sprite)
                renderer.draw_sprite(inv[frame], sprite['width'], sprite['height'],
                                     x, y, transparent=True, transparent_color=1)
            renderer.draw_sprite(sprite['frames'][frame], sprite['width'], sprite['height'], x, y)
