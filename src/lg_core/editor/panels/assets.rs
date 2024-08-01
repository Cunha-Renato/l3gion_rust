const ASSETS_DIR: &str = "assets";

pub(crate) fn imgui_assets_tree(ui: &imgui::Ui) {
    let assets_window = ui.window("Assets").begin().unwrap();
    imgui_assets_tree_panel(ui, ASSETS_DIR);
    assets_window.end()
}

fn imgui_assets_tree_panel(ui: &imgui::Ui, path: &str) {
    let entries = std::fs::read_dir(path).unwrap();

    for entry in entries {
        let path = entry.unwrap().path();                

        if path.is_dir() {
            if let Some(tn) = ui.tree_node(path.file_name().unwrap().to_str().unwrap()) {
                imgui_assets_tree_panel(ui, path.to_str().unwrap());
                tn.pop();
            }
        }
        else {
            ui.text(path.file_name().unwrap().to_str().unwrap());
        }
    }                
}