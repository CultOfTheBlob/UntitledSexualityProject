use std::fmt::Write;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url, wasm_bindgen::JsCast as _};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(deserialize_with = "deserialize_spectrums")]
    spectrums: Vec<(String, Spectrum)>,

    name: String,
    pronouns: String,

    #[serde(skip)]
    background_image: Option<Vec<u8>>,
    #[serde(skip)]
    background_image_name: Option<String>,

    #[serde(skip)]
    background_receiver: std::sync::mpsc::Receiver<(String, Vec<u8>)>,
    #[serde(skip)]
    background_sender: std::sync::mpsc::Sender<(String, Vec<u8>)>,
}

impl Default for App {
    fn default() -> Self {
        let (sender, reciever) = std::sync::mpsc::channel();

        Self {
            spectrums: vec![
                (String::from("Spectrum1"), Spectrum::default()),
                (String::from("Spectrum2"), Spectrum::default()),
                (String::from("Spectrum3"), Spectrum::default()),
                (String::from("Spectrum4"), Spectrum::default()),
                (String::from("Spectrum5"), Spectrum::default()),
                (String::from("Spectrum6"), Spectrum::default()),
                (String::from("Spectrum7"), Spectrum::default()),
                (String::from("Spectrum8"), Spectrum::default()),
                (String::from("Spectrum9"), Spectrum::default()),
                (String::from("Spectrum10"), Spectrum::default()),
                (String::from("Spectrum11"), Spectrum::default()),
                (String::from("Spectrum12"), Spectrum::default()),
            ],

            name: String::new(),
            pronouns: String::new(),

            background_image: None,
            background_image_name: None,

            background_receiver: reciever,
            background_sender: sender,
        }
    }
}

fn deserialize_spectrums<'de, D>(deserializer: D) -> Result<Vec<(String, Spectrum)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let saved: Vec<(String, Spectrum)> = serde::Deserialize::deserialize(deserializer)?;

    let mut merged = App::default().spectrums;

    for (key, value) in &mut merged {
        if let Some((_, saved_spectrum)) = saved.iter().find(|(k, _)| k == key) {
            *value = saved_spectrum.clone();
        }
    }

    Ok(merged)
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.storage.map_or_else(Default::default, |storage| {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        })
    }

    fn get_pdf_bytes(&self) -> Option<Vec<u8>> {
        let name = &self.name;
        let pronouns = &self.pronouns;

        let spectrums: Vec<(&String, String)> = self
            .spectrums
            .iter()
            .map(|(name, spectrum)| {
                let spectrum = match spectrum {
                    Spectrum::Quadrants(quadrant) => quadrant.to_string(),
                    Spectrum::Unknown => String::from("U"),
                    Spectrum::Number(value) => value.to_string(),
                };

                (name, spectrum)
            })
            .collect();

        let dejavu_regular_bytes = include_bytes!("../assets/fonts/DejaVuSans.ttf");
        let dejavu_bold_bytes = include_bytes!("../assets/fonts/DejaVuSans-Bold.ttf");

        let mut rows = String::new();

        for (name, spectrum) in spectrums {
            let _ = write!(
                rows,
                r"
                <tr>
                    <td>{name}</td>
                    <td>{spectrum}</td>
                </tr>
                "
            );
        }

        let html = format!(
            r#"
            <html>
                <head>
                    <style>
                        body {{
                            font-family: "DejaVu Sans";
                            font-size: 8pt;
                            padding: 5mm;
                        }}

                        table {{
                            width: 100%;
                            border-collapse: collapse;
                        }}

                        th {{
                            font-family: "DejaVu Sans Bold";
                            font-size: 12pt;
                            text-align: left;
                            padding-bottom: 2mm;
                        }}

                        td {{
                            padding-top: 1.0mm;
                            padding-bottom: 1.0mm;
                            text-align: left;
                        }}

                        th:first-child,
                        td:first-child {{
                            width: 65%;
                        }}

                        th:last-child,
                        td:last-child {{
                            width: 35%;
                        }}
                    </style>
                </head>

                <body>
                    <table>
                        <thead>
                            <tr>
                                <th>{name} ({pronouns})</th>
                            </tr>
                        </thead>

                        <tbody>
                            {rows}
                        </tbody>
                    </table>
                </body>
            </html>
            "#
        );

        let mut fonts = std::collections::BTreeMap::new();

        fonts.insert(
            "DejaVu Sans".to_string(),
            printpdf::Base64OrRaw::Raw(dejavu_regular_bytes.to_vec()),
        );

        fonts.insert(
            "DejaVu Sans Bold".to_string(),
            printpdf::Base64OrRaw::Raw(dejavu_bold_bytes.to_vec()),
        );

        let images = std::collections::BTreeMap::new();

        let options = printpdf::GeneratePdfOptions {
            page_width: Some(90.0),
            page_height: Some(120.0),

            margin_left: Some(2.0),
            margin_right: Some(2.0),

            margin_top: Some(2.0),
            margin_bottom: Some(2.0),

            ..Default::default()
        };

        let mut warnings = Vec::new();

        let mut doc =
            printpdf::PdfDocument::from_html(&html, &images, &fonts, &options, &mut warnings)
                .ok()?;

        if let Some(background_bytes) = &self.background_image {
            let Ok(background) =
                printpdf::RawImage::decode_from_bytes(background_bytes, &mut warnings)
            else {
                panic!();
            };

            let image_id = doc.add_image(&background);

            let page_width_mm = 90.0;
            let page_height_mm = 120.0;

            let image_width = background.width as f32;
            let image_height = background.height as f32;

            let page_width_pt = printpdf::Mm(page_width_mm).into_pt().0;
            let page_height_pt = printpdf::Mm(page_height_mm).into_pt().0;

            let image_width_pt = image_width / 300.0 * 72.0;
            let image_height_pt = image_height / 300.0 * 72.0;

            let scale = f32::max(
                page_width_pt / image_width_pt,
                page_height_pt / image_height_pt,
            );

            let scaled_width = image_width_pt * scale;
            let scaled_height = image_height_pt * scale;

            let x = (page_width_pt - scaled_width) / 2.0;
            let y = (page_height_pt - scaled_height) / 2.0;

            doc.pages[0].ops.insert(
                0,
                printpdf::Op::UseXobject {
                    id: image_id,
                    transform: printpdf::XObjectTransform {
                        translate_x: Some(printpdf::Pt(x)),
                        translate_y: Some(printpdf::Pt(y)),
                        scale_x: Some(scale),
                        scale_y: Some(scale),
                        dpi: Some(300.0),
                        ..Default::default()
                    },
                },
            );
        }

        let pdf_bytes: Vec<u8> = doc.save(&printpdf::PdfSaveOptions::default(), &mut warnings);

        Some(pdf_bytes)
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok((name, bytes)) = self.background_receiver.try_recv() {
            self.background_image = Some(bytes);
            self.background_image_name = Some(name);
        }

        ui.ctx().set_zoom_factor(1.25);

        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export").clicked() {
                        let Some(pdf_bytes) = self.get_pdf_bytes() else {
                            return;
                        };

                        let Some(window) = web_sys::window() else {
                            return;
                        };

                        let Some(document) = window.document() else {
                            return;
                        };

                        let array = js_sys::Array::new();
                        let uint8_array = js_sys::Uint8Array::from(&pdf_bytes[..]);
                        array.push(&uint8_array.buffer());

                        let properties = BlobPropertyBag::new();
                        properties.set_type("application/pdf");

                        let Ok(blob) =
                            Blob::new_with_u8_array_sequence_and_options(&array, &properties)
                        else {
                            return;
                        };

                        let Ok(url) = Url::create_object_url_with_blob(&blob) else {
                            return;
                        };

                        let Ok(element) = document.create_element("a") else {
                            return;
                        };

                        let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() else {
                            return;
                        };

                        anchor.set_href(&url);
                        anchor.set_download("spectrum_card.pdf");
                        anchor.click();

                        let _ = Url::revoke_object_url(&url);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Background:");
                                ui.label(
                                    self.background_image_name
                                        .clone()
                                        .unwrap_or_else(|| String::from("None")),
                                );

                                ui.weak("|");

                                if ui.button("Choose image").clicked() {
                                    let sender = self.background_sender.clone();

                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Some(file) = rfd::AsyncFileDialog::new()
                                            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                            .pick_file()
                                            .await
                                        {
                                            let name = file.file_name();
                                            let bytes = file.read().await;

                                            let _ = sender.send((name, bytes));
                                        }
                                    });
                                }

                                if ui.button("Clear").clicked() {
                                    self.background_image_name = None;
                                    self.background_image = None;
                                }
                            });
                        });

                        ui.add_space(16.0);

                        ui.group(|ui| {
                            egui::Grid::new("grid")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.name);
                                    ui.end_row();

                                    ui.label("Pronouns:");
                                    ui.text_edit_singleline(&mut self.pronouns);
                                    ui.end_row();
                                });
                        });

                        ui.add_space(16.0);

                        for (name, spectrum) in &mut self.spectrums {
                            ui.group(|ui| {
                                egui::ComboBox::from_label(name.clone())
                                    .selected_text(spectrum.to_string())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            spectrum,
                                            Spectrum::Quadrants(Quadrant::default()),
                                            Spectrum::Quadrants(Quadrant::default()).to_string(),
                                        );
                                        ui.selectable_value(
                                            spectrum,
                                            Spectrum::Unknown,
                                            Spectrum::Unknown.to_string(),
                                        );
                                        ui.selectable_value(
                                            spectrum,
                                            Spectrum::Number(0.0),
                                            Spectrum::Number(0.0).to_string(),
                                        );
                                    });

                                match spectrum {
                                    Spectrum::Quadrants(quadrant) => {
                                        ui.horizontal(|ui| {
                                            ui.label("Quadrant:");
                                            ui.radio_value(
                                                quadrant,
                                                Quadrant::Q1,
                                                Quadrant::Q1.to_string(),
                                            );
                                            ui.radio_value(
                                                quadrant,
                                                Quadrant::Q2,
                                                Quadrant::Q2.to_string(),
                                            );
                                            ui.radio_value(
                                                quadrant,
                                                Quadrant::M,
                                                Quadrant::M.to_string(),
                                            );
                                            ui.radio_value(
                                                quadrant,
                                                Quadrant::Q3,
                                                Quadrant::Q3.to_string(),
                                            );
                                            ui.radio_value(
                                                quadrant,
                                                Quadrant::Q4,
                                                Quadrant::Q4.to_string(),
                                            );
                                        });
                                    }
                                    Spectrum::Number(value) => {
                                        ui.horizontal(|ui| {
                                            ui.label("Value:");
                                            ui.add(egui::Slider::new(value, 0.0..=1.0));
                                        });
                                    }
                                    Spectrum::Unknown => {
                                        ui.weak("No additional configuration needed.");
                                    }
                                }
                            });

                            ui.add_space(8.0);
                        }
                    });
                });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

#[derive(Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
enum Spectrum {
    Quadrants(Quadrant),
    #[default]
    Unknown,
    Number(f32),
}

impl std::fmt::Display for Spectrum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quadrants(_) => std::fmt::write(f, format_args!("Quadrants")),
            Self::Unknown => std::fmt::write(f, format_args!("Unknown")),
            Self::Number(_) => std::fmt::write(f, format_args!("Number")),
        }
    }
}

#[derive(Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
enum Quadrant {
    Q1,
    Q2,
    #[default]
    M,
    Q3,
    Q4,
}

impl std::fmt::Display for Quadrant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q1 => std::fmt::write(f, format_args!("Q1")),
            Self::Q2 => std::fmt::write(f, format_args!("Q2")),
            Self::M => std::fmt::write(f, format_args!("M")),
            Self::Q3 => std::fmt::write(f, format_args!("Q3")),
            Self::Q4 => std::fmt::write(f, format_args!("Q4")),
        }
    }
}
