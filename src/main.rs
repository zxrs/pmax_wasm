use anyhow::{Context, Result};
use image::{imageops::Lanczos3, DynamicImage, RgbImage};
use mozjpeg::{ColorSpace, Compress, Decompress, Marker, ScanMode, ALL_MARKERS};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let input = fs::read("/input")?;

    let quality = env::args().nth(1).context("no quality.")?.parse()?;
    let size = env::args().nth(2).and_then(|v| v.parse().ok());
    let delete_exif = env::args().nth(3).map_or(true, |v| v.starts_with("true"));
    let denoise = env::args()
        .nth(4)
        .and_then(|v| v.parse().ok())
        .and_then(|v| if 0.0 <= v && v <= 10.0 { Some(v) } else { None })
        .map(|v| f64::MAX - f64::MAX / 10.0 * v);

    let data = generate(input, quality, size, delete_exif, denoise)?;

    fs::write("/output", &data)?;

    Ok(())
}

fn generate(
    input: Vec<u8>,
    quality: f32,
    size: Option<u32>,
    delete_exif: bool,
    _denoise: Option<f64>,
) -> Result<Vec<u8>> {
    let decoded = decode(input, delete_exif)?;
    let resized = resize(decoded, size)?;
    let encoded = encode(resized, quality)?;
    Ok(encoded)
}

struct Decoded {
    data: Vec<u8>,
    width: usize,
    height: usize,
    markers: Option<Vec<(Marker, Vec<u8>)>>,
}

fn decode(input: Vec<u8>, delete_exif: bool) -> Result<Decoded> {
    let decomp = Decompress::builder()
        .with_markers(ALL_MARKERS)
        .from_mem(&input)?;
    let markers = if delete_exif {
        None
    } else {
        Some(
            decomp
                .markers()
                .into_iter()
                .map(|m| (m.marker, m.data.into()))
                .collect(),
        )
    };
    let mut decomp = decomp.rgb()?;
    let width = decomp.width();
    let height = decomp.height();
    let data = decomp
        .read_scanlines::<[u8; 3]>()?
        .iter()
        .flatten()
        .copied()
        .collect();
    decomp.finish()?;
    Ok(Decoded {
        data,
        width,
        height,
        markers,
    })
}

fn encode(decoded: Decoded, quality: f32) -> Result<Vec<u8>> {
    let mut buf = vec![];
    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_quality(quality);
    comp.set_size(decoded.width, decoded.height);
    comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
    let mut comp = comp.start_compress(&mut buf)?;

    if let Some(markers) = decoded.markers {
        markers.into_iter().for_each(|m| {
            comp.write_marker(m.0, &m.1);
        });
    }

    decoded
        .data
        .chunks(decoded.width * 3)
        .into_iter()
        .try_for_each(|d| comp.write_scanlines(d))?;

    comp.finish()?;

    Ok(buf)
}

fn resize(decoded: Decoded, size: Option<u32>) -> Result<Decoded> {
    let Some(size) = size else {
        return Ok(decoded);
    };

    if decoded.width < size as _ && decoded.height < size as _ {
        return Ok(decoded);
    }

    let img = RgbImage::from_raw(decoded.width as _, decoded.height as _, decoded.data)
        .context("no image.")?;
    let img = DynamicImage::ImageRgb8(img);
    let img = img.resize(size, size, Lanczos3);
    let width = img.width() as _;
    let height = img.height() as _;
    Ok(Decoded {
        data: img.into_rgb8().into_vec(),
        width,
        height,
        ..decoded
    })
}
