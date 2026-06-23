use crate::{
    assets::icons,
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scene::{Scene, SceneId},
    ui::menu::{Menu, MenuItem, MenuResult},
};

const END_VAC: Option<&'static str> = Some("End vacation?");

const LOCATIONS: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Living Room", icon: Some(icons::HOUSE), submenu: None, action: Some(SceneId::Inside),    confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Bedroom",     icon: Some(icons::HOUSE), submenu: None, action: Some(SceneId::Bedroom),   confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Kitchen",     icon: Some(icons::MEAL),  submenu: None, action: Some(SceneId::Kitchen),   confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Outside",     icon: Some(icons::SUN),   submenu: None, action: Some(SceneId::Outside),   confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Treehouse",   icon: Some(icons::TREES), submenu: None, action: Some(SceneId::Treehouse), confirm: None, confirm_on_vacation: END_VAC },
];

const MINIGAMES: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Zoomies",   icon: Some(icons::ZOOMIES),    submenu: None, action: Some(SceneId::Zoomies),    confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Breakout",  icon: Some(icons::BREAKOUT),   submenu: None, action: Some(SceneId::Breakout),   confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Snake",     icon: Some(icons::SNAKE),      submenu: None, action: Some(SceneId::Snake),      confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Hunter",    icon: Some(icons::PLATFORMER), submenu: None, action: Some(SceneId::Platformer), confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Memory",    icon: Some(icons::MEMORY),     submenu: None, action: Some(SceneId::Memory),     confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Maze",      icon: Some(icons::MAZE),       submenu: None, action: Some(SceneId::Maze),       confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "TicTacToe", icon: Some(icons::TICTACTOE),  submenu: None, action: Some(SceneId::TicTacToe),  confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Hanjie",    icon: Some(icons::HANJIE),     submenu: None, action: Some(SceneId::Hanjie),     confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Lights Out", icon: Some(icons::POWER),     submenu: None, action: Some(SceneId::LightsOut),  confirm: None, confirm_on_vacation: END_VAC },
    MenuItem { label: "Pipes",     icon: Some(icons::PLUMBING),   submenu: None, action: Some(SceneId::Pipes),      confirm: None, confirm_on_vacation: END_VAC },
];

const VACATIONS: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Park",     icon: Some(icons::TREES), submenu: None, action: Some(SceneId::VacationPark),     confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Forest",   icon: Some(icons::TREES), submenu: None, action: Some(SceneId::VacationForest),   confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Aquarium", icon: Some(icons::FISH),  submenu: None, action: Some(SceneId::VacationAquarium), confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Beach",    icon: Some(icons::SUN),   submenu: None, action: Some(SceneId::VacationBeach),    confirm: None, confirm_on_vacation: None },
];

const WIRELESS: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Wifi",    icon: Some(icons::WIFI), submenu: None, action: Some(SceneId::DebugWifi),            confirm: None, confirm_on_vacation: None },
    MenuItem { label: "ESP-NOW", icon: Some(icons::WIFI), submenu: None, action: Some(SceneId::DebugEspnow), confirm: None, confirm_on_vacation: None },
];

const DEBUG: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Environment", icon: Some(icons::SUN),    submenu: None,             action: Some(SceneId::DebugEnv),       confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Poses",       icon: Some(icons::CAT),    submenu: None,             action: Some(SceneId::PoseViewer),     confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Behaviors",   icon: Some(icons::CAT),    submenu: None,             action: Some(SceneId::DebugBehaviors), confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Stats",       icon: Some(icons::CAT),    submenu: None,             action: Some(SceneId::DebugStats),     confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Plants",      icon: Some(icons::TREES),  submenu: None,             action: Some(SceneId::DebugPlants),    confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Vacations",   icon: Some(icons::SUN),    submenu: Some(VACATIONS),  action: None,                          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Time Speed",  icon: Some(icons::WRENCH), submenu: None,             action: Some(SceneId::DebugTime),      confirm: None, confirm_on_vacation: None },
    MenuItem { label: "RGB LED",     icon: Some(icons::WRENCH), submenu: None,             action: Some(SceneId::DebugLed),       confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Power",       icon: Some(icons::POWER),  submenu: None,             action: Some(SceneId::DebugPower),     confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Context",     icon: Some(icons::WRENCH), submenu: None,             action: Some(SceneId::DebugContext),   confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Wireless",    icon: Some(icons::WIFI),   submenu: Some(WIRELESS),   action: None,                          confirm: None, confirm_on_vacation: None },
];

// TODO: Filter items based on config.WIFI_ENABLED (hides Social) and
// config.SHOW_DEBUG_MENUS (hides Debug). We always show everything for now.
// TODO: Wrap every label in `t()` for i18n. Strings are hardcoded English here.
const BIG_MENU: &[MenuItem<SceneId>] = &[
    MenuItem { label: "Pet stats",  icon: Some(icons::STATS),     submenu: None,             action: Some(SceneId::Stats),          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Locations",  icon: Some(icons::HOUSE),     submenu: Some(LOCATIONS),  action: None,                          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Forecast",   icon: Some(icons::SUN),       submenu: None,             action: Some(SceneId::Forecast),       confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Minigames",  icon: Some(icons::MINIGAMES), submenu: Some(MINIGAMES),  action: None,                          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Store",      icon: Some(icons::STORE),     submenu: None,             action: Some(SceneId::Store),          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Social",     icon: Some(icons::CAT),       submenu: None,             action: Some(SceneId::Social),         confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Pet info",   icon: Some(icons::CAT),       submenu: None,             action: Some(SceneId::PetInfo),        confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Debug",      icon: Some(icons::WRENCH),    submenu: Some(DEBUG),      action: None,                          confirm: None, confirm_on_vacation: None },
    MenuItem { label: "Credits",    icon: Some(icons::CREDITS),   submenu: None,             action: Some(SceneId::Credits),        confirm: None, confirm_on_vacation: None },
];

pub struct MenuScene {
    menu: Menu<SceneId>,
    /// Set on the frame the player dismissed the menu (Menu1/Menu2 or B at
    /// root). `SceneManager` reads this to close the overlay without firing
    /// a scene swap, so the player goes back to whatever was under the menu.
    dismissed: bool,
}

impl MenuScene {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(BIG_MENU),
            dismissed: false,
        }
    }

    pub fn take_dismissed(&mut self) -> bool {
        let was = self.dismissed;
        self.dismissed = false;
        was
    }
}

impl Scene for MenuScene {
    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        match self.menu.handle_input(buttons, ctx.on_vacation) {
            MenuResult::Continue => None,
            MenuResult::Closed => {
                self.dismissed = true;
                None
            }
            MenuResult::Action(id) => Some(id),
        }
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.menu.draw(renderer);
    }
}
