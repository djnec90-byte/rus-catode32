pub mod adoption_scene;
pub mod bedroom_scene;
pub mod breakout_scene;
pub mod credits_scene;
pub mod debug_behaviors_scene;
pub mod debug_context_scene;
pub mod debug_env_scene;
pub mod debug_espnow_scene;
pub mod debug_led_scene;
pub mod debug_plants_scene;
pub mod debug_power_scene;
pub mod debug_stats_scene;
pub mod debug_time_scene;
pub mod debug_wifi_scene;
pub mod forecast_scene;
pub mod hanjie_scene;
pub mod inside_scene;
pub mod kitchen_scene;
pub mod lightsout_scene;
pub mod maze_scene;
pub mod memory_scene;
pub mod menu_scene;
pub mod outside_scene;
pub mod pet_info_scene;
pub mod pipes_scene;
pub mod platformer_scene;
pub mod pose_scene;
pub mod snake_scene;
pub mod social_scene;
pub mod stats_scene;
pub mod store_scene;
pub mod stub_scene;
pub mod tictactoe_scene;
pub mod treehouse_scene;
pub mod vacation_aquarium_scene;
pub mod vacation_base;
pub mod vacation_beach_scene;
pub mod vacation_forest_scene;
pub mod vacation_park_scene;
pub mod zoomies_scene;

use crate::scene::{Scene, SceneId};

use adoption_scene::AdoptionScene;
use bedroom_scene::BedroomScene;
use breakout_scene::BreakoutScene;
use credits_scene::CreditsScene;
use debug_behaviors_scene::DebugBehaviorsScene;
use debug_context_scene::DebugContextScene;
use debug_env_scene::DebugEnvScene;
use debug_espnow_scene::DebugEspnowScene;
use debug_led_scene::DebugLedScene;
use debug_plants_scene::DebugPlantsScene;
use debug_power_scene::DebugPowerScene;
use debug_stats_scene::DebugStatsScene;
use debug_time_scene::DebugTimeScene;
use debug_wifi_scene::DebugWifiScene;
use forecast_scene::ForecastScene;
use hanjie_scene::HanjieScene;
use inside_scene::InsideScene;
use kitchen_scene::KitchenScene;
use lightsout_scene::LightsOutScene;
use maze_scene::MazeScene;
use memory_scene::MemoryScene;
use menu_scene::MenuScene;
use outside_scene::OutsideScene;
use pet_info_scene::PetInfoScene;
use pipes_scene::PipesScene;
use platformer_scene::PlatformerScene;
use pose_scene::PoseScene;
use snake_scene::SnakeScene;
use social_scene::SocialScene;
use stats_scene::StatsScene;
use store_scene::StoreScene;
use stub_scene::StubScene;
use tictactoe_scene::TicTacToeScene;
use treehouse_scene::TreehouseScene;
use vacation_aquarium_scene::VacationAquariumScene;
use vacation_beach_scene::VacationBeachScene;
use vacation_forest_scene::VacationForestScene;
use vacation_park_scene::VacationParkScene;
use zoomies_scene::ZoomiesScene;

pub enum ActiveScene {
    Inside(InsideScene),
    Outside(OutsideScene),
    Bedroom(BedroomScene),
    Kitchen(KitchenScene),
    Treehouse(TreehouseScene),
    Menu(MenuScene),
    PoseViewer(PoseScene),
    Stats(StatsScene),
    Forecast(ForecastScene),
    Store(StoreScene),
    Adoption(AdoptionScene),
    PetInfo(PetInfoScene),
    Credits(CreditsScene),
    DebugBehaviors(DebugBehaviorsScene),
    DebugContext(DebugContextScene),
    DebugEnv(DebugEnvScene),
    DebugLed(DebugLedScene),
    DebugPlants(DebugPlantsScene),
    DebugPower(DebugPowerScene),
    DebugStats(DebugStatsScene),
    DebugTime(DebugTimeScene),
    DebugWifi(DebugWifiScene),
    DebugEspnow(DebugEspnowScene),
    Social(SocialScene),
    Zoomies(ZoomiesScene),
    Breakout(BreakoutScene),
    Snake(SnakeScene),
    Memory(MemoryScene),
    Maze(MazeScene),
    Hanjie(HanjieScene),
    TicTacToe(TicTacToeScene),
    LightsOut(LightsOutScene),
    Pipes(PipesScene),
    Platformer(PlatformerScene),
    VacationPark(VacationParkScene),
    VacationForest(VacationForestScene),
    VacationAquarium(VacationAquariumScene),
    VacationBeach(VacationBeachScene),
    Stub(StubScene),
}

impl ActiveScene {
    pub fn from_id(id: SceneId) -> Self {
        match id {
            SceneId::Inside => ActiveScene::Inside(InsideScene::new()),
            SceneId::Outside => ActiveScene::Outside(OutsideScene::new()),
            SceneId::Bedroom => ActiveScene::Bedroom(BedroomScene::new()),
            SceneId::Kitchen => ActiveScene::Kitchen(KitchenScene::new()),
            SceneId::Treehouse => ActiveScene::Treehouse(TreehouseScene::new()),
            SceneId::Menu => ActiveScene::Menu(MenuScene::new()),
            SceneId::PoseViewer => ActiveScene::PoseViewer(PoseScene::new()),
            SceneId::Stats => ActiveScene::Stats(StatsScene::new()),
            SceneId::Forecast => ActiveScene::Forecast(ForecastScene::new()),
            SceneId::Store => ActiveScene::Store(StoreScene::new()),
            SceneId::Adoption => ActiveScene::Adoption(AdoptionScene::new()),
            SceneId::PetInfo => ActiveScene::PetInfo(PetInfoScene::new()),
            SceneId::Credits => ActiveScene::Credits(CreditsScene::new()),
            SceneId::DebugBehaviors => ActiveScene::DebugBehaviors(DebugBehaviorsScene::new()),
            SceneId::DebugContext => ActiveScene::DebugContext(DebugContextScene::new()),
            SceneId::DebugEnv => ActiveScene::DebugEnv(DebugEnvScene::new()),
            SceneId::DebugLed => ActiveScene::DebugLed(DebugLedScene::new()),
            SceneId::DebugPlants => ActiveScene::DebugPlants(DebugPlantsScene::new()),
            SceneId::DebugPower => ActiveScene::DebugPower(DebugPowerScene::new()),
            SceneId::DebugStats => ActiveScene::DebugStats(DebugStatsScene::new()),
            SceneId::DebugTime => ActiveScene::DebugTime(DebugTimeScene::new()),
            SceneId::DebugWifi => ActiveScene::DebugWifi(DebugWifiScene::new()),
            SceneId::DebugEspnow => ActiveScene::DebugEspnow(DebugEspnowScene::new()),
            SceneId::Social => ActiveScene::Social(SocialScene::new()),
            SceneId::Zoomies => ActiveScene::Zoomies(ZoomiesScene::new()),
            SceneId::Breakout => ActiveScene::Breakout(BreakoutScene::new()),
            SceneId::Snake => ActiveScene::Snake(SnakeScene::new()),
            SceneId::Memory => ActiveScene::Memory(MemoryScene::new()),
            SceneId::Maze => ActiveScene::Maze(MazeScene::new()),
            SceneId::Hanjie => ActiveScene::Hanjie(HanjieScene::new()),
            SceneId::TicTacToe => ActiveScene::TicTacToe(TicTacToeScene::new()),
            SceneId::LightsOut => ActiveScene::LightsOut(LightsOutScene::new()),
            SceneId::Pipes => ActiveScene::Pipes(PipesScene::new()),
            SceneId::Platformer => ActiveScene::Platformer(PlatformerScene::new()),
            SceneId::VacationPark => ActiveScene::VacationPark(VacationParkScene::new()),
            SceneId::VacationForest => ActiveScene::VacationForest(VacationForestScene::new()),
            SceneId::VacationAquarium => {
                ActiveScene::VacationAquarium(VacationAquariumScene::new())
            }
            SceneId::VacationBeach => ActiveScene::VacationBeach(VacationBeachScene::new()),
            SceneId::Stub(name) => ActiveScene::Stub(StubScene::new(name)),
        }
    }

    pub fn as_scene_mut(&mut self) -> &mut dyn Scene {
        match self {
            ActiveScene::Inside(s) => s,
            ActiveScene::Outside(s) => s,
            ActiveScene::Bedroom(s) => s,
            ActiveScene::Kitchen(s) => s,
            ActiveScene::Treehouse(s) => s,
            ActiveScene::Menu(s) => s,
            ActiveScene::PoseViewer(s) => s,
            ActiveScene::Stats(s) => s,
            ActiveScene::Forecast(s) => s,
            ActiveScene::Store(s) => s,
            ActiveScene::Adoption(s) => s,
            ActiveScene::PetInfo(s) => s,
            ActiveScene::Credits(s) => s,
            ActiveScene::DebugBehaviors(s) => s,
            ActiveScene::DebugContext(s) => s,
            ActiveScene::DebugEnv(s) => s,
            ActiveScene::DebugLed(s) => s,
            ActiveScene::DebugPlants(s) => s,
            ActiveScene::DebugPower(s) => s,
            ActiveScene::DebugStats(s) => s,
            ActiveScene::DebugTime(s) => s,
            ActiveScene::DebugWifi(s) => s,
            ActiveScene::DebugEspnow(s) => s,
            ActiveScene::Social(s) => s,
            ActiveScene::Zoomies(s) => s,
            ActiveScene::Breakout(s) => s,
            ActiveScene::Snake(s) => s,
            ActiveScene::Memory(s) => s,
            ActiveScene::Maze(s) => s,
            ActiveScene::Hanjie(s) => s,
            ActiveScene::TicTacToe(s) => s,
            ActiveScene::LightsOut(s) => s,
            ActiveScene::Pipes(s) => s,
            ActiveScene::Platformer(s) => s,
            ActiveScene::VacationPark(s) => s,
            ActiveScene::VacationForest(s) => s,
            ActiveScene::VacationAquarium(s) => s,
            ActiveScene::VacationBeach(s) => s,
            ActiveScene::Stub(s) => s,
        }
    }

    pub fn as_scene(&self) -> &dyn Scene {
        match self {
            ActiveScene::Inside(s) => s,
            ActiveScene::Outside(s) => s,
            ActiveScene::Bedroom(s) => s,
            ActiveScene::Kitchen(s) => s,
            ActiveScene::Treehouse(s) => s,
            ActiveScene::Menu(s) => s,
            ActiveScene::PoseViewer(s) => s,
            ActiveScene::Stats(s) => s,
            ActiveScene::Forecast(s) => s,
            ActiveScene::Store(s) => s,
            ActiveScene::Adoption(s) => s,
            ActiveScene::PetInfo(s) => s,
            ActiveScene::Credits(s) => s,
            ActiveScene::DebugBehaviors(s) => s,
            ActiveScene::DebugContext(s) => s,
            ActiveScene::DebugEnv(s) => s,
            ActiveScene::DebugLed(s) => s,
            ActiveScene::DebugPlants(s) => s,
            ActiveScene::DebugPower(s) => s,
            ActiveScene::DebugStats(s) => s,
            ActiveScene::DebugTime(s) => s,
            ActiveScene::DebugWifi(s) => s,
            ActiveScene::DebugEspnow(s) => s,
            ActiveScene::Social(s) => s,
            ActiveScene::Zoomies(s) => s,
            ActiveScene::Breakout(s) => s,
            ActiveScene::Snake(s) => s,
            ActiveScene::Memory(s) => s,
            ActiveScene::Maze(s) => s,
            ActiveScene::Hanjie(s) => s,
            ActiveScene::TicTacToe(s) => s,
            ActiveScene::LightsOut(s) => s,
            ActiveScene::Pipes(s) => s,
            ActiveScene::Platformer(s) => s,
            ActiveScene::VacationPark(s) => s,
            ActiveScene::VacationForest(s) => s,
            ActiveScene::VacationAquarium(s) => s,
            ActiveScene::VacationBeach(s) => s,
            ActiveScene::Stub(s) => s,
        }
    }
}
