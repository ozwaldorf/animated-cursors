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

pub fn encode(images: &[Image]) -> Vec<u8> {
    let mut toc = Vec::new();
    let mut chunks = Vec::new();
    let mut position = FILE_HEADER + 12 * images.len() as u32;
    for image in images {
        toc.extend([IMAGE, image.size, position]);
        let header =
            [IMAGE_HEADER, IMAGE, image.size, 1, image.width, image.height, image.xhot, image.yhot, image.delay];
        chunks.extend(header.into_iter().chain(image.pixels.iter().copied()));
        position += IMAGE_HEADER + 4 * image.pixels.len() as u32;
    }
    [MAGIC, FILE_HEADER, VERSION, images.len() as u32]
        .into_iter()
        .chain(toc)
        .chain(chunks)
        .flat_map(u32::to_le_bytes)
        .collect()
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
        let data = encode(&images);
        assert_eq!(&data[..4], b"Xcur");
        assert_eq!(decode(&data), Some(images));
    }
}
