use super::GifFile;
use super::GifHeader;
use super::GlobalColorTable;
use super::LogicalScreenDescriptor;
use super::Pixel;
use std::str;
use nom::multi::count;
use nom::number::complete::{le_u8, le_u16};
use nom::bits;
use nom::{
    bytes::complete::take,
    IResult,
    combinator::{map, map_res},
};

// Thanks https://blog.adamchalmers.com/nom-bits/
type BitInput<'a> = (&'a [u8], usize);

fn take_bit(i: BitInput) -> IResult<BitInput, bool> {
    map(bits::complete::take(1usize), |bits: u8| bits > 0)(i)
}

fn take_pixel(bytes: &[u8]) -> IResult<&[u8], Pixel> {
    let (bytes, red) = le_u8(bytes)?;
    let (bytes, green) = le_u8(bytes)?;
    let (bytes, blue) = le_u8(bytes)?;
    Ok((bytes, Pixel { red, green, blue }))
}

fn parse_header(bytes: &[u8]) -> IResult<&[u8], GifHeader> {
    let (bytes, magic) = map_res(take(6usize), str::from_utf8)(bytes)?;
    Ok((bytes, GifHeader::from(magic).unwrap()))
}

fn parse_logical_screen_descriptor(bytes: &[u8]) -> IResult<&[u8], LogicalScreenDescriptor> {
    struct PackedField {
        global_color_table_flag: bool,
        color_resolution: u16,
        sort_flag: bool,
        global_color_table_size: u16,
    }

    fn parse_packed_field(bits: BitInput) -> IResult<BitInput, PackedField> {
        let (bits, global_color_table_flag) = take_bit(bits)?;
        let (bits, color_resolution) = bits::complete::take(3usize)(bits)?;
        let (bits, sort_flag) = take_bit(bits)?;
        let (bits, global_color_table_size) = bits::complete::take(3usize)(bits)?;
        Ok((
                bits,
                PackedField {
                    global_color_table_flag,
                    color_resolution,
                    sort_flag,
                    global_color_table_size,
                }
           ))

    }
    let (bytes, canvas_width) = le_u16(bytes)?;
    let (bytes, canvas_height) = le_u16(bytes)?;
    let (bytes, packed_field) = nom::bits::bits(parse_packed_field)(bytes)?;
    let (bytes, background_color_index) = le_u8(bytes)?;
    let (bytes, pixel_aspect_ratio) = le_u8(bytes)?;
    Ok((
        bytes,
        LogicalScreenDescriptor{
            canvas_width,
            canvas_height,

            global_color_table_flag: packed_field.global_color_table_flag,
            color_resolution: packed_field.color_resolution, 
            sort_flag: packed_field.sort_flag,
            global_color_table_size: packed_field.global_color_table_size,

            background_color_index,
            pixel_aspect_ratio,
        }
      )
        )

}
fn parse_global_color_table<'a>(bytes: &'a[u8], lsd: &LogicalScreenDescriptor) -> IResult<&'a[u8], Option<GlobalColorTable>> {

    // Early exit if not global color table
    if !lsd.global_color_table_flag {
        return Ok((bytes, None));
    }

    // `lsd.global_color_table_size` is at most 0b111, so plus 1 is 0b1000 which fits into the u16.
    // 2^(0b1000) is 256 which fits in an u16 (not u8 since its 1 over since acceptable
    // range is 0 to 255)
    let num_colors: usize = (2_usize).pow((lsd.global_color_table_size+1).into());
    let (bytes, ret) = count(take_pixel, num_colors)(bytes)?;

    Ok((bytes, Some(ret)))
}

impl GifFile {
    pub fn new(bytes: &[u8]) -> Result<GifFile, &'static str> {
        let (bytes, header) = parse_header(bytes).unwrap();
        let (bytes, logical_screen_descriptor) = parse_logical_screen_descriptor(bytes).unwrap();
        let (bytes, global_color_table) = parse_global_color_table(bytes, &logical_screen_descriptor).unwrap();
        Ok(
            GifFile {
                header,
                logical_screen_descriptor,
                global_color_table,
            }
        )
    }
}
