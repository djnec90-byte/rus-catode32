pub mod adoption_scene;
pub mod bedroom_scene;
pub mod forecast_scene;
pub mod inside_scene;
pub mod kitchen_scene;
pub mod menu_scene;
pub mod outside_scene;
pub mod pet_info_scene;
pub mod pose_scene;
pub mod stats_scene;
pub mod store_scene;
pub mod stub_scene;
pub mod treehouse_scene;

use crate::scene::{Scene, SceneId};

use adoption_scene::AdoptionScene;
use bedroom_scene::BedroomScene;
use forecast_scene::ForecastScene;
use inside_scene::InsideScene;
use kitchen_scene::KitchenScene;
use menu_scene::MenuScene;
use outside_scene::OutsideScene;
use pet_info_scene::PetInfoScene;
use pose_scene::PoseScene;
use stats_scene::StatsScene;
use store_scene::StoreScene;
use stub_scene::StubScene;
use treehouse_scene::TreehouseScene;

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
            ActiveScene::Stub(s) => s,
        }
    }
}
