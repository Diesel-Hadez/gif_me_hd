pub enum GifHeader {
    GIF89a,
    GIF87a,
}
impl GifHeader {
    pub fn from(header: &str) -> Result<GifHeader, &'static str> {
        match header
            .to_uppercase()
            .as_str() 
        {
            "GIF89A" => Ok(GifHeader::GIF89a),
            "GIF87A" => Ok(GifHeader::GIF87a),
            _ => Err("File format header not supported!")

        }

    }
}

#[derive(Debug)]
pub struct LogicalScreenDescriptor {
    pub canvas_width: u16,
    pub canvas_height: u16,

    // Packed field, maybe nice to have some sort of bitfield instead.
    pub global_color_table_flag: bool,

    // TO-DO: Make this a custom type that fits into the 3-bit range
    pub color_resolution: u16, 

    pub sort_flag: bool,
    pub global_color_table_size: u16,

    pub background_color_index: u8,
    pub pixel_aspect_ratio: u8,
}

pub struct GifFile {
    pub header: GifHeader,
    pub logical_screen_descriptor: LogicalScreenDescriptor,
}

