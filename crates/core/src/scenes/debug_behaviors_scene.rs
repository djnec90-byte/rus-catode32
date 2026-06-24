use embedded_graphics::prelude::{Point, Size};
use crate::t;

use crate::{
    behavior::{
        AffectionVariant, AttentionVariant, BehaviorId, BehaviorManager, EatingSource, GiftKind,
        NextBehavior, PlayVariant, TrainingKind,
    },
    context::{FoodItem, GameContext},
    entities::character::Character,
    input::{Button, Buttons},
    render::Renderer,
    scene::{Scene, SceneId},
    ui::scrollbar::Scrollbar,
};

const LINES_VISIBLE: usize = 7;
const LINE_HEIGHT: i32 = 8;
const FLOOR_Y: i32 = 60;
const CHAR_X: i32 = 100;
const CHAR_Y: i32 = 60;
const SCENE_X_MIN: i32 = 10;
const SCENE_X_MAX: i32 = 118;

#[derive(Clone, Copy)]
enum Entry {
    Idle,
    Sleeping,
    Napping,
    Stretching,
    Kneading,
    Lounging,
    Investigating,
    Startled,
    Observing,
    Chattering,
    Zoomies,
    Vocalizing,
    SelfGrooming,
    BeingGroomed,
    Hunting,
    GiftFish,
    GiftMouse,
    Pacing,
    Meandering,
    Sulking,
    Mischief,
    Hiding,
    TrainIntel,
    TrainBehav,
    TrainFitness,
    TrainSocial,
    PlayBall,
    PlayString,
    PlayFeather,
    PlayMouse,
    PlayHand,
    PlayLaser,
    PlayBubbles,
    AffectionKiss,
    AffectionPets,
    AffectionScratch,
    AttentionPsst,
    AttentionBird,
    HearingExclaim,
    HearingHeart,
    HearingNote,
    EatKibble,
    EatWet,
    EatFish,
    EatSnack,
    EatTreat,
    Greeting,
}

impl Entry {
    fn label(self) -> &'static str {
        match self {
            Entry::Idle => t!("Idle"),
            Entry::Sleeping => t!("Sleeping"),
            Entry::Napping => t!("Napping"),
            Entry::Stretching => t!("Stretching"),
            Entry::Kneading => t!("Kneading"),
            Entry::Lounging => t!("Lounging"),
            Entry::Investigating => t!("Investigating"),
            Entry::Startled => t!("Startled"),
            Entry::Observing => t!("Observing"),
            Entry::Chattering => t!("Chattering"),
            Entry::Zoomies => t!("Zoomies"),
            Entry::Vocalizing => t!("Vocalizing"),
            Entry::SelfGrooming => t!("Self Grooming"),
            Entry::BeingGroomed => t!("Being Groomed"),
            Entry::Hunting => t!("Hunting"),
            Entry::GiftFish => "Gift (fish)",
            Entry::GiftMouse => "Gift (mouse)",
            Entry::Pacing => t!("Pacing"),
            Entry::Meandering => t!("Meandering"),
            Entry::Sulking => t!("Sulking"),
            Entry::Mischief => t!("Mischief"),
            Entry::Hiding => t!("Hiding"),
            Entry::TrainIntel => "Train (intel)",
            Entry::TrainBehav => "Train (behav)",
            Entry::TrainFitness => "Train (fit)",
            Entry::TrainSocial => "Train (social)",
            Entry::PlayBall => "Play (ball)",
            Entry::PlayString => "Play (string)",
            Entry::PlayFeather => "Play (feather)",
            Entry::PlayMouse => "Play (mouse)",
            Entry::PlayHand => "Play (hand)",
            Entry::PlayLaser => "Play (laser)",
            Entry::PlayBubbles => "Play (bubbles)",
            Entry::AffectionKiss => "Affection (kiss)",
            Entry::AffectionPets => "Affection (pets)",
            Entry::AffectionScratch => "Affection (scratch)",
            Entry::AttentionPsst => "Attention (psst)",
            Entry::AttentionBird => "Attention (bird)",
            Entry::HearingExclaim => t!("Hearing (exclaim)"),
            Entry::HearingHeart => t!("Hearing (heart)"),
            Entry::HearingNote => t!("Hearing (note)"),
            Entry::EatKibble => "Eat (kibble)",
            Entry::EatWet => "Eat (wet)",
            Entry::EatFish => "Eat (fish)",
            Entry::EatSnack => "Eat (snack)",
            Entry::EatTreat => "Eat (treat)",
            Entry::Greeting => "Greeting",
        }
    }

    fn to_next(self) -> NextBehavior {
        match self {
            Entry::Idle => NextBehavior::Idle,
            Entry::Sleeping => NextBehavior::Sleeping,
            Entry::Napping => NextBehavior::Napping,
            Entry::Stretching => NextBehavior::Stretching,
            Entry::Kneading => NextBehavior::Kneading,
            Entry::Lounging => NextBehavior::Lounging,
            Entry::Investigating => NextBehavior::Investigating,
            Entry::Startled => NextBehavior::Startled,
            Entry::Observing => NextBehavior::Observing,
            Entry::Chattering => NextBehavior::Chattering,
            Entry::Zoomies => NextBehavior::Zoomies,
            Entry::Vocalizing => NextBehavior::Vocalizing,
            Entry::SelfGrooming => NextBehavior::SelfGrooming,
            Entry::BeingGroomed => NextBehavior::BeingGroomed,
            Entry::Hunting => NextBehavior::Hunting,
            Entry::GiftFish => NextBehavior::GiftBringing(GiftKind::Fish),
            Entry::GiftMouse => NextBehavior::GiftBringing(GiftKind::Mouse),
            Entry::Pacing => NextBehavior::Pacing,
            Entry::Meandering => NextBehavior::Meandering,
            Entry::Sulking => NextBehavior::Sulking,
            Entry::Mischief => NextBehavior::Mischief,
            Entry::Hiding => NextBehavior::Hiding,
            Entry::TrainIntel => NextBehavior::Training(TrainingKind::Intelligence),
            Entry::TrainBehav => NextBehavior::Training(TrainingKind::Behavior),
            Entry::TrainFitness => NextBehavior::Training(TrainingKind::Fitness),
            Entry::TrainSocial => NextBehavior::Training(TrainingKind::Sociability),
            Entry::PlayBall => NextBehavior::Playing(PlayVariant::Ball),
            Entry::PlayString => NextBehavior::Playing(PlayVariant::String),
            Entry::PlayFeather => NextBehavior::Playing(PlayVariant::Feather),
            Entry::PlayMouse => NextBehavior::Playing(PlayVariant::Mouse),
            Entry::PlayHand => NextBehavior::Playing(PlayVariant::Hand),
            Entry::PlayLaser => NextBehavior::Playing(PlayVariant::Laser),
            Entry::PlayBubbles => NextBehavior::Playing(PlayVariant::Bubbles),
            Entry::AffectionKiss => NextBehavior::Affection(AffectionVariant::Kiss),
            Entry::AffectionPets => NextBehavior::Affection(AffectionVariant::Pets),
            Entry::AffectionScratch => NextBehavior::Affection(AffectionVariant::Scratching),
            Entry::AttentionPsst => NextBehavior::Attention(AttentionVariant::Psst),
            Entry::AttentionBird => NextBehavior::Attention(AttentionVariant::PointBird),
            Entry::HearingExclaim => NextBehavior::Hearing(Some("exclaim")),
            Entry::HearingHeart => NextBehavior::Hearing(Some("heart")),
            Entry::HearingNote => NextBehavior::Hearing(Some("note")),
            Entry::EatKibble => NextBehavior::Eating(EatingSource::Item(FoodItem::Kibble)),
            Entry::EatWet => NextBehavior::Eating(EatingSource::Item(FoodItem::Chicken)),
            Entry::EatFish => NextBehavior::Eating(EatingSource::Item(FoodItem::Tuna)),
            Entry::EatSnack => NextBehavior::Eating(EatingSource::CaughtSnack),
            Entry::EatTreat => NextBehavior::Eating(EatingSource::Item(FoodItem::Treats)),
            Entry::Greeting => NextBehavior::Greeting,
        }
    }

    fn target_id(self) -> BehaviorId {
        match self {
            Entry::Idle => BehaviorId::Idle,
            Entry::Sleeping => BehaviorId::Sleeping,
            Entry::Napping => BehaviorId::Napping,
            Entry::Stretching => BehaviorId::Stretching,
            Entry::Kneading => BehaviorId::Kneading,
            Entry::Lounging => BehaviorId::Lounging,
            Entry::Investigating => BehaviorId::Investigating,
            Entry::Startled => BehaviorId::Startled,
            Entry::Observing => BehaviorId::Observing,
            Entry::Chattering => BehaviorId::Chattering,
            Entry::Zoomies => BehaviorId::Zoomies,
            Entry::Vocalizing => BehaviorId::Vocalizing,
            Entry::SelfGrooming => BehaviorId::SelfGrooming,
            Entry::BeingGroomed => BehaviorId::BeingGroomed,
            Entry::Hunting => BehaviorId::Hunting,
            Entry::GiftFish | Entry::GiftMouse => BehaviorId::GiftBringing,
            Entry::Pacing => BehaviorId::Pacing,
            Entry::Meandering => BehaviorId::Meandering,
            Entry::Sulking => BehaviorId::Sulking,
            Entry::Mischief => BehaviorId::Mischief,
            Entry::Hiding => BehaviorId::Hiding,
            Entry::TrainIntel
            | Entry::TrainBehav
            | Entry::TrainFitness
            | Entry::TrainSocial => BehaviorId::Training,
            Entry::PlayBall
            | Entry::PlayString
            | Entry::PlayFeather
            | Entry::PlayMouse
            | Entry::PlayHand
            | Entry::PlayLaser
            | Entry::PlayBubbles => BehaviorId::Playing,
            Entry::AffectionKiss
            | Entry::AffectionPets
            | Entry::AffectionScratch => BehaviorId::Affection,
            Entry::AttentionPsst | Entry::AttentionBird => BehaviorId::Attention,
            Entry::HearingExclaim | Entry::HearingHeart | Entry::HearingNote => {
                BehaviorId::Hearing
            }
            Entry::EatKibble
            | Entry::EatWet
            | Entry::EatFish
            | Entry::EatSnack
            | Entry::EatTreat => BehaviorId::Eating,
            Entry::Greeting => BehaviorId::Greeting,
        }
    }
}

const ENTRIES: &[Entry] = &[
    Entry::Idle,
    Entry::Sleeping,
    Entry::Napping,
    Entry::Stretching,
    Entry::Kneading,
    Entry::Lounging,
    Entry::Investigating,
    Entry::Startled,
    Entry::Observing,
    Entry::Chattering,
    Entry::Zoomies,
    Entry::Vocalizing,
    Entry::SelfGrooming,
    Entry::BeingGroomed,
    Entry::Hunting,
    Entry::GiftFish,
    Entry::GiftMouse,
    Entry::Pacing,
    Entry::Meandering,
    Entry::Sulking,
    Entry::Mischief,
    Entry::Hiding,
    Entry::TrainIntel,
    Entry::TrainBehav,
    Entry::TrainFitness,
    Entry::TrainSocial,
    Entry::PlayBall,
    Entry::PlayString,
    Entry::PlayFeather,
    Entry::PlayMouse,
    Entry::PlayHand,
    Entry::PlayLaser,
    Entry::PlayBubbles,
    Entry::AffectionKiss,
    Entry::AffectionPets,
    Entry::AffectionScratch,
    Entry::AttentionPsst,
    Entry::AttentionBird,
    Entry::HearingExclaim,
    Entry::HearingHeart,
    Entry::HearingNote,
    Entry::EatKibble,
    Entry::EatWet,
    Entry::EatFish,
    Entry::EatSnack,
    Entry::EatTreat,
    Entry::Greeting,
];

pub struct DebugBehaviorsScene {
    character: Character,
    behaviors: BehaviorManager,
    selected: usize,
    scroll: usize,
}

impl DebugBehaviorsScene {
    pub fn new() -> Self {
        Self {
            character: Character::new(Point::new(CHAR_X, CHAR_Y)),
            behaviors: BehaviorManager::new(),
            selected: 0,
            scroll: 0,
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
    }

    fn move_down(&mut self) {
        if self.selected + 1 < ENTRIES.len() {
            self.selected += 1;
        }
        if self.selected >= self.scroll + LINES_VISIBLE {
            self.scroll = self.selected + 1 - LINES_VISIBLE;
        }
    }

    fn trigger_selected(&mut self, ctx: &mut GameContext) {
        let next = ENTRIES[self.selected].to_next();
        self.behaviors.trigger(next, ctx, &mut self.character);
    }
}

impl Scene for DebugBehaviorsScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        ctx.scene_x_min = SCENE_X_MIN;
        ctx.scene_x_max = SCENE_X_MAX;
        self.character.reseed_anim();
        self.behaviors.start(ctx, &mut self.character);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::B) || buttons.was_just_pressed(Button::Menu1) {
            return Some(ctx.last_main_scene);
        }
        if buttons.was_just_pressed(Button::Up) {
            self.move_up();
        }
        if buttons.was_just_pressed(Button::Down) {
            self.move_down();
        }
        if buttons.was_just_pressed(Button::A) {
            self.trigger_selected(ctx);
        }

        self.behaviors.update(ctx, &mut self.character, dt);
        let pose = self.behaviors.current_pose();
        self.character.set_pose(pose);
        self.character.animate(dt);

        // Behaviors may request a scene transition (e.g. go_to). Consume it so
        // the debug scene doesn't end up jumping somewhere unexpected.
        ctx.pending_scene.take();
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        renderer.draw_line(Point::new(0, FLOOR_Y), Point::new(128, FLOOR_Y));

        self.draw_list(renderer);
        self.draw_progress(renderer);

        self.character.draw(renderer, 0);
        let char_screen = Point::new(self.character.pos.x, self.character.pos.y);
        self.behaviors
            .draw_overlay(renderer, ctx, char_screen, self.character.mirror_h);
    }
}

impl DebugBehaviorsScene {
    fn draw_list(&self, renderer: &mut Renderer) {
        let visible_end = (self.scroll + LINES_VISIBLE).min(ENTRIES.len());
        let current_id = self.behaviors.current_id();
        for (row, idx) in (self.scroll..visible_end).enumerate() {
            let entry = ENTRIES[idx];
            let y = row as i32 * LINE_HEIGHT;
            let selected = idx == self.selected;
            if selected {
                renderer.draw_rect(Point::new(0, y), Size::new(128, LINE_HEIGHT as u32), true);
            }

            let mut buf = [0u8; 24];
            let label = entry.label();
            let active_marker = entry.target_id() == current_id;
            let written = format_label(&mut buf, label, active_marker);
            let text = core::str::from_utf8(&buf[..written]).unwrap_or(label);

            if selected {
                renderer.draw_text_inverted(text, Point::new(1, y));
            } else {
                renderer.draw_text(text, Point::new(1, y));
            }
        }

        let track_height = (LINES_VISIBLE as i32 * LINE_HEIGHT) as u32;
        let bar = Scrollbar::right_edge(0, track_height);
        bar.draw(renderer, ENTRIES.len(), LINES_VISIBLE, self.scroll);
    }

    fn draw_progress(&self, renderer: &mut Renderer) {
        let progress = self.behaviors.current_progress();
        if progress <= 0.0 {
            return;
        }
        let width = (progress.clamp(0.0, 1.0) * 128.0) as i32;
        if width > 0 {
            renderer.draw_rect(
                Point::new(0, FLOOR_Y),
                Size::new(width as u32, 4),
                true,
            );
        }
    }
}

/// Copy `label` into `buf`, appending `*` if `active`. Returns the byte length
/// written. The label is truncated if it would overflow `buf`.
fn format_label(buf: &mut [u8], label: &str, active: bool) -> usize {
    let max = if active { buf.len().saturating_sub(1) } else { buf.len() };
    let bytes = label.as_bytes();
    let n = bytes.len().min(max);
    buf[..n].copy_from_slice(&bytes[..n]);
    if active && n < buf.len() {
        buf[n] = b'*';
        n + 1
    } else {
        n
    }
}
