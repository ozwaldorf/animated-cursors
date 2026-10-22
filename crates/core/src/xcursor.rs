//! Minimal Xcursor file encoding and decoding.

const MAGIC: u32 = 0x7275_6358;
const VERSION: u32 = 0x1_0000;
const IMAGE: u32 = 0xfffd_0002;
const FILE_HEADER: u32 = 16;
const IMAGE_HEADER: u32 = 36;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub xhot: u32,
    pub yhot: u32,
    pub delay: u32,
    pub pixels: Vec<u32>,
}

/// Writes `images`, each stored once, with a table entry for each index in
/// `entries` in turn, so a frame can play several times in one loop.
pub fn encode(images: &[Image], entries: &[usize]) -> Vec<u8> {
    let mut positions = Vec::with_capacity(images.len());
    let mut chunks = Vec::new();
    let mut position = FILE_HEADER + 12 * entries.len() as u32;
    for image in images {
        positions.push(position);
        let header =
            [IMAGE_HEADER, IMAGE, image.size, 1, image.width, image.height, image.xhot, image.yhot, image.delay];
        chunks.extend(header.into_iter().chain(image.pixels.iter().copied()));
        position += IMAGE_HEADER + 4 * image.pixels.len() as u32;
    }
    let toc = entries.iter().flat_map(|&i| [IMAGE, images[i].size, positions[i]]);
    [MAGIC, FILE_HEADER, VERSION, entries.len() as u32]
        .into_iter()
        .chain(toc)
        .chain(chunks)
        .flat_map(u32::to_le_bytes)
        .collect()
}

/// Crops images of one width and height to the box around every hotspot and
/// visible pixel among them, keeping a transparent margin of a pixel where
/// the canvas has room, so they keep one shared shape.
pub fn crop(images: &mut [Image]) {
    let Some(first) = images.first() else { return };
    let (width, height) = (first.width, first.height);
    let points = images.iter().flat_map(|image| {
        let visible = image.pixels.iter().enumerate().filter(|(_, pixel)| *pixel >> 24 > 0);
        visible.map(|(i, _)| (i as u32 % width, i as u32 / width)).chain([(image.xhot, image.yhot)])
    });
    let (x0, y0, x1, y1) = points
        .fold((u32::MAX, u32::MAX, 0, 0), |(x0, y0, x1, y1), (x, y)| (x0.min(x), y0.min(y), x1.max(x), y1.max(y)));
    let (x0, y0, x1, y1) =
        (x0.saturating_sub(1), y0.saturating_sub(1), (x1 + 1).min(width - 1), (y1 + 1).min(height - 1));
    for image in images {
        let row = |y: u32| &image.pixels[(y * width + x0) as usize..=(y * width + x1) as usize];
        image.pixels = (y0..=y1).flat_map(row).copied().collect();
        (image.width, image.height) = (x1 - x0 + 1, y1 - y0 + 1);
        (image.xhot, image.yhot) = (image.xhot - x0, image.yhot - y0);
    }
}

pub fn decode(data: &[u8]) -> Option<Vec<Image>> {
    let word = |offset: usize| Some(u32::from_le_bytes(data.get(offset..offset + 4)?.try_into().ok()?));
    if word(0)? != MAGIC {
        return None;
    }
    let header = word(4)? as usize;
    (0..word(12)? as usize)
        .filter(|index| word(header + index * 12) == Some(IMAGE))
        .map(|index| {
            let offset = word(header + index * 12 + 8)? as usize;
            let field = |n: usize| word(offset + 4 * n);
            let (width, height) = (field(4)?, field(5)?);
            let start = offset + field(0)? as usize;
            let pixels = (0..(width * height) as usize).map(|i| word(start + 4 * i)).collect::<Option<_>>()?;
            Some(Image { size: field(2)?, width, height, xhot: field(6)?, yhot: field(7)?, delay: field(8)?, pixels })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_images() {
        let images = vec![
            Image { size: 2, width: 2, height: 1, xhot: 1, yhot: 0, delay: 5, pixels: vec![1, 0xff00_0000] },
            Image { size: 3, width: 1, height: 1, xhot: 0, yhot: 0, delay: 7, pixels: vec![42] },
        ];
        let data = encode(&images, &[0, 1]);
        assert_eq!(&data[..4], b"Xcur");
        assert_eq!(decode(&data), Some(images));
    }

    #[test]
    fn repeats_share_one_image() {
        let images = vec![
            Image { size: 2, width: 1, height: 1, xhot: 0, yhot: 0, delay: 5, pixels: vec![7] },
            Image { size: 2, width: 1, height: 1, xhot: 0, yhot: 0, delay: 5, pixels: vec![9] },
        ];
        let (once, repeated) = (encode(&images, &[0, 1]), encode(&images, &[0, 1, 0, 0]));
        assert_eq!(repeated.len(), once.len() + 2 * 12);
        let played = decode(&repeated).unwrap();
        assert_eq!(played, vec![images[0].clone(), images[1].clone(), images[0].clone(), images[0].clone()]);
    }

    #[test]
    fn crops_to_shared_box_with_margin() {
        let canvas = |visible: usize, xhot| {
            let mut pixels = vec![0; 36];
            pixels[visible] = 0xff00_0000;
            Image { size: 6, width: 6, height: 6, xhot, yhot: 0, delay: 1, pixels }
        };
        // Visible at (2, 2) and (3, 3), hotspots at (0, 0) and (2, 0).
        let mut images = vec![canvas(14, 0), canvas(21, 2)];
        crop(&mut images);
        assert_eq!((images[0].width, images[0].height), (5, 5));
        assert_eq!([(images[0].xhot, images[0].yhot), (images[1].xhot, images[1].yhot)], [(0, 0), (2, 0)]);
        assert_eq!(images[0].pixels[2 * 5 + 2], 0xff00_0000);
        assert_eq!(images[1].pixels[3 * 5 + 3], 0xff00_0000);
        assert_eq!(images[0].pixels.iter().filter(|&&pixel| pixel != 0).count(), 1);
    }
}
