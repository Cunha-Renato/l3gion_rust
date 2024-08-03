use std::sync::{Arc, Mutex};

use crate::lg_core::{asset_manager::AssetManager, renderer::texture::TextureSpecs, uuid::UUID};

const ASSETS_DIR: &str = "assets";
const RESOURCES_DIR: &str = "src/lg_core/editor/resources/";

pub(crate) struct ImGuiAssetsPanel {
    pub(crate) selected_dir: String,
    pub(crate) selected_asset: String,
    
    assets_browser_padding: f32,
    assets_browser_thumbnail_size: f32,

    asset_manager: Option<Arc<Mutex<AssetManager>>>,
    dir_icon: String,
    file_icon: String,
}
impl ImGuiAssetsPanel {
    pub(crate) fn new() -> Self {
        let dir_icon = std::format!("{}/icons/assets_panel/dir_icon.png", RESOURCES_DIR);
        let file_icon = std::format!("{}/icons/assets_panel/file_icon.png", RESOURCES_DIR);

        Self {
            selected_dir: ASSETS_DIR.to_string(),
            selected_asset: String::default(),
            
            assets_browser_padding: 4.5,
            assets_browser_thumbnail_size: 110.0,
            
            asset_manager: None,
            dir_icon,
            file_icon,
        }
    }
    
    pub(crate) fn init(&mut self, am: Arc<Mutex<AssetManager>>) {
        {
            let mut am = am.lock().unwrap();
            let specs = TextureSpecs {
                tex_format: crate::lg_core::renderer::texture::TextureFormat::RGBA,
                ..Default::default()
            };
            let _ = am.create_texture("dir_icon", &self.dir_icon, specs.clone());
            let _ = am.create_texture("file_icon", &self.file_icon, specs);
        }
        
        self.asset_manager = Some(am);
    }

    pub(crate) fn imgui_assets_panel(&mut self, ui: &imgui::Ui) {
        ui.window("Assets Tree").build(|| {
            self.imgui_assets_tree_panel(ui, ASSETS_DIR);
        });

        ui.window("Assets Browser")
            .scrollable(false)
            .scroll_bar(false)
            .build(|| {
                self.imgui_assets_browser_panel(ui);
            });
    }

    fn imgui_assets_tree_panel(&mut self, ui: &imgui::Ui, path: &str) {
        let entries = if let Ok(entries) = std::fs::read_dir(path) {
            entries
        }
        else { return; };

        for entry in entries {
            let path = entry.unwrap().path();                

            let flags = if path.is_dir() {
                imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
                | imgui::TreeNodeFlags::OPEN_ON_ARROW
                | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
                | imgui::TreeNodeFlags::SELECTED
            }
            else {
                imgui::TreeNodeFlags::LEAF
                | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
            };

            ui.tree_node_config(path.file_name().unwrap().to_str().unwrap())
                .flags(flags)
                .build(|| {
                    self.imgui_assets_tree_panel(ui, path.to_str().unwrap());
                });
            
            if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                if !path.is_dir() {
                    if let Some(parent) = path.parent() {
                        self.selected_dir = parent.to_string_lossy().to_string();
                    }
                    self.selected_asset = path.to_string_lossy().to_string();
                }
            }
        }                
    }

    fn imgui_assets_browser_panel(&mut self, ui: &imgui::Ui) {
        if let Some(table) = ui.begin_table_with_flags(
            "Parent Table", 
            1, 
            imgui::TableFlags::empty()
        ) {
            // Top row
            ui.table_next_row();
            ui.table_next_column();
            if &self.selected_dir != ASSETS_DIR {
                let back_btn_content = std::path::Path::new(&self.selected_dir)
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                if ui.button(&back_btn_content) {
                    self.selected_dir = back_btn_content;
                    self.selected_asset.clear();
                }
            }
            else {
                ui.button("../");
            }
            ui.same_line();
            ui.text(std::format!("{}", self.selected_dir));
            ui.same_line();
            ui.text(std::format!("{}", self.selected_asset));

            // Bottom row
            ui.table_next_row();
            ui.table_next_column();

            let cell_size = self.assets_browser_thumbnail_size + self.assets_browser_padding;
            let panel_width = ui.current_column_width();
            let mut column_count = (panel_width / cell_size) as usize;
            if column_count < 1 { column_count = 1; }
            
            // Thumbnail Table
            if let Some(table2) = ui.begin_table_with_flags(
                "Assets Browser", 
                column_count, 
                imgui::TableFlags::SCROLL_Y
            ) {
                ui.table_next_column();

                let entries = if let Ok(selected) = std::fs::read_dir(&self.selected_dir) {
                    selected
                }
                else {
                    std::fs::read_dir(ASSETS_DIR).unwrap()
                };

                let mut am = self.asset_manager.as_ref().unwrap().lock().unwrap();
                let (gl_dir_icon, gl_file_icon) = unsafe {
                    (
                        am.get_texture(&UUID::from_string(&self.dir_icon).unwrap())
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .gl_id(),
                        am.get_texture(&UUID::from_string(&self.file_icon).unwrap())
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .gl_id(),
                    )
                };

                for entry in entries {
                    let path = entry.unwrap().path();                        
                    let image_to_use = if path.is_dir() { gl_dir_icon } else { gl_file_icon };
                    
                    if let Some(tex) = image_to_use {
                        ui.image_button(path.to_string_lossy().to_string(), imgui::TextureId::new(tex as usize), [self.assets_browser_thumbnail_size, self.assets_browser_thumbnail_size]);
                        
                        if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                            if path.is_dir() {
                                self.selected_dir = path.to_string_lossy().to_string();
                                self.selected_asset.clear();
                            }
                            else {
                                self.selected_asset = path.to_string_lossy().to_string();
                            }
                        }

                        ui.text_wrapped(path.file_name().unwrap().to_string_lossy().to_string());
                        ui.table_next_column();
                    }
                }

                table2.end();
            } 

            table.end();
        }
    }
}