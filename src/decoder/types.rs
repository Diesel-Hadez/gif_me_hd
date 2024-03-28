#[derive(Debug, PartialEq)]
pub enum GifHeader {
    GIF89a,
    GIF87a,
}
impl GifHeader {
    pub fn from(header: &str) -> Result<GifHeader, &'static str> {
        match header.to_uppercase().as_str() {
            "GIF89A" => Ok(GifHeader::GIF89a),
            "GIF87A" => Ok(GifHeader::GIF87a),
            _ => Err("File format header not supported!"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LogicalScreenDescriptor {
    pub canvas_width: u16,
    pub canvas_height: u16,

    // Packed field, maybe nice to have some sort of bitfield instead.
    pub global_color_table_flag: bool,

    pub color_resolution: u16,

    pub sort_flag: bool,
    pub global_color_table_size: u16,

    pub background_color_index: u8,
    pub pixel_aspect_ratio: u8,
}

#[derive(Debug, PartialEq)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub type GlobalColorTable = Vec<Pixel>;

#[derive(Debug, PartialEq)]
pub enum DisposalMethod {
    NoDisposal,
    DoNotDispose,
    RestoreToBackground,
    RestoreToPrevious,
}

#[derive(Debug, PartialEq)]
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
    Application {
        identifier: String,
        authentication_code: String,
        // Custom data for application-specific purposes
        data: Vec<u8>,
    },
    Comment {
        text: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct ImageDescriptor {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub local_color_table_flag: bool,
    pub interlace_flag: bool,
    pub sort_flag: bool,
    pub reserved: u8,
    pub local_color_table_size: u8,
}

pub type LocalColorTable = Vec<Pixel>;
pub type FrameIndices = Vec<u8>;

#[derive(Debug)]
pub struct GifFrame {
    pub image_descriptor: ImageDescriptor,
    pub local_color_table: Option<LocalColorTable>,
    pub frame_indices: FrameIndices,
    pub extensions: Vec<Extension>,
}

pub struct GifFile {
    pub header: GifHeader,
    pub logical_screen_descriptor: LogicalScreenDescriptor,
    pub global_color_table: Option<GlobalColorTable>,
    pub frames: Vec<GifFrame>,
}
