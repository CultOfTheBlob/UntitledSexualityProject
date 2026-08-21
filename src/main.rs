//!---
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use genpdf::{
    Alignment, Document, Size,
    elements::{Paragraph, TableLayout},
    fonts::{self, FontData, FontFamily},
    style::Style,
};
use slint::{Model as _, ModelRc, SharedString};

#[allow(missing_docs, clippy::all, clippy::unwrap_used)]
mod ui {
    slint::include_modules!();
}
use ui::*;

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    main_window.on_validate_input(|text: SharedString| {
        let text = text.trim();

        if text.is_empty() {
            return false;
        }

        if text.starts_with('[') && text.ends_with(']') {
            let inner = &text[1..text.len() - 1];

            let Some((from_str, to_str)) = inner.split_once('~') else {
                return false;
            };

            let (Ok(from), Ok(to)) = (from_str.trim().parse::<f32>(), to_str.trim().parse::<f32>())
            else {
                return false;
            };

            return (0.0..=1.0).contains(&from) && (0.0..=1.0).contains(&to) && from < to;
        }

        if let Ok(number) = text.parse::<f32>() {
            return (0.0..=1.0).contains(&number);
        }

        false
    });

    main_window.on_submit_spectrums(|spectrums: ModelRc<SpectrumData>| {
        let spectrums: Vec<(String, String)> = spectrums
            .iter()
            .map(|item| {
                let data = item.data.to_string();

                if data == "Unknown" {
                    return (item.name.to_string(), String::from("\u{1D4E4}"));
                }

                if data == "Quadrants" || data == "M" {
                    return (item.name.to_string(), String::from("\u{1D4DC}"));
                }

                if data.starts_with('[') && data.ends_with(']') {
                    let inner = &data[1..data.len() - 1];

                    let Some((from_str, to_str)) = inner.split_once('~') else {
                        panic!();
                    };

                    let (Ok(from), Ok(to)) =
                        (from_str.trim().parse::<f32>(), to_str.trim().parse::<f32>())
                    else {
                        panic!();
                    };

                    return (item.name.to_string(), format!("{from} ~ {to}"));
                }

                if let Ok(data) = data.parse::<f32>()
                    && data == 0.0
                {
                    return (item.name.to_string(), String::from("\u{2205}"));
                }

                (item.name.to_string(), data)
            })
            .collect();

        let Ok(font_family) = fonts::from_files("./fonts", "DejaVuSerif", None) else {
            return;
        };

        let mut doc = Document::new(font_family);

        let Ok(math_font) = FontData::load("./fonts/NotoSansMath-Regular.ttf", None) else {
            return;
        };
        let math_family = doc.add_font_family(FontFamily {
            regular: math_font.clone(),
            bold: math_font.clone(),
            italic: math_font.clone(),
            bold_italic: math_font,
        });

        let card_size = Size::new(90.0, 60.0);
        doc.set_paper_size(card_size);

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(1.0);
        doc.set_page_decorator(decorator);

        let title_style = Style::new().bold().with_font_size(8);

        doc.push(Paragraph::default().styled_string("Spectrum List", title_style));

        let mut table = TableLayout::new(vec![1, 1]);

        let text_style = Style::new().with_font_size(5);
        let math_style = Style::new().with_font_size(5).with_font_family(math_family);

        for (name, mode) in spectrums {
            let mut row = table.row();

            row.push_element(Paragraph::default().styled_string(name, text_style));

            let mut right_align = Paragraph::default().styled_string(mode, math_style);

            right_align.set_alignment(Alignment::Right);

            row.push_element(right_align);

            let _ = row.push();
        }

        doc.push(table);

        let Ok(mut file) = std::fs::File::create("spectrum_card.pdf") else {
            return;
        };
        let _ = doc.render(&mut file);
    });

    main_window.run()?;

    Ok(())
}
