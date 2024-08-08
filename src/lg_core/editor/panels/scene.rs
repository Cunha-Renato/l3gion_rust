use crate::lg_core::entity::LgEntity;

pub(crate) fn imgui_scene_panel(ui: &mut imgui::Ui, entities: &[LgEntity]) {
    ui.window("Scene").build(|| {
        for e in entities {
            entity_node(ui, e);
        }
    });
}

fn entity_node(ui: &imgui::Ui, entity: &LgEntity) {
    let flags = imgui::TreeNodeFlags::OPEN_ON_ARROW
    | imgui::TreeNodeFlags::OPEN_ON_ARROW
    | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
    | imgui::TreeNodeFlags::FRAMED
    | imgui::TreeNodeFlags::FRAME_PADDING
    | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH;
    
    ui.tree_node_config(entity.name())
        .flags(flags)
        .build(|| {

        });
}