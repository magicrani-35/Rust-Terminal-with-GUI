use eframe::egui::Color32;

pub(crate) fn resolve(color: vt100::Color, default: Color32) -> Color32 {
    match color {
        vt100::Color::Default => default,
        vt100::Color::Rgb(red, green, blue) => {
            Color32::from_rgb(red, green, blue)
        }
        vt100::Color::Idx(index) => indexed(index),
    }
}

fn indexed(index: u8) -> Color32 {
    const BASIC: [[u8; 3]; 16] = [
        [0, 0, 0],
        [205, 49, 49],
        [13, 188, 121],
        [229, 229, 16],
        [36, 114, 200],
        [188, 63, 188],
        [17, 168, 205],
        [229, 229, 229],
        [102, 102, 102],
        [241, 76, 76],
        [35, 209, 139],
        [245, 245, 67],
        [59, 142, 234],
        [214, 112, 214],
        [41, 184, 219],
        [255, 255, 255],
    ];

    match index {
        0..=15 => {
            let [red, green, blue] = BASIC[usize::from(index)];
            Color32::from_rgb(red, green, blue)
        }
        16..=231 => {
            const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

            let offset = index - 16;
            let red = LEVELS[usize::from(offset / 36)];
            let green = LEVELS[usize::from((offset / 6) % 6)];
            let blue = LEVELS[usize::from(offset % 6)];

            Color32::from_rgb(red, green, blue)
        }
        232..=255 => {
            let shade = 8 + (index - 232) * 10;
            Color32::from_gray(shade)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_default_and_rgb_colors() {
        let default = Color32::from_rgb(20, 20, 24);

        assert_eq!(resolve(vt100::Color::Default, default), default);

        assert_eq!(
            resolve(vt100::Color::Rgb(10, 20, 30), default),
            Color32::from_rgb(10, 20, 30),
        );
    }

    #[test]
    fn resolves_indexed_color_boundaries() {
        assert_eq!(indexed(0), Color32::BLACK);
        assert_eq!(indexed(15), Color32::WHITE);
        assert_eq!(indexed(16), Color32::BLACK);
        assert_eq!(indexed(21), Color32::from_rgb(0, 0, 255));
        assert_eq!(indexed(231), Color32::WHITE);
        assert_eq!(indexed(232), Color32::from_gray(8));
        assert_eq!(indexed(255), Color32::from_gray(238));
    }
}