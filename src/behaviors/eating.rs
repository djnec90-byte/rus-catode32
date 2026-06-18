use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{FoodKind, GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Lowering,
    PreEating,
    Eating,
    PostEating,
    Rejecting,
}

pub struct EatingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    food: FoodKind,
    rejected: bool,
}

impl EatingBehavior {
    pub fn new(food: FoodKind) -> Self {
        Self {
            phase: Phase::Lowering,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 8.0,
            pose_id: PoseId::StandingSideHappy,
            food,
            rejected: false,
        }
    }

    fn would_reject(&self, ctx: &GameContext) -> bool {
        // TODO(meal_system): least_fav_meal is now a specific FoodItem, not a
        // FoodKind. Reinstate the comparison once eating accepts a FoodItem.
        ctx.fullness > 85.0
    }
}

impl Behavior for EatingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Eating
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.rejected = self.would_reject(ctx);
        self.phase = if self.rejected {
            Phase::Rejecting
        } else {
            Phase::Lowering
        };
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 7.0, 10.0);
        self.pose_id = if self.rejected {
            PoseId::SittingSideAnnoyed
        } else {
            PoseId::StandingSideHappy
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Rejecting if self.phase_timer >= 3.0 => return BehaviorState::Completed,
            Phase::Lowering if self.phase_timer >= 1.0 => {
                self.phase = Phase::PreEating;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LeaningForwardSideNeutral;
            }
            Phase::PreEating if self.phase_timer >= 1.0 => {
                self.phase = Phase::Eating;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LeaningForwardSideEating;
            }
            Phase::Eating if self.phase_timer >= self.total - 3.0 => {
                self.phase = Phase::PostEating;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideHappy;
            }
            Phase::PostEating if self.phase_timer >= 2.0 => return BehaviorState::Completed,
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
            ctx.record_meal(self.food);
            ctx.milestone_fed = true;
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        if self.rejected {
            return;
        }
        // Stat changes per food kind. Mirrors Python eating.py bonuses.
        let mut bonus: heapless::Vec<(StatId, f32), 6> = heapless::Vec::new();
        let fullness = match self.food {
            FoodKind::Kibble => 22.0,
            FoodKind::WetFood => 28.0,
            FoodKind::Treat => 8.0,
            FoodKind::Fish => 30.0,
            FoodKind::CaughtSnack => 12.0,
        };
        let _ = bonus.push((StatId::Fullness, fullness));
        let _ = bonus.push((StatId::Comfort, 1.0));
        let _ = bonus.push((StatId::Affection, 0.5));
        if matches!(self.food, FoodKind::Treat) {
            let _ = bonus.push((StatId::Playfulness, 2.0));
        }
        // TODO(meal_system): apply variety penalty using ctx.recent_meals once
        // the meal-flavor system is ported.
        // TODO(personality): apply fav_meal bonus when fav_meal == self.food.
        // Snack streak adds sickness.
        if matches!(self.food, FoodKind::Treat | FoodKind::CaughtSnack) {
            let snack_streak = ctx
                .recent_meals
                .iter()
                .take_while(|m| matches!(m, FoodKind::Treat | FoodKind::CaughtSnack))
                .count();
            if snack_streak >= 2 {
                ctx.sickness = (ctx.sickness + 0.2 * snack_streak as f32).min(10.0);
            }
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
