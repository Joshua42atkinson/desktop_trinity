use crate::game::vaam::{
    CognitiveWeight, LessonNodeComponent, LogisticsCheckRequest, SemanticSocket, TrainComponent,
    VocabularyItem,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use iron_road_physics::{CognitiveLoad, VocabularyTier};

pub struct GlassUiPlugin;

impl Plugin for GlassUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, configure_glass_theme).add_systems(
            Update,
            (
                render_inventory_panel,
                render_train_panel,
                render_socket_panel,
            ),
        );
    }
}

/// Configures the Egui visuals for a "Glass" aesthetic: darker, translucent, modern.
fn configure_glass_theme(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    let mut visuals = egui::Visuals::dark();

    // Glassy Backgrounds
    visuals.window_fill = egui::Color32::from_rgba_premultiplied(20, 20, 30, 200);
    visuals.panel_fill = egui::Color32::from_rgba_premultiplied(20, 20, 30, 200);

    // Neonish Accents
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 180, 255); // Cyan
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 220, 255));

    // Rounder corners
    visuals.window_rounding = egui::Rounding::same(12.0);

    ctx.set_visuals(visuals);
}

/// Renders the VaaM Inventory Panel
/// Shows Vocabulary Items as "Cards"
fn render_inventory_panel(
    mut contexts: EguiContexts,
    query: Query<(Entity, &VocabularyItem, &CognitiveWeight)>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Vocabulary Inventory")
        .default_width(320.0)
        .anchor(egui::Align2::LEFT_TOP, [20.0, 20.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Active Vocabulary");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (entity, item, weight) in query.iter() {
                    // Drag Source
                    // We use the entity as the payload
                    let item_id = egui::Id::new(entity);

                    // Correct DnD check using standard interact:
                    let response = egui::Frame::none()
                        .show(ui, |ui| {
                            render_vocabulary_card(ui, item, weight);
                        })
                        .response;

                    // Make it draggable
                    let response = response.interact(egui::Sense::drag());

                    if response.drag_started() {
                        // Store payload in egui memory
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("dragged_vocab"), entity)
                        });
                    }
                }
            });
        });
}

/// Renders the VaaM Socket Panel (The "World" or "Context")
fn render_socket_panel(
    mut contexts: EguiContexts,
    mut socket_query: Query<(Entity, &SemanticSocket)>,
    mut events: EventWriter<LogisticsCheckRequest>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Semantic Context")
        .default_width(320.0)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Awaiting Input...");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (socket_entity, socket) in socket_query.iter_mut() {
                    render_socket_card(ui, socket, socket_entity, &mut events);
                }
            });
        });
}

fn render_socket_card(
    ui: &mut egui::Ui,
    socket: &SemanticSocket,
    socket_entity: Entity,
    events: &mut EventWriter<LogisticsCheckRequest>,
) {
    // 1. Render the Socket UI
    let response = egui::Frame::none()
        .fill(egui::Color32::from_rgba_premultiplied(30, 30, 40, 200))
        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
        .rounding(8.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Socket").strong());
                if socket.is_solved {
                    ui.label("✅ Solved");
                }
            });
            ui.label(format!("Requires: {:?}", socket.required_tags));
            ui.label(format!("DC: {}", socket.difficulty_class));
        })
        .response; // Get the outer response of the Frame

    // 2. Interaction Logic (Drop Zone)
    // Highlight if we are hovering and dragging something
    // (We don't know *what* we are dragging easily without checking memory, but visual feedback is good)

    // Check if anything is being dragged in the context
    if ui.memory(|mem| mem.is_anything_being_dragged()) {
        if response.hovered() {
            // Highlight
            ui.painter().rect_stroke(
                response.rect,
                egui::Rounding::same(8.0),
                egui::Stroke::new(2.0, egui::Color32::YELLOW),
            );
        }
    }

    // 3. Detect Drop
    // If pointer released while hovering this socket
    if response.hovered() && ui.input(|i| i.pointer.any_released()) {
        // Retrieve payload from memory
        // We accept that we might consume it multiple times if multiple sockets overlap?
        // But UI usually doesn't overlap in this layout.

        let dragged_entity_opt: Option<Entity> =
            ui.memory(|mem| mem.data.get_temp(egui::Id::new("dragged_vocab")));

        if let Some(vocab_entity) = dragged_entity_opt {
            info!(
                "📍 Dropped Entity {:?} onto Socket {:?}",
                vocab_entity, socket_entity
            );

            // Fire Event
            events.send(LogisticsCheckRequest {
                entity: vocab_entity,
                target_socket: Some(socket_entity),
            });
        }
    }
}

/// Renders the Train Physics Status (Steam, Coal, Velocity)
fn render_train_panel(
    mut contexts: EguiContexts,
    query: Query<&TrainComponent>,
    node_query: Query<&LessonNodeComponent>,
) {
    let ctx = contexts.ctx_mut();

    if let Ok(train_wrapper) = query.get_single() {
        let train = &train_wrapper.0;

        egui::Window::new("Iron Road Engine")
            // .intro_pattern(egui::TextureId::default(), egui::Rect::ZERO) // No pattern - removed invalid method
            .default_width(200.0)
            .anchor(egui::Align2::RIGHT_TOP, [-20.0, 20.0])
            .resizable(false)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.heading("Locomotive Stats");
                ui.separator();

                // Coal Meter (Motivation)
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("Coal (Motivation):");
                    ui.label(format!("{:.1}", train.coal));
                });
                let coal_progress = (train.coal / 100.0).clamp(0.0, 1.0);
                ui.add(
                    egui::ProgressBar::new(coal_progress)
                        .text("Fuel")
                        .fill(egui::Color32::from_rgb(50, 50, 50)),
                );

                ui.add_space(8.0);

                // Steam Meter (Germane Load)
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("Steam (Potential):");
                    ui.label(format!("{:.1}", train.steam));
                });
                // Arbitrary max steam for visualization
                let steam_progress = (train.steam / 50.0).clamp(0.0, 1.0);
                ui.add(
                    egui::ProgressBar::new(steam_progress)
                        .text("Pressure")
                        .fill(egui::Color32::WHITE),
                );

                ui.add_space(8.0);

                // Velocity
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("Velocity:");
                    ui.label(
                        egui::RichText::new(format!("{:.2} m/s", train.velocity))
                            .strong()
                            .size(20.0),
                    );
                });

                ui.add_space(8.0);

                // Friction (Extraneous Load) from current Node
                if let Ok(node_wrapper) = node_query.get_single() {
                    let node = &node_wrapper.0;
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.label("Friction (Drag):");
                        ui.label(format!("{:.1}", node.friction));
                    });
                    // Visualize friction (1.0 is low/good, 5.0 is high/bad)
                    let friction_progress = (node.friction / 5.0).clamp(0.0, 1.0);
                    ui.add(
                        egui::ProgressBar::new(friction_progress)
                            .text("Cognitive Drag")
                            .fill(if node.friction > 2.0 {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GREEN
                            }),
                    );
                }
            });
    }
}

fn render_vocabulary_card(ui: &mut egui::Ui, item: &VocabularyItem, weight: &CognitiveWeight) {
    // Card Style
    let card_color = match item.tier {
        VocabularyTier::Basic => egui::Color32::from_rgba_premultiplied(40, 200, 40, 50), // Greenish
        VocabularyTier::Academic => egui::Color32::from_rgba_premultiplied(40, 40, 200, 50), // Blueish
        VocabularyTier::Hazardous => egui::Color32::from_rgba_premultiplied(200, 40, 40, 50), // Reddish
    };

    let stroke_color = match item.tier {
        VocabularyTier::Basic => egui::Color32::GREEN,
        VocabularyTier::Academic => egui::Color32::from_rgb(100, 100, 255),
        VocabularyTier::Hazardous => egui::Color32::RED,
    };

    egui::Frame::none()
        .fill(card_color)
        .stroke(egui::Stroke::new(1.0, stroke_color))
        .rounding(8.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&item.word).strong().size(16.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(match item.tier {
                        VocabularyTier::Basic => "🟢",
                        VocabularyTier::Academic => "🔵",
                        VocabularyTier::Hazardous => "🔴",
                    });
                });
            });

            ui.label(egui::RichText::new(&item.definition).italics().size(12.0));
            ui.add_space(4.0);

            // Mass / Friction Indicators
            ui.horizontal(|ui| {
                ui.label("Mass:");
                let mass_color = if weight.effective_mass > 50.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                };
                ui.label(
                    egui::RichText::new(format!("{:.1}", weight.effective_mass)).color(mass_color),
                );

                // Visual mass bar
                let bar_width = 100.0;
                let fill = (weight.effective_mass / 100.0).clamp(0.0, 1.0) * bar_width;
                let (rect, _response) =
                    ui.allocate_exact_size(egui::vec2(bar_width, 6.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 3.0, egui::Color32::from_gray(60));
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(fill, 6.0)),
                    3.0,
                    mass_color,
                );
            });

            if !item.tags.is_empty() {
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    for tag in &item.tags {
                        ui.label(
                            egui::RichText::new(format!("#{}", tag))
                                .size(10.0)
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                    }
                });
            }
        });
    ui.add_space(8.0);
}
