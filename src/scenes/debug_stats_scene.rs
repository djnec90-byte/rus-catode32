use crate::{
    context::{GameContext, StatId},
    input::Buttons,
    render::Renderer,
    scene::{Scene, SceneId},
    ui::settings::{SettingItem, SettingValue, Settings, SettingsResult},
};

#[derive(Clone, Copy)]
enum Field {
    Stat(StatId),
    Health,
    Sickness,
}

struct Entry {
    label: &'static str,
    field: Field,
    max: i32,
}

const ENTRIES: &[Entry] = &[
    Entry { label: "Fullness",    field: Field::Stat(StatId::Fullness),        max: 100 },
    Entry { label: "Energy",      field: Field::Stat(StatId::Energy),          max: 100 },
    Entry { label: "Comfort",     field: Field::Stat(StatId::Comfort),         max: 100 },
    Entry { label: "Playfulness", field: Field::Stat(StatId::Playfulness),     max: 100 },
    Entry { label: "Focus",       field: Field::Stat(StatId::Focus),           max: 100 },
    Entry { label: "Health",      field: Field::Health,                        max: 100 },
    Entry { label: "Fulfillment", field: Field::Stat(StatId::Fulfillment),     max: 100 },
    Entry { label: "Cleanliness", field: Field::Stat(StatId::Cleanliness),     max: 100 },
    Entry { label: "Curiosity",   field: Field::Stat(StatId::Curiosity),       max: 100 },
    Entry { label: "Sociability", field: Field::Stat(StatId::Sociability),     max: 100 },
    Entry { label: "Intelligence",field: Field::Stat(StatId::Intelligence),    max: 100 },
    Entry { label: "Maturity",    field: Field::Stat(StatId::Maturity),        max: 100 },
    Entry { label: "Affection",   field: Field::Stat(StatId::Affection),       max: 100 },
    Entry { label: "Fitness",     field: Field::Stat(StatId::Fitness),         max: 100 },
    Entry { label: "Serenity",    field: Field::Stat(StatId::Serenity),        max: 100 },
    Entry { label: "Courage",     field: Field::Stat(StatId::Courage),         max: 100 },
    Entry { label: "Loyalty",     field: Field::Stat(StatId::Loyalty),         max: 100 },
    Entry { label: "Mischief",    field: Field::Stat(StatId::Mischievousness), max: 100 },
    Entry { label: "Sickness",    field: Field::Sickness,                      max: 10  },
];

fn read(ctx: &GameContext, field: Field) -> f32 {
    match field {
        Field::Stat(id) => *stat_ref(ctx, id),
        Field::Health => ctx.health,
        Field::Sickness => ctx.sickness,
    }
}

fn write(ctx: &mut GameContext, field: Field, value: f32) {
    match field {
        Field::Stat(id) => *stat_ref_mut(ctx, id) = value,
        Field::Health => ctx.health = value,
        Field::Sickness => ctx.sickness = value,
    }
}

fn stat_ref(ctx: &GameContext, id: StatId) -> &f32 {
    match id {
        StatId::Fullness => &ctx.fullness,
        StatId::Energy => &ctx.energy,
        StatId::Comfort => &ctx.comfort,
        StatId::Playfulness => &ctx.playfulness,
        StatId::Focus => &ctx.focus,
        StatId::Fulfillment => &ctx.fulfillment,
        StatId::Cleanliness => &ctx.cleanliness,
        StatId::Intelligence => &ctx.intelligence,
        StatId::Maturity => &ctx.maturity,
        StatId::Affection => &ctx.affection,
        StatId::Fitness => &ctx.fitness,
        StatId::Serenity => &ctx.serenity,
        StatId::Courage => &ctx.courage,
        StatId::Loyalty => &ctx.loyalty,
        StatId::Mischievousness => &ctx.mischievousness,
        StatId::Curiosity => &ctx.curiosity,
        StatId::Sociability => &ctx.sociability,
    }
}

fn stat_ref_mut(ctx: &mut GameContext, id: StatId) -> &mut f32 {
    match id {
        StatId::Fullness => &mut ctx.fullness,
        StatId::Energy => &mut ctx.energy,
        StatId::Comfort => &mut ctx.comfort,
        StatId::Playfulness => &mut ctx.playfulness,
        StatId::Focus => &mut ctx.focus,
        StatId::Fulfillment => &mut ctx.fulfillment,
        StatId::Cleanliness => &mut ctx.cleanliness,
        StatId::Intelligence => &mut ctx.intelligence,
        StatId::Maturity => &mut ctx.maturity,
        StatId::Affection => &mut ctx.affection,
        StatId::Fitness => &mut ctx.fitness,
        StatId::Serenity => &mut ctx.serenity,
        StatId::Courage => &mut ctx.courage,
        StatId::Loyalty => &mut ctx.loyalty,
        StatId::Mischievousness => &mut ctx.mischievousness,
        StatId::Curiosity => &mut ctx.curiosity,
        StatId::Sociability => &mut ctx.sociability,
    }
}

pub struct DebugStatsScene {
    settings: Settings,
}

impl DebugStatsScene {
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
        }
    }
}

impl Scene for DebugStatsScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        let mut items: heapless::Vec<SettingItem, { crate::ui::settings::MAX_ITEMS }> =
            heapless::Vec::new();
        for entry in ENTRIES {
            let cur = read(ctx, entry.field) as i32;
            let _ = items.push(SettingItem::int(entry.label, cur, 0, entry.max, 1));
        }
        self.settings.open(&items);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        match self.settings.handle_input(buttons) {
            SettingsResult::Continue | SettingsResult::Activated(_) => None,
            SettingsResult::Closed => {
                for (i, entry) in ENTRIES.iter().enumerate() {
                    if let Some(SettingValue::Int { value, .. }) = self.settings.value(i) {
                        write(ctx, entry.field, *value as f32);
                    }
                }
                Some(ctx.last_main_scene)
            }
        }
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.settings.draw(renderer);
    }
}
