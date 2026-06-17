pub mod main_scene;
pub mod menu_scene;

use crate::scene::{Scene, SceneId};

use main_scene::MainScene;
use menu_scene::MenuScene;

pub enum ActiveScene {
    Main(MainScene),
    Menu(MenuScene),
}

impl ActiveScene {
    pub fn from_id(id: SceneId) -> Self {
        match id {
            SceneId::Main => ActiveScene::Main(MainScene::new()),
            SceneId::Menu => ActiveScene::Menu(MenuScene::new()),
        }
    }

    pub fn as_scene_mut(&mut self) -> &mut dyn Scene {
        match self {
            ActiveScene::Main(s) => s,
            ActiveScene::Menu(s) => s,
        }
    }

    pub fn as_scene(&self) -> &dyn Scene {
        match self {
            ActiveScene::Main(s) => s,
            ActiveScene::Menu(s) => s,
        }
    }
}
