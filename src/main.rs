use std::env;
use std::fs;
use std::io;

use png;

fn main() -> io::Result<()> {
    // parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} [filename] [greyscale filename]", args[0]);
        return Ok(())
    }

    // decode input file header
    let f = fs::File::open(args[1].as_str())?;
    let decoder = png::Decoder::new(f);
    let (out_info, mut reader) = decoder.read_info()?;

    println!("OUT INFO: {:?}", out_info);

    // read image data to a buffer
    let mut buff: Vec<u8> = vec![0; reader.output_buffer_size()];
    reader.next_frame(buff.as_mut_slice())?;
    
    // allocate new buffer for greyscale
    let mut grayscale_buff: Vec<u8> = vec![0; buff.len() / 3];

    // calculate greyscale as simple average of R G and B values
    for i in 0..grayscale_buff.len() {
        let j = i * 3;
        grayscale_buff[i] = buff[j] / 3 + buff[j + 1] / 3 + buff[j + 2] / 3;
    }

    let mut grayscale_float_buff: Vec<f32> = vec![0.0; grayscale_buff.len()];
    for i in 0..grayscale_float_buff.len() {
        grayscale_float_buff[i] = (grayscale_buff[i] as f32) / 256.0;
    }

    // sobel
    let width = out_info.width as isize;
    let height = out_info.height as isize;
    let mut sobel_buff1: Vec<f32> = vec![0.0; grayscale_float_buff.len()];
    let mut sobel_buff2: Vec<f32> = vec![0.0; grayscale_float_buff.len()];

    let sobel_mat1: [[f32; 3]; 3] = [
        [  1.0,  2.0,  1.0 ],
        [  0.0,  0.0,  0.0 ],
        [ -1.0, -2.0, -1.0 ]
    ];

    let sobel_mat2: [[f32; 3]; 3] = [
        [ -1.0,  0.0,  1.0 ],
        [ -2.0,  0.0,  2.0 ],
        [ -1.0,  0.0,  1.0 ]
    ];

    for row in 0isize..height {
        for col in 0isize..width {
            let i = (row * width + col) as usize;
            sobel_buff1[i] = 0.0;
            sobel_buff2[i] = 0.0;

            for x in 0..3 {
                for y in 0..3 {
                    let j = (row + y) * width + (col + x);
                    if j < 0 || j >= (height * width) { continue };
                    
                    let j = j as usize;
                    let x = x as usize;
                    let y = y as usize;
                    sobel_buff1[i] += grayscale_float_buff[j] * sobel_mat1[y][x];
                    sobel_buff2[i] += grayscale_float_buff[j] * sobel_mat2[y][x];
                }
            }
        }
    }

    let mut output_buff: Vec<u8> = vec![0; sobel_buff2.len()];
    for i in 0..output_buff.len() {
        output_buff[i] = ((sobel_buff1[i] * 255.0).powi(2) + (sobel_buff2[i] * 255.0).powi(2)).sqrt() as u8;
    }

    // encode output file header
    let new_f = fs::File::create(args[2].as_str())?;
    let mut encoder = png::Encoder::new(new_f, out_info.width, out_info.height);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_color(png::ColorType::Grayscale);

    let mut writer = encoder.write_header()?;

    // write image data from buffer to output file
    writer.write_image_data(output_buff.as_slice())?;

    Ok(())
}
