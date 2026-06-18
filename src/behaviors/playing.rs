use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior, PlayVariant},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Excited,
    Playing,
    Tired,
    Rejecting,
}

pub struct PlayingBehavior {
    variant: PlayVariant,
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    rejected: bool,
    eye_frame: Option<usize>,
}

impl PlayingBehavior {
    pub fn new(variant: PlayVariant) -> Self {
        Self {
            variant,
            phase: Phase::Excited,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 12.0,
            pose_id: PoseId::SittingForwardShocked,
            rejected: false,
            eye_frame: None,
        }
    }
}

impl Behavior for PlayingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Playing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }
    fn eye_frame_override(&self) -> Option<usize> {
        self.eye_frame
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        // Reject if playfulness extremely low or sick.
        self.rejected = ctx.playfulness < 15.0 || ctx.sickness >= 5.0;
        self.phase = if self.rejected {
            Phase::Rejecting
        } else {
            Phase::Excited
        };
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 10.0, 16.0);
        self.pose_id = if self.rejected {
            PoseId::SittingSideAnnoyed
        } else {
            PoseId::SittingForwardShocked
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        // TODO(playing): port full sliding-toy variant logic (verlet feather,
        // bubble particles, laser line, pounce slides, durability decrement
        // on exit). Currently behaves as a simple phased play loop.
        match self.phase {
            Phase::Rejecting if self.phase_timer >= 3.0 => return BehaviorState::Completed,
            Phase::Excited if self.phase_timer >= 1.5 => {
                self.phase = Phase::Playing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::PlayfulForwardWowed;
            }
            Phase::Playing if self.phase_timer >= self.total - 2.0 => {
                self.phase = Phase::Tired;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideContent;
            }
            Phase::Tired if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        if self.rejected {
            Some(NextBehavior::Meandering)
        } else {
            None
        }
    }

    fn exit(&mut self, ctx: &mut GameContext, completed: bool) {
        if completed && !self.rejected {
            ctx.milestone_played = true;
            // TODO(inventory): decrement toy durability for self.variant.
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        if self.rejected {
            return;
        }
        let mut bonus: heapless::Vec<(StatId, f32), 6> = heapless::Vec::new();
        let (play, energy, fit) = match self.variant {
            PlayVariant::Ball => (10.0, -7.0, 0.5),
            PlayVariant::String | PlayVariant::Feather => (12.0, -6.0, 0.6),
            PlayVariant::Mouse => (11.0, -7.0, 0.6),
            PlayVariant::Hand => (9.0, -4.0, 0.3),
            PlayVariant::Laser => (12.0, -8.0, 0.5),
            PlayVariant::Bubbles => (10.0, -5.0, 0.4),
        };
        let _ = bonus.push((StatId::Playfulness, play));
        let _ = bonus.push((StatId::Energy, energy));
        let _ = bonus.push((StatId::Fitness, fit));
        let _ = bonus.push((StatId::Affection, 0.7));
        let _ = bonus.push((StatId::Fulfillment, 0.6));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
