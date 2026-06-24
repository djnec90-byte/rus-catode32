use crate::t;

use crate::{
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scene::{Scene, SceneId},
    ui::settings::{SettingItem, SettingValue, Settings, SettingsResult},
};

const STEP: i32 = 16;
const TOGGLE_OPTIONS: &[&str] = &["ON", "OFF"];

const TOGGLE_IDX: usize = 0;
const R_IDX: usize = 1;
const G_IDX: usize = 2;
const B_IDX: usize = 3;

pub struct DebugLedScene {
    settings: Settings,
}

impl DebugLedScene {
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
        }
    }

    fn apply(&self, ctx: &mut GameContext) {
        let on = matches!(
            self.settings.value(TOGGLE_IDX),
            Some(SettingValue::Choice { index: 0, .. })
        );
        if !on {
            ctx.led.off();
            return;
        }
        let r = read_int(&self.settings, R_IDX);
        let g = read_int(&self.settings, G_IDX);
        let b = read_int(&self.settings, B_IDX);
        ctx.led.set(r, g, b);
    }
}

fn read_int(settings: &Settings, idx: usize) -> u8 {
    match settings.value(idx) {
        Some(SettingValue::Int { value, .. }) => (*value).clamp(0, 255) as u8,
        _ => 0,
    }
}

impl Scene for DebugLedScene {
    fn enter(&mut self, _ctx: &mut GameContext) {
        let items = [
            SettingItem::choice(t!("Toggle"), 0, TOGGLE_OPTIONS),
            SettingItem::int(t!("R"), 255, 0, 255, STEP),
            SettingItem::int(t!("G"), 255, 0, 255, STEP),
            SettingItem::int(t!("B"), 255, 0, 255, STEP),
        ];
        self.settings.open(&items);
    }

    fn exit(&mut self, ctx: &mut GameContext) {
        self.apply(ctx);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        let result = self.settings.handle_input(buttons);
        self.apply(ctx);
        match result {
            SettingsResult::Continue | SettingsResult::Activated(_) => None,
            SettingsResult::Closed => Some(ctx.last_main_scene),
        }
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.settings.draw(renderer);
    }
}
