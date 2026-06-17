"""Generate Rust source for character sprites and poses from assets.character.

Run from the repo root:
    python3 micropython/tools/gen_character.py > src/assets/character.rs

Re-run only when micropython/src/assets/character.py changes. The output is
committed.
"""

import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_SRC = os.path.normpath(os.path.join(_HERE, "..", "src"))
sys.path.insert(0, _SRC)

from assets import character as cm


def rust_byte_literal(data):
    return 'b"' + "".join("\\x%02x" % b for b in data) + '"'


def part_kind(name):
    """Return ('body'|'head'|'eyes'|'tail') for a CHAR_X identifier."""
    rest = name[len("CHAR_"):]
    for kind, prefix in (
        ("body", "BODY_"),
        ("head", "HEAD_"),
        ("eyes", "EYES_"),
        ("tail", "TAIL_"),
    ):
        if rest.startswith(prefix):
            return kind
    raise ValueError("Unknown CHAR_ prefix: " + name)


def short_name(char_name):
    """CHAR_BODY_SIDE_SITTING -> BODY_SIDE_SITTING."""
    return char_name[len("CHAR_"):]


def emit_sprite_blocks(out, name, sprite):
    width = sprite["width"]
    height = sprite["height"]
    frames = sprite["frames"]
    fill_frames = sprite.get("fill_frames")
    speed = float(sprite.get("speed", 1))
    extra_frames = int(sprite.get("extra_frames", 0))

    short = short_name(name)
    frames_const = short + "_FRAMES"
    out.append("const " + frames_const + ": &[&[u8]] = &[")
    for fr in frames:
        out.append("    " + rust_byte_literal(fr) + ",")
    out.append("];\n")

    if fill_frames is not None:
        fill_const = short + "_FILL_FRAMES"
        out.append("const " + fill_const + ": &[&[u8]] = &[")
        for fr in fill_frames:
            out.append("    " + rust_byte_literal(fr) + ",")
        out.append("];\n")
        fill_expr = "Some(" + fill_const + ")"
    else:
        fill_expr = "None"

    kind = part_kind(name)
    sprite_block = (
        "    sprite: Sprite {\n"
        "        width: " + str(width) + ",\n"
        "        height: " + str(height) + ",\n"
        "        frames: " + frames_const + ",\n"
        "        fill_frames: " + fill_expr + ",\n"
        "    },\n"
        "    anchor_x: " + str(sprite["anchor_x"]) + ",\n"
        "    anchor_y: " + str(sprite["anchor_y"]) + ",\n"
    )

    if kind == "body":
        type_name = "CharBody"
        sprite_block += (
            "    head_x: " + str(sprite["head_x"]) + ",\n"
            "    head_y: " + str(sprite["head_y"]) + ",\n"
            "    tail_x: " + str(sprite["tail_x"]) + ",\n"
            "    tail_y: " + str(sprite["tail_y"]) + ",\n"
        )
    elif kind == "head":
        type_name = "CharHead"
        sprite_block += (
            "    eye_x: " + str(sprite.get("eye_x", 0)) + ",\n"
            "    eye_y: " + str(sprite.get("eye_y", 0)) + ",\n"
        )
    else:
        type_name = "CharPart"

    sprite_block += (
        "    speed: " + repr(speed) + ",\n"
        "    extra_frames: " + str(extra_frames) + ",\n"
    )

    out.append("pub static " + short + ": " + type_name + " = " + type_name + " {")
    out.append(sprite_block.rstrip())
    out.append("};\n")


def pascal_case(name):
    """sitting_silly.side.happy -> SittingSillySideHappy."""
    parts = name.replace(".", "_").split("_")
    return "".join(p[:1].upper() + p[1:] for p in parts if p)


def upper_name(name):
    return name.upper().replace(".", "_")


def walk_poses(poses):
    """Yield (dotted_name, leaf_dict) for every leaf in POSES."""
    for position, dirs in poses.items():
        for direction, emotions in dirs.items():
            for emotion, leaf in emotions.items():
                yield (position + "." + direction + "." + emotion, leaf)


def emit_pose(out, dotted, leaf):
    variant = pascal_case(dotted)
    const_name = "POSE_" + upper_name(dotted)
    body_short = short_name(leaf["body"].__name__) if False else None  # placeholder
    # The leaf values are sprite dicts; we need to look them up by identity in CHAR_X module names.
    raise NotImplementedError


def emit_pose_constants(out, ident_to_name, poses):
    pose_entries = []
    for dotted, leaf in walk_poses(poses):
        variant = pascal_case(dotted)
        const_name = "POSE_" + upper_name(dotted)
        body_short = ident_to_name[id(leaf["body"])]
        head_short = ident_to_name[id(leaf["head"])]
        tail_short = ident_to_name[id(leaf["tail"])]
        eyes_dict = leaf.get("eyes")
        if eyes_dict is not None:
            eyes_short = ident_to_name[id(eyes_dict)]
            eyes_expr = "Some(&" + eyes_short + ")"
        else:
            eyes_expr = "None"
        head_first = bool(leaf.get("head_first", False))
        tail_last = bool(leaf.get("tail_last", False))

        out.append("pub static " + const_name + ": Pose = Pose {")
        out.append("    body: &" + body_short + ",")
        out.append("    head: &" + head_short + ",")
        out.append("    tail: &" + tail_short + ",")
        out.append("    eyes: " + eyes_expr + ",")
        out.append("    head_first: " + ("true" if head_first else "false") + ",")
        out.append("    tail_last: " + ("true" if tail_last else "false") + ",")
        out.append("};\n")

        pose_entries.append((dotted, variant, const_name))

    return pose_entries


def main():
    out = []
    out.append("// AUTO-GENERATED by micropython/tools/gen_character.py")
    out.append("// DO NOT EDIT MANUALLY. Re-run the generator if character.py changes.")
    out.append("")
    out.append("#![allow(dead_code)]")
    out.append("")
    out.append("use crate::{")
    out.append("    character::{CharBody, CharHead, CharPart, Pose},")
    out.append("    render::Sprite,")
    out.append("};")
    out.append("")

    ident_to_name = {}
    for name in sorted(vars(cm)):
        if not name.startswith("CHAR_"):
            continue
        sprite = getattr(cm, name)
        if not isinstance(sprite, dict):
            continue
        ident_to_name[id(sprite)] = short_name(name)
        emit_sprite_blocks(out, name, sprite)

    pose_entries = emit_pose_constants(out, ident_to_name, cm.POSES)

    out.append("#[derive(Clone, Copy, PartialEq, Eq, Debug)]")
    out.append("pub enum PoseId {")
    for _, variant, _ in pose_entries:
        out.append("    " + variant + ",")
    out.append("}")
    out.append("")

    out.append("impl PoseId {")
    out.append("    pub fn data(self) -> &'static Pose {")
    out.append("        match self {")
    for _, variant, const_name in pose_entries:
        out.append("            PoseId::" + variant + " => &" + const_name + ",")
    out.append("        }")
    out.append("    }")
    out.append("")
    out.append("    pub fn name(self) -> &'static str {")
    out.append("        match self {")
    for dotted, variant, _ in pose_entries:
        out.append("            PoseId::" + variant + ' => "' + dotted + '",')
    out.append("        }")
    out.append("    }")
    out.append("}")
    out.append("")

    out.append("pub const ALL_POSES: &[PoseId] = &[")
    for _, variant, _ in pose_entries:
        out.append("    PoseId::" + variant + ",")
    out.append("];")
    out.append("")

    sys.stdout.write("\n".join(out))


if __name__ == "__main__":
    main()
