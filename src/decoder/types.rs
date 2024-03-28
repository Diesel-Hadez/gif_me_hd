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

#[derive(Debug)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub type GlobalColorTable = Vec<Pixel>;

pub struct SubBlock<'a> {
    pub size: u8,
    pub data: &'a[u8],
}

#[derive(Debug)]
pub enum DisposalMethod {
  NoDisposal,
  DoNotDispose,
  RestoreToBackground,
  RestoreToPrevious,
}

#[derive(Debug)]
pub enum Extension {
    GraphicsControlExtension {
        reserved: u8,
        disposal_method: DisposalMethod,
        user_input_flag: bool,
        transparent_color_flag: bool,

        delay_timer: u16,
        transparent_color_index: u8,
    },
    PlainText {
        text: String,
    },
    Application{
        identifier: String,
        authentication_code: String,
        // Custom data for application-specific purposes
        data: Vec<u8>, 
    },
    Comment {
        text: String,
    },
}

pub struct GifFile {
    pub header: GifHeader,
    pub logical_screen_descriptor: LogicalScreenDescriptor,
    pub global_color_table: Option<GlobalColorTable>,
    pub extensions: Vec<Extension>,
}

