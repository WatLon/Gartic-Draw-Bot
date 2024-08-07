use druid::image::{
    imageops::colorops::{dither, ColorMap},
    imageops::FilterType::Lanczos3,
    DynamicImage, ImageBuffer, Rgb,
};

pub struct Palette {
    colors: Vec<Rgb<u8>>,
}

impl Palette {
    pub fn from_colors(colors: Vec<Rgb<u8>>) -> Self {
        Palette { colors }
    }
}

impl ColorMap for Palette {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        self.colors.iter().position(|c| c == color).unwrap()
    }

    fn map_color(&self, pixel: &mut Self::Color) {
        let old_color = *pixel;
        let mut closest_color = &self.colors[0];
        let mut min_distance = color_distance(&old_color, &closest_color);

        for color in &self.colors {
            let distance = color_distance(&old_color, color);

            if distance < min_distance {
                min_distance = distance;
                closest_color = color;
            }
        }

        *pixel = *closest_color;
    }
}

pub fn quantize(
    img: &DynamicImage,
    palette: &[Rgb<u8>],
    size: (u32, u32),
    dithering: bool,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let img = img.resize(size.0, size.1, Lanczos3);
    let mut img = img.to_rgb8();

    match dithering {
        true => {
            dither(&mut img, &Palette::from_colors(palette.to_vec()));

            img
        }
        false => {
            let mut quantized_img = ImageBuffer::new(img.width(), img.height());

            for (x, y, pixel) in img.enumerate_pixels() {
                // Находим ближайший цвет из палитры
                let nearest_color = find_nearest_color(pixel, palette);
                quantized_img.put_pixel(x, y, nearest_color);
            }

            quantized_img
        }
    }
}

fn find_nearest_color(pixel: &Rgb<u8>, palette: &[Rgb<u8>]) -> Rgb<u8> {
    let mut nearest_color = palette[0];
    let mut nearest_distance = color_distance(pixel, &nearest_color);

    for &palette_color in palette.iter() {
        let distance = color_distance(pixel, &palette_color);
        if distance < nearest_distance {
            nearest_color = palette_color;
            nearest_distance = distance;
        }
    }

    nearest_color
}

fn color_distance(c1: &Rgb<u8>, c2: &Rgb<u8>) -> f64 {
    let r_diff = c1[0] as f64 - c2[0] as f64;
    let g_diff = c1[1] as f64 - c2[1] as f64;
    let b_diff = c1[2] as f64 - c2[2] as f64;
    (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff).sqrt()
}
