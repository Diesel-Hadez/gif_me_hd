use crate::decoder::DisposalMethod;
use crate::decoder::ImageDescriptor;

use super::GifFile;
use super::GifFrame;
use super::GifHeader;
use super::Extension;
use super::GlobalColorTable;
use super::LocalColorTable;
use super::LogicalScreenDescriptor;
use super::Pixel;
use super::lzw;
use std::str;
use nom::bytes::complete::tag;
use nom::multi::fold_many1;
use nom::multi::{count, many0};
use nom::number::complete::{le_u8, le_u16};
use nom::bits;
use nom::sequence::preceded;
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

fn parse_extensions(bytes: &[u8]) -> IResult<&[u8], Vec<Extension>> {
    fn parse_extension(bytes: &[u8]) -> IResult<&[u8], Extension> {
        struct PackedField {
            reserved: u8,
            disposal_method: u8,
            user_input_flag: bool,
            transparent_color_flag: bool,
        }

        fn parse_packed_field(bits: BitInput) -> IResult<BitInput, PackedField> {
            let (bits, reserved) = bits::complete::take(3usize)(bits)?;
            let (bits, disposal_method) = bits::complete::take(3usize)(bits)?;
            let (bits, user_input_flag) = take_bit(bits)?;
            let (bits, transparent_color_flag) = take_bit(bits)?;
            Ok((
                    bits,
                    PackedField {
                        reserved,
                        disposal_method,
                        user_input_flag,
                        transparent_color_flag,
                    }
               ))

        }

        const BLOCK_TERMINATOR: &[u8] = &[0x00];
        const INTRODUCER: &[u8] = &[0x21];
        let (bytes, ext_type) = preceded(tag(INTRODUCER), le_u8)(bytes)?;
        match ext_type {
            0xF9 => {
                // Should always be 4 according to the specificatioins.
                // IDK why they put it there then.
                const GCE_BLOCK_SIZE: &[u8] = &[0x04];
                let (bytes, _) = tag(GCE_BLOCK_SIZE)(bytes)?;
                let (bytes, packed_field) = nom::bits::bits(parse_packed_field)(bytes)?;
                let (bytes, delay_timer) = le_u16(bytes)?;
                let (bytes, transparent_color_index) = le_u8(bytes)?;
                let (bytes, _) = tag(BLOCK_TERMINATOR)(bytes)?;

                Ok((
                    bytes,
                    Extension::GraphicsControlExtension{
                        reserved: packed_field.reserved, 
                        disposal_method: match packed_field.disposal_method {
                            0 => DisposalMethod::NoDisposal,
                            1 => DisposalMethod::DoNotDispose,
                            2 => DisposalMethod::RestoreToBackground,
                            3 => DisposalMethod::RestoreToPrevious,
                            _ => panic!("Invalid Disposal Method {}!", packed_field.disposal_method),
                        }, 
                        user_input_flag: packed_field.user_input_flag, 
                        transparent_color_flag: packed_field.transparent_color_flag, 
                        delay_timer, 
                        transparent_color_index,
                    }
                ))
                
            },
            0x01 => {
                unimplemented!("PlainText Extension not supported!");
                #[allow(unreachable_code)]
                Ok((
                    bytes,
                    Extension::PlainText { text: "".into() }
                ))
            },
            0xFF => {
                unimplemented!("Application Extension not supported!");
                #[allow(unreachable_code)]
                Ok((
                    bytes,
                    Extension::Application { identifier: "".into(), authentication_code: "".into(), data: vec![] }
                ))

            },
            0xFE => {
                unimplemented!("Comment Extension not supported!");
                #[allow(unreachable_code)]
                Ok((
                    bytes,
                    Extension::Comment { text: "".into() }
                ))

            },
            _ => panic!("Unsupported Extension {:#x}!", ext_type),
        }
    }
    let (bytes, extensions) = many0(parse_extension)(bytes)?;
    Ok((bytes, extensions))
}

fn parse_image_descriptor(bytes: &[u8]) -> IResult<&[u8], ImageDescriptor> {
    struct PackedField {
        local_color_table_flag: bool,
        interlace_flag: bool,
        sort_flag: bool,
        reserved: u8,
        local_color_table_size: u8,
    }

    fn parse_packed_field(bits: BitInput) -> IResult<BitInput, PackedField> {
        let (bits, local_color_table_flag) = take_bit(bits)?;
        let (bits, interlace_flag) = take_bit(bits)?;
        let (bits, sort_flag) = take_bit(bits)?;
        let (bits, reserved) = bits::complete::take(2usize)(bits)?;
        let (bits, local_color_table_size) = bits::complete::take(3usize)(bits)?;
        Ok((
                bits,
                PackedField {
                    local_color_table_flag,
                    interlace_flag,
                    sort_flag,
                    reserved,
                    local_color_table_size,
                }
           ))
    }

    const IMAGE_SEPARATOR: &[u8] = &[0x2C];
    let (bytes, _) = tag(IMAGE_SEPARATOR)(bytes)?;
    let (bytes, left) = le_u16(bytes)?; 
    let (bytes, top) = le_u16(bytes)?; 
    let (bytes, width) = le_u16(bytes)?; 
    let (bytes, height) = le_u16(bytes)?; 
    let (bytes, packed_field) = nom::bits::bits(parse_packed_field)(bytes)?;
    Ok((bytes, ImageDescriptor {
        left,
        top,
        width,
        height,
        local_color_table_flag: packed_field.local_color_table_flag,
        interlace_flag: packed_field.interlace_flag,
        sort_flag: packed_field.sort_flag,
        reserved: packed_field.reserved,
        local_color_table_size: packed_field.local_color_table_size,
    }))
}

fn parse_local_color_table<'a>(bytes: &'a[u8], image_descriptor: &ImageDescriptor) -> IResult<&'a[u8], Option<LocalColorTable>> {
    // Early exit if not local color table
    if !image_descriptor.local_color_table_flag{
        return Ok((bytes, None));
    }

    // `image_descriptor.local_color_table_size` is at most 0b111, so plus 1 is 0b1000 which fits into the u16.
    // 2^(0b1000) is 256 which fits in an u16 (not u8 since its 1 over since acceptable
    // range is 0 to 255)
    let num_colors: usize = (2_usize).pow((image_descriptor.local_color_table_size+1).into());
    let (bytes, ret) = count(take_pixel, num_colors)(bytes)?;

    Ok((bytes, Some(ret)))
}


// This is LZW compressed code used for both Image Data
// and a few other things like the Plain Text Extension.
fn parse_data_block(bytes: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // We try get the entire block out first because we weant
    // the decompression code to be somewhere else and not here.
    fn parse_data_subblock(bytes: &[u8]) -> IResult<&[u8], &[u8]> {
        let (bytes, subblock_length) = le_u8(bytes)?;
        let (bytes, subblock) = take(subblock_length)(bytes)?;
        Ok((bytes, subblock))
    }
    let (bytes, block) = fold_many1(parse_data_subblock, 
                                    Vec::new, 
                                    |mut acc: Vec<_>, item| {
                                        acc.extend_from_slice(item);
                                        acc
                                    })(bytes)?;
    Ok((bytes, block))
}

fn parse_image_data(bytes: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (bytes, lzw_minimum_code_size) = le_u8(bytes)?;
    let (bytes, compressed_data) = parse_data_block(bytes)?;

    Ok((bytes, lzw::decompress(compressed_data, lzw_minimum_code_size)))
}

fn parse_frame(bytes: &[u8]) -> IResult<&[u8], GifFrame> {
    let (bytes, extensions) = parse_extensions(bytes).unwrap();
    let (bytes, image_descriptor) = parse_image_descriptor(bytes)?;
    let (bytes, local_color_table) = parse_local_color_table(bytes, &image_descriptor)?;
    let (bytes, frame_indices) = parse_image_data(bytes)?;
    Ok((
            bytes,
            GifFrame {
                image_descriptor,
                local_color_table,
                frame_indices,
                extensions,
            }
        ))
}

impl GifFile {
    pub fn new(bytes: &[u8]) -> Result<GifFile, &'static str> {
        const TRAILER: &[u8] = &[0x3B];
        let (bytes, header) = parse_header(bytes).unwrap();
        let (bytes, logical_screen_descriptor) = parse_logical_screen_descriptor(bytes).unwrap();
        let (bytes, global_color_table) = parse_global_color_table(bytes, &logical_screen_descriptor).unwrap();
        let (bytes, frame) = parse_frame(bytes).unwrap();
        Ok(
            GifFile {
                header,
                logical_screen_descriptor,
                global_color_table,
                frames: vec![frame],
            }
        )
    }
}
