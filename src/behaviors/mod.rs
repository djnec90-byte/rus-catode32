pub mod affection;
pub mod attention;
pub mod being_groomed;
pub mod chattering;
pub mod common;
pub mod eating;
pub mod gift_bringing;
pub mod go_to;
pub mod greeting;
pub mod hearing;
pub mod hiding;
pub mod hunting;
pub mod idle;
pub mod investigating;
pub mod kneading;
pub mod lounging;
pub mod meandering;
pub mod mischief;
pub mod napping;
pub mod observing;
pub mod pacing;
pub mod playing;
pub mod self_grooming;
pub mod sleeping;
pub mod startled;
pub mod stretching;
pub mod sulking;
pub mod training;
pub mod vocalizing;
pub mod zoomies;

use crate::{
    behavior::{Behavior, BehaviorId, NextBehavior},
    context::GameContext,
    rand,
    scene::SceneId,
};

pub use affection::AffectionBehavior;
pub use attention::AttentionBehavior;
pub use being_groomed::BeingGroomedBehavior;
pub use chattering::ChatteringBehavior;
pub use eating::EatingBehavior;
pub use gift_bringing::GiftBringingBehavior;
pub use go_to::GoToBehavior;
pub use greeting::GreetingBehavior;
pub use hearing::HearingBehavior;
pub use hiding::HidingBehavior;
pub use hunting::HuntingBehavior;
pub use idle::IdleBehavior;
pub use investigating::InvestigatingBehavior;
pub use kneading::KneadingBehavior;
pub use lounging::LoungingBehavior;
pub use meandering::MeanderingBehavior;
pub use mischief::MischiefBehavior;
pub use napping::NappingBehavior;
pub use observing::ObservingBehavior;
pub use pacing::PacingBehavior;
pub use playing::PlayingBehavior;
pub use self_grooming::SelfGroomingBehavior;
pub use sleeping::SleepingBehavior;
pub use startled::StartledBehavior;
pub use stretching::StretchingBehavior;
pub use sulking::SulkingBehavior;
pub use training::TrainingBehavior;
pub use vocalizing::VocalizingBehavior;
pub use zoomies::ZoomiesBehavior;

/// Enum-dispatched union of every concrete behavior implementation. New
/// behaviors must be added here and in `from_next`.
#[allow(dead_code)]
pub enum ActiveBehavior {
    Idle(IdleBehavior),
    Sleeping(SleepingBehavior),
    Napping(NappingBehavior),
    Stretching(StretchingBehavior),
    Kneading(KneadingBehavior),
    Lounging(LoungingBehavior),
    Investigating(InvestigatingBehavior),
    Observing(ObservingBehavior),
    Chattering(ChatteringBehavior),
    Zoomies(ZoomiesBehavior),
    Vocalizing(VocalizingBehavior),
    SelfGrooming(SelfGroomingBehavior),
    BeingGroomed(BeingGroomedBehavior),
    Hunting(HuntingBehavior),
    GiftBringing(GiftBringingBehavior),
    Pacing(PacingBehavior),
    Sulking(SulkingBehavior),
    Mischief(MischiefBehavior),
    Hiding(HidingBehavior),
    Training(TrainingBehavior),
    Playing(PlayingBehavior),
    Affection(AffectionBehavior),
    Attention(AttentionBehavior),
    Eating(EatingBehavior),
    Startled(StartledBehavior),
    Meandering(MeanderingBehavior),
    GoTo(GoToBehavior),
    Hearing(HearingBehavior),
    Greeting(GreetingBehavior),
}

impl ActiveBehavior {
    pub fn from_next(next: NextBehavior) -> Self {
        match next {
            NextBehavior::Idle => ActiveBehavior::Idle(IdleBehavior::new()),
            NextBehavior::Sleeping => ActiveBehavior::Sleeping(SleepingBehavior::new()),
            NextBehavior::Napping => ActiveBehavior::Napping(NappingBehavior::new()),
            NextBehavior::Stretching => ActiveBehavior::Stretching(StretchingBehavior::new()),
            NextBehavior::Kneading => ActiveBehavior::Kneading(KneadingBehavior::new()),
            NextBehavior::Lounging => ActiveBehavior::Lounging(LoungingBehavior::new()),
            NextBehavior::Investigating => {
                ActiveBehavior::Investigating(InvestigatingBehavior::new())
            }
            NextBehavior::Observing => ActiveBehavior::Observing(ObservingBehavior::new()),
            NextBehavior::Chattering => ActiveBehavior::Chattering(ChatteringBehavior::new()),
            NextBehavior::Zoomies => ActiveBehavior::Zoomies(ZoomiesBehavior::new()),
            NextBehavior::Vocalizing => ActiveBehavior::Vocalizing(VocalizingBehavior::new()),
            NextBehavior::SelfGrooming => {
                ActiveBehavior::SelfGrooming(SelfGroomingBehavior::new())
            }
            NextBehavior::Hunting => ActiveBehavior::Hunting(HuntingBehavior::new()),
            NextBehavior::Pacing => ActiveBehavior::Pacing(PacingBehavior::new()),
            NextBehavior::Sulking => ActiveBehavior::Sulking(SulkingBehavior::new()),
            NextBehavior::Mischief => ActiveBehavior::Mischief(MischiefBehavior::new()),
            NextBehavior::Hiding => ActiveBehavior::Hiding(HidingBehavior::new()),
            NextBehavior::Meandering => ActiveBehavior::Meandering(MeanderingBehavior::new()),
            NextBehavior::Hearing(icon) => ActiveBehavior::Hearing(HearingBehavior::new(icon)),
            NextBehavior::Playing(variant) => {
                ActiveBehavior::Playing(PlayingBehavior::new(variant))
            }
            NextBehavior::Eating(source) => ActiveBehavior::Eating(EatingBehavior::new(source)),
            NextBehavior::GiftBringing(gift) => {
                ActiveBehavior::GiftBringing(GiftBringingBehavior::new(gift))
            }
            NextBehavior::Training(kind) => {
                ActiveBehavior::Training(TrainingBehavior::new(kind))
            }
            NextBehavior::GoTo(params) => ActiveBehavior::GoTo(GoToBehavior::new(params)),
            NextBehavior::Startled => ActiveBehavior::Startled(StartledBehavior::new()),
            NextBehavior::Greeting => ActiveBehavior::Greeting(GreetingBehavior::new()),
            NextBehavior::BeingGroomed => {
                ActiveBehavior::BeingGroomed(BeingGroomedBehavior::new())
            }
            NextBehavior::Affection(variant) => {
                ActiveBehavior::Affection(AffectionBehavior::new(variant))
            }
            NextBehavior::Attention(variant) => {
                ActiveBehavior::Attention(AttentionBehavior::new(variant))
            }
        }
    }

    pub fn as_dyn(&self) -> &dyn Behavior {
        match self {
            ActiveBehavior::Idle(b) => b,
            ActiveBehavior::Sleeping(b) => b,
            ActiveBehavior::Napping(b) => b,
            ActiveBehavior::Stretching(b) => b,
            ActiveBehavior::Kneading(b) => b,
            ActiveBehavior::Lounging(b) => b,
            ActiveBehavior::Investigating(b) => b,
            ActiveBehavior::Observing(b) => b,
            ActiveBehavior::Chattering(b) => b,
            ActiveBehavior::Zoomies(b) => b,
            ActiveBehavior::Vocalizing(b) => b,
            ActiveBehavior::SelfGrooming(b) => b,
            ActiveBehavior::BeingGroomed(b) => b,
            ActiveBehavior::Hunting(b) => b,
            ActiveBehavior::GiftBringing(b) => b,
            ActiveBehavior::Pacing(b) => b,
            ActiveBehavior::Sulking(b) => b,
            ActiveBehavior::Mischief(b) => b,
            ActiveBehavior::Hiding(b) => b,
            ActiveBehavior::Training(b) => b,
            ActiveBehavior::Playing(b) => b,
            ActiveBehavior::Affection(b) => b,
            ActiveBehavior::Attention(b) => b,
            ActiveBehavior::Eating(b) => b,
            ActiveBehavior::Startled(b) => b,
            ActiveBehavior::Meandering(b) => b,
            ActiveBehavior::GoTo(b) => b,
            ActiveBehavior::Hearing(b) => b,
            ActiveBehavior::Greeting(b) => b,
        }
    }

    pub fn as_dyn_mut(&mut self) -> &mut dyn Behavior {
        match self {
            ActiveBehavior::Idle(b) => b,
            ActiveBehavior::Sleeping(b) => b,
            ActiveBehavior::Napping(b) => b,
            ActiveBehavior::Stretching(b) => b,
            ActiveBehavior::Kneading(b) => b,
            ActiveBehavior::Lounging(b) => b,
            ActiveBehavior::Investigating(b) => b,
            ActiveBehavior::Observing(b) => b,
            ActiveBehavior::Chattering(b) => b,
            ActiveBehavior::Zoomies(b) => b,
            ActiveBehavior::Vocalizing(b) => b,
            ActiveBehavior::SelfGrooming(b) => b,
            ActiveBehavior::BeingGroomed(b) => b,
            ActiveBehavior::Hunting(b) => b,
            ActiveBehavior::GiftBringing(b) => b,
            ActiveBehavior::Pacing(b) => b,
            ActiveBehavior::Sulking(b) => b,
            ActiveBehavior::Mischief(b) => b,
            ActiveBehavior::Hiding(b) => b,
            ActiveBehavior::Training(b) => b,
            ActiveBehavior::Playing(b) => b,
            ActiveBehavior::Affection(b) => b,
            ActiveBehavior::Attention(b) => b,
            ActiveBehavior::Eating(b) => b,
            ActiveBehavior::Startled(b) => b,
            ActiveBehavior::Meandering(b) => b,
            ActiveBehavior::GoTo(b) => b,
            ActiveBehavior::Hearing(b) => b,
            ActiveBehavior::Greeting(b) => b,
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-selection
// ---------------------------------------------------------------------------

const AUTO_SELECT_NAMES: &[BehaviorId] = &[
    BehaviorId::Sleeping,
    BehaviorId::Napping,
    BehaviorId::Zoomies,
    BehaviorId::Vocalizing,
    BehaviorId::Hunting,
    BehaviorId::Investigating,
    BehaviorId::Observing,
    BehaviorId::SelfGrooming,
    BehaviorId::Stretching,
    BehaviorId::Pacing,
    BehaviorId::Sulking,
    BehaviorId::Mischief,
    BehaviorId::Hiding,
    BehaviorId::Lounging,
    BehaviorId::Startled,
];
// TODO(playing): include `BehaviorId::Playing` once toy inventory exists so
// auto-select can require a solo-play variant.

pub fn auto_select(ctx: &mut GameContext) -> NextBehavior {
    // Random meander gate — scaled by sickness.
    let meander_p = if ctx.sickness >= 8.0 {
        0.02
    } else if ctx.sickness >= 5.0 {
        0.06
    } else if ctx.sickness >= 2.0 {
        0.12
    } else {
        0.2
    };
    if ctx.fullness >= 5.0
        && can_trigger(BehaviorId::Meandering, ctx)
        && rand::rand_f32(&mut ctx.rng) <= meander_p
    {
        return NextBehavior::Meandering;
    }

    // Scene-exit detour.
    if let Some(next) = common::auto_select_scene_exit(ctx) {
        return next;
    }

    // High serenity stay-idle gate.
    if ctx.fullness >= 5.0
        && ctx.serenity > 25.0
        && rand::rand_f32(&mut ctx.rng) < (ctx.serenity - 25.0) / 150.0
    {
        return NextBehavior::Idle;
    }

    // Gather eligible candidates.
    let mut candidates: heapless::Vec<(BehaviorId, u32), 16> = heapless::Vec::new();
    for &id in AUTO_SELECT_NAMES {
        if common::sick_blocks(id, ctx) {
            continue;
        }
        if !can_trigger(id, ctx) {
            continue;
        }
        let mut p = priority(id, ctx);
        // Recency penalty.
        if let Some(idx) = ctx.recent_index(id) {
            let mut penalty = 50_i32 - (idx as i32) * 10;
            if ctx.sickness >= 2.0
                && matches!(id, BehaviorId::Sleeping | BehaviorId::Napping)
            {
                penalty /= 2;
            }
            p = p.saturating_add(penalty.max(0) as u32);
        }
        let _ = candidates.push((id, p));
    }

    if candidates.is_empty() {
        return NextBehavior::Idle;
    }

    // Bin priorities (ceil to nearest 10) and randomly pick within the lowest bin.
    let mut best_bin = u32::MAX;
    for &(_, p) in &candidates {
        let b = ((p + 9) / 10) * 10;
        if b < best_bin {
            best_bin = b;
        }
    }
    let mut tied: heapless::Vec<BehaviorId, 16> = heapless::Vec::new();
    for &(id, p) in &candidates {
        if ((p + 9) / 10) * 10 == best_bin {
            let _ = tied.push(id);
        }
    }
    let pick = rand::rand_range_u32(&mut ctx.rng, 0, (tied.len() - 1) as u32) as usize;
    let chosen = tied[pick];
    to_next_default(chosen)
}

fn to_next_default(id: BehaviorId) -> NextBehavior {
    match id {
        BehaviorId::Sleeping => NextBehavior::Sleeping,
        BehaviorId::Napping => NextBehavior::Napping,
        BehaviorId::Zoomies => NextBehavior::Zoomies,
        BehaviorId::Vocalizing => NextBehavior::Vocalizing,
        BehaviorId::Hunting => NextBehavior::Hunting,
        BehaviorId::Investigating => NextBehavior::Investigating,
        BehaviorId::Observing => NextBehavior::Observing,
        BehaviorId::SelfGrooming => NextBehavior::SelfGrooming,
        BehaviorId::Stretching => NextBehavior::Stretching,
        BehaviorId::Pacing => NextBehavior::Pacing,
        BehaviorId::Sulking => NextBehavior::Sulking,
        BehaviorId::Mischief => NextBehavior::Mischief,
        BehaviorId::Hiding => NextBehavior::Hiding,
        BehaviorId::Lounging => NextBehavior::Lounging,
        BehaviorId::Startled => NextBehavior::Startled,
        BehaviorId::Meandering => NextBehavior::Meandering,
        BehaviorId::Idle => NextBehavior::Idle,
        BehaviorId::Chattering => NextBehavior::Chattering,
        _ => NextBehavior::Idle,
    }
}

// can_trigger dispatch — keeps the gate logic centralized.
fn can_trigger(id: BehaviorId, ctx: &GameContext) -> bool {
    match id {
        BehaviorId::Sleeping => SleepingBehavior::can_trigger(ctx),
        BehaviorId::Napping => NappingBehavior::can_trigger(ctx),
        BehaviorId::Zoomies => ZoomiesBehavior::can_trigger(ctx),
        BehaviorId::Vocalizing => VocalizingBehavior::can_trigger(ctx),
        BehaviorId::Hunting => HuntingBehavior::can_trigger(ctx),
        BehaviorId::Investigating => InvestigatingBehavior::can_trigger(ctx),
        BehaviorId::Observing => ObservingBehavior::can_trigger(ctx),
        BehaviorId::SelfGrooming => SelfGroomingBehavior::can_trigger(ctx),
        BehaviorId::Stretching => StretchingBehavior::can_trigger(ctx),
        BehaviorId::Pacing => PacingBehavior::can_trigger(ctx),
        BehaviorId::Sulking => SulkingBehavior::can_trigger(ctx),
        BehaviorId::Mischief => MischiefBehavior::can_trigger(ctx),
        BehaviorId::Hiding => HidingBehavior::can_trigger(ctx),
        BehaviorId::Lounging => LoungingBehavior::can_trigger(ctx),
        BehaviorId::Startled => StartledBehavior::can_trigger(ctx),
        BehaviorId::Meandering => MeanderingBehavior::can_trigger(ctx),
        _ => false,
    }
}

fn priority(id: BehaviorId, ctx: &GameContext) -> u32 {
    // The same `ctx.rng` is borrowed mutably by each priority fn via interior
    // copy — they take `&GameContext` and roll using a local RNG view. We use
    // the global rng field via a tiny mutation helper to keep priorities
    // deterministically advanced.
    let mut rng = ctx.rng.wrapping_mul(2654435769).wrapping_add(id as u32);
    match id {
        BehaviorId::Sleeping => SleepingBehavior::priority(ctx, &mut rng),
        BehaviorId::Napping => NappingBehavior::priority(ctx, &mut rng),
        BehaviorId::Zoomies => ZoomiesBehavior::priority(ctx, &mut rng),
        BehaviorId::Vocalizing => VocalizingBehavior::priority(ctx, &mut rng),
        BehaviorId::Hunting => HuntingBehavior::priority(ctx, &mut rng),
        BehaviorId::Investigating => InvestigatingBehavior::priority(ctx, &mut rng),
        BehaviorId::Observing => ObservingBehavior::priority(ctx, &mut rng),
        BehaviorId::SelfGrooming => SelfGroomingBehavior::priority(ctx, &mut rng),
        BehaviorId::Stretching => StretchingBehavior::priority(ctx, &mut rng),
        BehaviorId::Pacing => PacingBehavior::priority(ctx, &mut rng),
        BehaviorId::Sulking => SulkingBehavior::priority(ctx, &mut rng),
        BehaviorId::Mischief => MischiefBehavior::priority(ctx, &mut rng),
        BehaviorId::Hiding => HidingBehavior::priority(ctx, &mut rng),
        BehaviorId::Lounging => LoungingBehavior::priority(ctx, &mut rng),
        BehaviorId::Startled => StartledBehavior::priority(ctx, &mut rng),
        _ => 100,
    }
}

/// Helper used by every scene to know whether the current location is an
/// outdoor one (drives weather effects on bonuses, etc.).
#[allow(dead_code)]
pub fn scene_is_outdoor(scene: SceneId) -> bool {
    common::is_outdoor(scene)
}
