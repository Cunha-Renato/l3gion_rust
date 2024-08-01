const ASSETS_DIR: &str = "assets";

pub(crate) struct ImGuiAssetsPanel {
    pub(crate) selected_dir: String,
    pub(crate) selected_asset: String,
    
    assets_browser_padding: f32,
    assets_browser_thumbnail_size: f32,
}
impl ImGuiAssetsPanel {
    pub(crate) fn new() -> Self {
        Self {
            selected_dir: ASSETS_DIR.to_string(),
            selected_asset: String::default(),
            
            assets_browser_padding: 4.5,
            assets_browser_thumbnail_size: 110.0,
        }
    }

    pub(crate) fn imgui_assets_panel(&mut self, ui: &imgui::Ui) {
        ui.window("Assets Tree").build(|| {
            self.imgui_assets_tree_panel(ui, ASSETS_DIR);
        });

        ui.window("Assets Browser").build(|| {
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
        if let Some(table) = ui.begin_table_with_flags("Table", 1, imgui::TableFlags::BORDERS) {
            // Top row
            ui.table_next_row();
            ui.table_next_column();
            if ui.button("<--") {
                if &self.selected_dir != ASSETS_DIR {
                    let path = std::path::Path::new(&self.selected_dir);
                    
                    if let Some(parent) = path.parent() {
                        self.selected_dir = parent.to_string_lossy().to_string();
                        self.selected_asset.clear();
                    }
                }
                
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
            
            if let Some(table2) = ui.begin_table_with_flags("Assets Browser", column_count, imgui::TableFlags::SCROLL_Y) {
                ui.table_next_column();

                let entries = if let Ok(selected) = std::fs::read_dir(&self.selected_dir) {
                    selected
                }
                else {
                    std::fs::read_dir(ASSETS_DIR).unwrap()
                };
                
                for entry in entries {
                    let path = entry.unwrap().path();                        
                    
                    ui.button_with_size(path.to_string_lossy().to_string(), [self.assets_browser_thumbnail_size, self.assets_browser_thumbnail_size]);
                    
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

                table2.end();
            }

            /*for row in 0..2 {
                ui.table_next_row();
                for column in 0..1 {
                    ui.table_next_column();
                    let text = format!("Cell ({}, {})", row, column);
                    ui.text(&text);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(&format!("Hovered over {}", text));
                    }
                }
            }*/
            
            table.end();
        }
    }
}