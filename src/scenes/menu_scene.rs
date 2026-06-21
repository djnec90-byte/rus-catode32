use crate::{
    assets::icons,
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scene::{Scene, SceneId},
    ui::menu::{Menu, MenuAction, MenuItem, MenuResult},
};

const LOCATIONS: &[MenuItem] = &[
    MenuItem { label: "Living Room", icon: Some(icons::HOUSE), submenu: None, action: Some(MenuAction::Scene(SceneId::Inside)), confirm: None },
    MenuItem { label: "Bedroom",     icon: Some(icons::HOUSE), submenu: None, action: Some(MenuAction::Scene(SceneId::Bedroom)),          confirm: None },
    MenuItem { label: "Kitchen",     icon: Some(icons::MEAL),  submenu: None, action: Some(MenuAction::Scene(SceneId::Kitchen)),          confirm: None },
    MenuItem { label: "Outside",     icon: Some(icons::SUN),   submenu: None, action: Some(MenuAction::Scene(SceneId::Outside)),         confirm: None },
    MenuItem { label: "Treehouse",   icon: Some(icons::TREES), submenu: None, action: Some(MenuAction::Scene(SceneId::Treehouse)),        confirm: None },
];

const MINIGAMES: &[MenuItem] = &[
    MenuItem { label: "Zoomies",   icon: Some(icons::ZOOMIES),    submenu: None, action: Some(MenuAction::Scene(SceneId::Zoomies)),         confirm: None },
    MenuItem { label: "Breakout",  icon: Some(icons::BREAKOUT),   submenu: None, action: Some(MenuAction::Scene(SceneId::Breakout)),         confirm: None },
    MenuItem { label: "Snake",     icon: Some(icons::SNAKE),      submenu: None, action: Some(MenuAction::Scene(SceneId::Snake)),             confirm: None },
    MenuItem { label: "Hunter",    icon: Some(icons::PLATFORMER), submenu: None, action: Some(MenuAction::Scene(SceneId::Platformer)),         confirm: None },
    MenuItem { label: "Memory",    icon: Some(icons::MEMORY),     submenu: None, action: Some(MenuAction::Scene(SceneId::Memory)),             confirm: None },
    MenuItem { label: "Maze",      icon: Some(icons::MAZE),       submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("maze"))),      confirm: None },
    MenuItem { label: "TicTacToe", icon: Some(icons::TICTACTOE),  submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("tictactoe"))), confirm: None },
    MenuItem { label: "Hanjie",    icon: Some(icons::HANJIE),     submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("hanjie"))),    confirm: None },
    MenuItem { label: "Lights Out", icon: Some(icons::POWER),     submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("lightsout"))), confirm: None },
    MenuItem { label: "Pipes",     icon: Some(icons::PLUMBING),   submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("pipes"))),     confirm: None },
];

const VACATIONS: &[MenuItem] = &[
    MenuItem { label: "Park",     icon: Some(icons::TREES), submenu: None, action: Some(MenuAction::Scene(SceneId::VacationPark)),     confirm: None },
    MenuItem { label: "Forest",   icon: Some(icons::TREES), submenu: None, action: Some(MenuAction::Scene(SceneId::VacationForest)),   confirm: None },
    MenuItem { label: "Aquarium", icon: Some(icons::FISH),  submenu: None, action: Some(MenuAction::Scene(SceneId::VacationAquarium)), confirm: None },
    MenuItem { label: "Beach",    icon: Some(icons::SUN),   submenu: None, action: Some(MenuAction::Scene(SceneId::VacationBeach)),    confirm: None },
];

const WIRELESS: &[MenuItem] = &[
    MenuItem { label: "Wifi",    icon: Some(icons::WIFI), submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("debug_wifi"))),   confirm: None },
    MenuItem { label: "ESP-NOW", icon: Some(icons::WIFI), submenu: None, action: Some(MenuAction::Scene(SceneId::Stub("debug_espnow"))), confirm: None },
];

const DEBUG: &[MenuItem] = &[
    MenuItem { label: "Environment", icon: Some(icons::SUN),    submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugEnv)),                     confirm: None },
    MenuItem { label: "Poses",       icon: Some(icons::CAT),    submenu: None,             action: Some(MenuAction::Scene(SceneId::PoseViewer)),                   confirm: None },
    MenuItem { label: "Behaviors",   icon: Some(icons::CAT),    submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugBehaviors)),               confirm: None },
    MenuItem { label: "Stats",       icon: Some(icons::CAT),    submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugStats)),                   confirm: None },
    MenuItem { label: "Plants",      icon: Some(icons::TREES),  submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugPlants)),                  confirm: None },
    MenuItem { label: "Vacations",   icon: Some(icons::SUN),    submenu: Some(VACATIONS),  action: None,                                                           confirm: None },
    MenuItem { label: "Time Speed",  icon: Some(icons::WRENCH), submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugTime)),                    confirm: None },
    MenuItem { label: "RGB LED",     icon: Some(icons::WRENCH), submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugLed)),                     confirm: None },
    MenuItem { label: "Power",       icon: Some(icons::POWER),  submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugPower)),                   confirm: None },
    MenuItem { label: "Context",     icon: Some(icons::WRENCH), submenu: None,             action: Some(MenuAction::Scene(SceneId::DebugContext)),                 confirm: None },
    MenuItem { label: "Wireless",    icon: Some(icons::WIFI),   submenu: Some(WIRELESS),   action: None,                                                           confirm: None },
];

// TODO: Python filters items based on config.WIFI_ENABLED (hides Social) and
// config.SHOW_DEBUG_MENUS (hides Debug). We always show everything for now.
// TODO: Python wraps every label in `t()` for i18n; strings are hardcoded English here.
const BIG_MENU: &[MenuItem] = &[
    MenuItem { label: "Pet stats",  icon: Some(icons::STATS),     submenu: None,             action: Some(MenuAction::Scene(SceneId::Stats)),            confirm: None },
    MenuItem { label: "Locations",  icon: Some(icons::HOUSE),     submenu: Some(LOCATIONS),  action: None,                                                confirm: None },
    MenuItem { label: "Forecast",   icon: Some(icons::SUN),       submenu: None,             action: Some(MenuAction::Scene(SceneId::Forecast)),         confirm: None },
    MenuItem { label: "Minigames",  icon: Some(icons::MINIGAMES), submenu: Some(MINIGAMES),  action: None,                                                confirm: None },
    MenuItem { label: "Store",      icon: Some(icons::STORE),     submenu: None,             action: Some(MenuAction::Scene(SceneId::Store)),            confirm: None },
    MenuItem { label: "Social",     icon: Some(icons::CAT),       submenu: None,             action: Some(MenuAction::Scene(SceneId::Stub("social"))),   confirm: None },
    MenuItem { label: "Pet info",   icon: Some(icons::CAT),       submenu: None,             action: Some(MenuAction::Scene(SceneId::PetInfo)),          confirm: None },
    MenuItem { label: "Debug",      icon: Some(icons::WRENCH),    submenu: Some(DEBUG),      action: None,                                                confirm: None },
    MenuItem { label: "Credits",    icon: Some(icons::CREDITS),   submenu: None,             action: Some(MenuAction::Scene(SceneId::Credits)),          confirm: None },
];

pub struct MenuScene {
    menu: Menu,
}

impl MenuScene {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(BIG_MENU),
        }
    }
}

impl Scene for MenuScene {
    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        match self.menu.handle_input(buttons) {
            MenuResult::Continue => None,
            MenuResult::Closed => Some(ctx.last_main_scene),
            MenuResult::Action(MenuAction::Scene(id)) => Some(id),
            // The main menu doesn't surface Store-purchase actions.
            MenuResult::Action(MenuAction::Store(_)) => None,
        }
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.menu.draw(renderer);
    }
}
