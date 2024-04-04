use gif_me_hd::decoder::{self, GifFile};
use std::env;
use std::fs;

fn get_ppm_representation(image: GifFile) -> Vec<(usize, String)> {
    let mut ret: Vec<(usize, String)> = Vec::new();
    let width = image.logical_screen_descriptor.canvas_width;
    let height = image.logical_screen_descriptor.canvas_height;
    let gct: Vec<gif_me_hd::decoder::Pixel> = image.global_color_table.unwrap();
    let mut cur_color_table = gct;
    let mut transparent_color_index = 0;
    let mut prev_frame: Option<Vec<Vec<u8>>> = None;

    for (idx, cur_frame) in image.frames.into_iter().enumerate() {
        let prev_str = format!("P3\n{} {}\n255\n", width, height);
        cur_color_table = match cur_frame.local_color_table {
            Some(table) => table,
            None => cur_color_table,
        };
        let frame_data: Vec<Vec<u8>> = cur_frame.frame_indices
            .iter()
            .map(|x| *(cur_color_table.get(*x as usize).unwrap()))
            .map(|x| vec![x.red, x.green, x.blue])
            .collect();

        let to_x_y  = |pos: usize, width: u16| {
            let x: u16 = (pos % (width as usize)) as u16;
            let y: u16 = ((pos - (x as usize)) / (width as usize)) as u16;
            (x,y)
        };

        let to_x_y_global = |pos: usize| {
            to_x_y(pos, image.logical_screen_descriptor.canvas_width)
        };

        
        let frame = match prev_frame {
            Some(prev_frame) => {
                         prev_frame
                         .into_iter()
                        .enumerate()
                        .collect::<Vec<(usize, Vec<u8>)>>()
                        .into_iter()
                        .map(|(pos, val)| -> Vec<u8> {
                            let (x, y) = to_x_y_global(pos);
                            if x >= cur_frame.image_descriptor.left && 
                                x < cur_frame.image_descriptor.left
                                    + cur_frame.image_descriptor.width &&
                                y >= cur_frame.image_descriptor.top &&
                                y < cur_frame.image_descriptor.top 
                                    + cur_frame.image_descriptor.height {
                                        let local_x = x - cur_frame.image_descriptor.left;
                                        let local_y = y - cur_frame.image_descriptor.top;
                                        let end_result = vec![((local_x as f32 / cur_frame.image_descriptor.width as f32) * 255.0) as u8,
                                        0, 0, 255];
                                        return frame_data
                                            .get(((local_y as usize)*(cur_frame.image_descriptor.width as usize)+(local_x as usize)) as usize)
                                            .unwrap()
                                            .to_vec();
                                    }
                            val.to_vec()
                        })
                        .map(|val| val.into_iter().map(|hmm| format!("{}", hmm)).collect::<Vec<String>>().join(","))
                        .collect::<Vec<String>>()
                        .join("\n")
            },
            None => {
                    frame_data
                        .clone()
                        .into_iter()
                        .map(|val| val.into_iter().map(|hmm| format!("{}", hmm)).collect::<Vec<String>>().join(","))
                        .collect::<Vec<String>>()
                        .join("\n")
            },
        };
    
        prev_frame = Some(frame_data);
        ret.push((idx, format!("{}{}", prev_str, frame)));
    }
    
    ret.into()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Not enough arguments!");
    }
    let file = &args[1];
    let gif_file = decoder::load(&file[..]).unwrap();
    // println!(
    //     "Logical Screen Descriptor: {:#?}",
    //     gif_file.logical_screen_descriptor
    // );
    // match gif_file.global_color_table {
    //     Some(gct) => println!("Global Color Table: {:#?}", gct),
    //     None => println!("No Global Color Table"),
    // }
    // gif_file.frames.into_iter().for_each(|frame| {
    //     println!("Frame: {:#?}", frame);
    // });
    let images = get_ppm_representation(gif_file);
    for (idx, image) in images {
        fs::write(format!("image_{}.ppm", idx), image).expect("Unable to write file");
    }
}
