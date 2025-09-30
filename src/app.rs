use crate::models::{Category, MediaItem, Query, SortField, SortOrder, Status};
use crate::repo::{Repository, Stats};
use crate::sqlite_repo::SqliteRepo;
use crate::util;
use chrono::Local;
use eframe::egui::{self, Button, Id, Key, Response, RichText, TextEdit};
use egui_extras::{Column, TableBuilder};
use std::path::Path;

pub struct CatalogApp {
    repo: Box<dyn Repository>,
    items: Vec<MediaItem>,
    query: Query,
    new_item_title: String,
    new_item_category: Category,
    error: Option<String>,
    stats: Stats,
}

impl CatalogApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, db_path: &Path) -> Self {
        let repo = SqliteRepo::new(db_path);
        let _ = repo.init();
        let mut app = Self {
            repo: Box::new(repo),
            items: vec![],
            query: Query {
                sort_field: SortField::UpdatedAt,
                sort_order: SortOrder::Desc,
                ..Default::default()
            },
            new_item_title: String::new(),
            new_item_category: Category::Movie,
            error: None,
            stats: Stats::default(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        match self.repo.list(&self.query) {
            Ok(list) => self.items = list,
            Err(e) => self.error = Some(e.to_string()),
        }
        match self.repo.stats() {
            Ok(stats) => self.stats = stats,
            Err(e) => self.error = Some(e.to_string()),
        }
    }
}

impl eframe::App for CatalogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Media Catalog");
                if ui.button("Export CSV (filtered)").clicked() {
                    match util::export_csv(&self.items) {
                        Ok(path) => self.error = Some(format!("Exported: {}", path.display())),
                        Err(e) => self.error = Some(format!("Export failed: {}", e)),
                    }
                }
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Total: {} | Finished: {} | Unfinished: {}",
                        self.stats.total, self.stats.finished, self.stats.unfinished
                    ))
                    .small(),
                );
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, err);
            }
            ui.horizontal(|ui| {
                ui.label("Add new:");
                ui.add(
                    TextEdit::singleline(&mut self.new_item_title)
                        .hint_text("Title")
                        .desired_width(200.0),
                );
                egui::ComboBox::from_label("Category")
                    .selected_text(self.new_item_category.to_string())
                    .show_ui(ui, |ui| {
                        for c in Category::ALL {
                            if ui
                                .selectable_label(self.new_item_category == c, c.to_string())
                                .clicked()
                            {
                                self.new_item_category = c;
                            }
                        }
                    });
                if ui.add(Button::new("+ Add")).clicked() || ui.input(|i| i.key_pressed(Key::Enter))
                {
                    let title = self.new_item_title.trim();
                    if title.is_empty() {
                        self.error = Some("Title cannot be empty".into());
                    } else {
                        let mut item = MediaItem::new(title, self.new_item_category);
                        if let Err(e) = self.repo.add(&mut item) {
                            self.error = Some(e.to_string());
                        }
                        self.new_item_title.clear();
                        self.refresh();
                    }
                }
            });
        });

        egui::SidePanel::left("filters")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Filters");
                ui.separator();
                ui.label("Search title contains:");
                ui.add(TextEdit::singleline(&mut self.query.title_substr).hint_text("e.g., Dune"));
                ui.label("Category:");
                egui::ComboBox::from_id_source("filter_cat")
                    .selected_text(
                        self.query
                            .category
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "Any".into()),
                    )
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(self.query.category.is_none(), "Any")
                            .clicked()
                        {
                            self.query.category = None;
                        }
                        for c in Category::ALL {
                            if ui
                                .selectable_label(self.query.category == Some(c), c.to_string())
                                .clicked()
                            {
                                self.query.category = Some(c);
                            }
                        }
                    });
                ui.label("Status:");
                egui::ComboBox::from_id_source("filter_status")
                    .selected_text(
                        self.query
                            .status
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Any".into()),
                    )
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(self.query.status.is_none(), "Any")
                            .clicked()
                        {
                            self.query.status = None;
                        }
                        for s in Status::ALL {
                            if ui
                                .selectable_label(self.query.status == Some(s), s.to_string())
                                .clicked()
                            {
                                self.query.status = Some(s);
                            }
                        }
                    });
                ui.label("Min rating:");
                let mut rating_str = self
                    .query
                    .min_rating
                    .map(|r| r.to_string())
                    .unwrap_or_default();
                if ui
                    .add(TextEdit::singleline(&mut rating_str).hint_text("0..10"))
                    .lost_focus()
                {
                    self.query.min_rating = rating_str.parse::<u8>().ok();
                }
                ui.separator();
                ui.label("Sort by:");
                egui::ComboBox::from_id_source("sort_field")
                    .selected_text(format!("{:?}", self.query.sort_field))
                    .show_ui(ui, |ui| {
                        for f in [
                            SortField::Title,
                            SortField::Category,
                            SortField::Status,
                            SortField::Rating,
                            SortField::CreatedAt,
                            SortField::UpdatedAt,
                        ] {
                            if ui
                                .selectable_label(self.query.sort_field == f, format!("{:?}", f))
                                .clicked()
                            {
                                self.query.sort_field = f;
                            }
                        }
                    });
                egui::ComboBox::from_id_source("sort_order")
                    .selected_text(match self.query.sort_order {
                        SortOrder::Asc => "Asc",
                        SortOrder::Desc => "Desc",
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(self.query.sort_order, SortOrder::Asc),
                                "Asc",
                            )
                            .clicked()
                        {
                            self.query.sort_order = SortOrder::Asc;
                        }
                        if ui
                            .selectable_label(
                                matches!(self.query.sort_order, SortOrder::Desc),
                                "Desc",
                            )
                            .clicked()
                        {
                            self.query.sort_order = SortOrder::Desc;
                        }
                    });
                if ui.button("Apply").clicked() {
                    self.refresh();
                }
                if ui.button("Clear filters").clicked() {
                    self.query = Query {
                        sort_field: self.query.sort_field,
                        sort_order: self.query.sort_order,
                        ..Default::default()
                    };
                    self.refresh();
                }

                ui.separator();
                ui.heading("By category");
                for (cat, count) in &self.stats.by_category {
                    ui.label(format!("{}: {}", cat, count));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Items");
            ui.add_space(6.0);

            let mut need_refresh = false;

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::auto())
                .column(Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Actions");
                    });
                    header.col(|ui| {
                        ui.strong("Title");
                    });
                    header.col(|ui| {
                        ui.strong("Category");
                    });
                    header.col(|ui| {
                        ui.strong("Status");
                    });
                    header.col(|ui| {
                        ui.strong("Rating");
                    });
                    header.col(|ui| {
                        ui.strong("Notes");
                    });
                    header.col(|ui| {
                        ui.strong("Cover");
                    });
                    header.col(|ui| {
                        ui.strong("Updated");
                    });
                })
                .body(|mut body| {
                    for item in &mut self.items {
                        body.row(24.0, |mut row| {
                            row.col(|ui| {
                                if ui
                                    .small_button("✓")
                                    .on_hover_text("Mark finished")
                                    .clicked()
                                {
                                    item.status = Status::Finished;
                                    item.updated_at = Local::now();
                                    if let Err(e) = self.repo.update(item) {
                                        self.error = Some(e.to_string());
                                    }
                                    need_refresh = true;
                                }

                                let edit_id =
                                    Id::new(format!("edit_{}", item.id.unwrap_or_default()));
                                let edit_response: Response =
                                    ui.small_button("✎").on_hover_text("Edit");
                                if edit_response.clicked() {
                                    ui.ctx().memory_mut(|m| m.toggle_popup(edit_id));
                                }

                                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                    if let Some(id) = item.id {
                                        if let Err(e) = self.repo.delete(id) {
                                            self.error = Some(e.to_string());
                                        }
                                    }
                                    need_refresh = true;
                                }

                                egui::popup::popup_below_widget(
                                    ui,
                                    edit_id,
                                    &edit_response,
                                    |ui| {
                                        ui.label(RichText::new("Edit item").strong());
                                        ui.separator();
                                        let mut title = item.title.clone();
                                        ui.label("Title:");
                                        ui.add(
                                            TextEdit::singleline(&mut title).desired_width(240.0),
                                        );
                                        ui.label("Category:");
                                        let mut cat = item.category;
                                        egui::ComboBox::from_id_source(edit_id.with("cat"))
                                            .selected_text(cat.to_string())
                                            .show_ui(ui, |ui| {
                                                for c in Category::ALL {
                                                    if ui
                                                        .selectable_label(cat == c, c.to_string())
                                                        .clicked()
                                                    {
                                                        cat = c;
                                                    }
                                                }
                                            });
                                        ui.label("Status:");
                                        let mut st = item.status;
                                        egui::ComboBox::from_id_source(edit_id.with("status"))
                                            .selected_text(st.to_string())
                                            .show_ui(ui, |ui| {
                                                for s in Status::ALL {
                                                    if ui
                                                        .selectable_label(st == s, s.to_string())
                                                        .clicked()
                                                    {
                                                        st = s;
                                                    }
                                                }
                                            });
                                        if ui.button("Save").clicked() {
                                            item.title = title;
                                            item.category = cat;
                                            item.status = st;
                                            item.updated_at = Local::now();
                                            if let Err(e) = self.repo.update(item) {
                                                self.error = Some(e.to_string());
                                            }
                                            ui.ctx().memory_mut(|m| m.close_popup());
                                            need_refresh = true;
                                        }
                                        if ui.button("Cancel").clicked() {
                                            ui.ctx().memory_mut(|m| m.close_popup());
                                        }
                                    },
                                );
                            });

                            row.col(|ui| {
                                ui.label(&item.title);
                            });

                            row.col(|ui| {
                                ui.label(item.category.to_string());
                            });

                            row.col(|ui| {
                                ui.label(item.status.to_string());
                            });

                            row.col(|ui| {
                                let mut tmp = item.rating.unwrap_or(0).to_string();
                                if ui
                                    .add(TextEdit::singleline(&mut tmp).desired_width(30.0))
                                    .lost_focus()
                                {
                                    item.rating = tmp.parse::<u8>().ok();
                                    item.updated_at = Local::now();
                                    if let Err(e) = self.repo.update(item) {
                                        self.error = Some(e.to_string());
                                    }
                                }
                            });

                            row.col(|ui| {
                                let mut text = item.notes.clone().unwrap_or_default();
                                if ui
                                    .add(TextEdit::singleline(&mut text).desired_width(200.0))
                                    .lost_focus()
                                {
                                    item.notes = if text.trim().is_empty() {
                                        None
                                    } else {
                                        Some(text)
                                    };
                                    item.updated_at = Local::now();
                                    if let Err(e) = self.repo.update(item) {
                                        self.error = Some(e.to_string());
                                    }
                                }
                            });

                            row.col(|ui| {
                                let cover_display = item.cover_path.clone().unwrap_or_default();
                                if ui.small_button("Pick...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg"])
                                        .pick_file()
                                    {
                                        item.cover_path = Some(path.display().to_string());
                                        item.updated_at = Local::now();
                                        if let Err(e) = self.repo.update(item) {
                                            self.error = Some(e.to_string());
                                        }
                                        need_refresh = true;
                                    }
                                }
                                if cover_display.is_empty() {
                                    ui.small("(none)");
                                } else {
                                    ui.small(cover_display);
                                }
                            });

                            row.col(|ui| {
                                ui.small(item.updated_at.format("%Y-%m-%d %H:%M").to_string());
                            });
                        });
                    }
                });

            if need_refresh {
                self.refresh();
            }
        });
    }
}
