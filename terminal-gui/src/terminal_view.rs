use eframe::egui;

use crate::palette;

pub(crate) fn show(
    ui: &mut egui::Ui, 
    screen: &vt100::Screen,
    active: bool,
) {
    let font = egui::FontId::monospace(14.0);

    let cell_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'M'));
    let cell_height = ui.fonts_mut(|fonts| fonts.row_height(&font));

    let (rows, cols) = screen.size();

    let size = egui::vec2(
        f32::from(cols) * cell_width,
        f32::from(rows) * cell_height,
    );

    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 24));

    for row in 0..rows {
        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };

            let position = rect.min + egui::vec2(
                f32::from(col) * cell_width,
                f32::from(row) * cell_height,
            );

            let (_, background) = cell_colors(cell);

            painter.rect_filled(
                egui::Rect::from_min_size(
                    position,
                    egui::vec2(cell_width, cell_height),
                ),
                0.0,
                background,
            );
        }
    }

    for row in 0..rows {
        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };

            if cell.is_wide_continuation() {
                continue;
            }

            let position = rect.min + egui::vec2(
                f32::from(col) * cell_width,
                f32::from(row) * cell_height,
            );

            let (foreground, _) = cell_colors(cell);

            painter.text(
                position,
                egui::Align2::LEFT_TOP,
                cell.contents(),
                font.clone(),
                foreground,
            );
        }
    }

    if active && !screen.hide_cursor() {
        let (row, col) = screen.cursor_position();

        let position = rect.min + egui::vec2(
            f32::from(col.min(cols.saturating_sub(1))) * cell_width,
            f32::from(row) * cell_height + cell_height - 2.0,
        );

        painter.rect_filled(
            egui::Rect::from_min_size(position, egui::vec2(cell_width, 2.0)),
            0.0,
            egui::Color32::LIGHT_GREEN,
        );
    }
}

fn cell_colors(cell: &vt100::Cell) -> (egui::Color32, egui::Color32) {
    let foreground = palette::resolve(
        cell.fgcolor(),
        egui::Color32::LIGHT_GRAY,
    );

    let background = palette::resolve(
        cell.bgcolor(),
        egui::Color32::from_rgb(20, 20, 24),
    );

    if cell.inverse() {
        (background, foreground)
    } else {
        (foreground, foreground)
    }
}