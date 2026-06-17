pub mod main_scene;
pub mod menu_scene;
pub mod pose_scene;

use crate::scene::{Scene, SceneId};

use main_scene::MainScene;
use menu_scene::MenuScene;
use pose_scene::PoseScene;

pub enum ActiveScene {
    Main(MainScene),
    Menu(MenuScene),
    PoseViewer(PoseScene),
}

impl ActiveScene {
    pub fn from_id(id: SceneId) -> Self {
        match id {
            SceneId::Main => ActiveScene::Main(MainScene::new()),
            SceneId::Menu => ActiveScene::Menu(MenuScene::new()),
            SceneId::PoseViewer => ActiveScene::PoseViewer(PoseScene::new()),
        }
    }

    pub fn as_scene_mut(&mut self) -> &mut dyn Scene {
        match self {
            ActiveScene::Main(s) => s,
            ActiveScene::Menu(s) => s,
            ActiveScene::PoseViewer(s) => s,
        }
    }

    pub fn as_scene(&self) -> &dyn Scene {
        match self {
            ActiveScene::Main(s) => s,
            ActiveScene::Menu(s) => s,
            ActiveScene::PoseViewer(s) => s,
        }
    }
}
