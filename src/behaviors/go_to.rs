use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, GoToParams, NextBehavior},
    behaviors::common,
    context::GameContext,
    entities::character::Character,
};

pub struct GoToBehavior {
    params: GoToParams,
    pose_id: PoseId,
    walker_accum: f32,
    arrived: bool,
    arrival_t: f32,
}

impl GoToBehavior {
    pub fn new(params: GoToParams) -> Self {
        Self {
            params,
            pose_id: PoseId::WalkingSideNeutral,
            walker_accum: 0.0,
            arrived: false,
            arrival_t: 0.0,
        }
    }
}

impl Behavior for GoToBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::GoTo
    }

    fn progress(&self) -> f32 {
        if self.arrived {
            1.0
        } else {
            0.5
        }
    }

    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, _ctx: &mut GameContext, character: &mut Character) {
        self.pose_id = PoseId::WalkingSideDetermined;
        character.mirror_h = self.params.target_x > character.pos.x;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        if self.arrived {
            self.arrival_t += dt;
            if self.arrival_t >= 0.5 {
                if let Some(scene) = self.params.pending_scene {
                    ctx.pending_scene = Some(scene);
                }
                return BehaviorState::Completed;
            }
            return BehaviorState::Running;
        }
        let dir = if character.pos.x < self.params.target_x {
            1
        } else {
            -1
        };
        common::step_walker(
            character,
            ctx,
            dir,
            self.params.speed,
            dt,
            &mut self.walker_accum,
        );
        if common::distance_to(character, self.params.target_x) <= 1 {
            self.arrived = true;
            self.pose_id = PoseId::SittingSideNeutral;
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        self.params.then.map(|t| t.into_next())
    }
}
