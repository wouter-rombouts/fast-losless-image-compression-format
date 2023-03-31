//#![feature(int_log)]
use std::env;

use std::io::prelude::*;
use std::fs;
use std::io;
use std::time::Instant;
mod code;
pub mod hfe;
pub mod bitwriter;
pub mod bitreader;

fn main()
{
    
    let args: Vec<String> = env::args().collect();
    let a_file_from = &args[1];
    let mut a_file_to = String::from(&args[2]);
    const NICE_FILE_EXT:  &str=".nice";
    const PNG_FILE_EXT:  &str=".png";
    const DUMP_FILE: &str=".\\dump.bin";
    println!("a_file_from: {}", a_file_from);
    println!("a_file_to: {}", a_file_to);
    if str::ends_with(a_file_from, PNG_FILE_EXT) {
        // The decoder is a build for reader and can be used to set various decoding options
        // via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
        let decoder = png::Decoder::new(fs::File::open(a_file_from).unwrap());
        let mut reader = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size()];
        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut buf).unwrap();
        let channels: u8;
        match info.color_type {
            png::ColorType::Rgba => channels = 4,
            png::ColorType::Rgb => channels = 3,
            _ => panic!("unsupported color type"),
        }
        // Grab the bytes of the image.
        let bytes = &mut buf[..info.buffer_size()];
        let mut dumpfile = io::BufWriter::new(fs::File::create(DUMP_FILE).unwrap());
        let now = Instant::now();
        for i in 0..16
        {
            dumpfile.write_all(&bytes[bytes.len()/16*i..bytes.len()/16*(i+1)]).unwrap();
        }
        
        let after = now.elapsed().as_millis();
        println!("dump: {}", after);
        if !str::ends_with(&a_file_to, NICE_FILE_EXT)
        {
            a_file_to.push_str(NICE_FILE_EXT);
        }
        println!("bytes length: {}", bytes.len());
        let mut my_output= Vec::new();

        let mut my_file = fs::File::create(a_file_to).expect("Error creating output file");

        let now = Instant::now();
        code::encode(bytes, code::Image{width:info.width, height:info.height, channels:channels},channels, &mut my_output).expect("Could not encode Nice");
        
        println!("{}", now.elapsed().as_millis());
        println!("read png file: {}", a_file_from);
        /*for i in 35999992..36000013
        {
            println!("i: {}", my_output[i]);
        }*/
        my_file.write_all(&(my_output)[..]).expect("could not write");
    }
    else {
        if str::ends_with( a_file_from, NICE_FILE_EXT )
        {  
            //let mut file_reader = io::BufReader::new( fs::File::open(a_file_from).expect("Error opening input file") );
            //TODO use enum for channels, make optional
            //let mut dump :Vec<u8> = Vec::new();
            //fs::File::open(DUMP_FILE).unwrap().read_to_end( &mut dump ).ok();
            let mut input :Vec<u8> = Vec::new();
            fs::File::open(a_file_from).unwrap().read_to_end( &mut input ).ok();
            let before_nice = Instant::now();
            let mut output_vec : Vec<u8> = Vec::new();
            //TODO decode from memory
            let image = code::decode( &mut & input[..] , 3,&mut output_vec).expect("Could not decode Nice");
            
            //println!("length: {}", imagebytes.bytes.len());
            println!("nice elapsed in: {}", before_nice.elapsed().as_millis());

            /*#[cfg(debug_assertions)]
            for (i,dump_byte) in fs::File::open("dump.bin").unwrap().bytes().enumerate()
            {
                let dump_byte=dump_byte.unwrap();
                if output_vec[i]!=dump_byte
                {
                    panic!("position {} has value {}, expected {}",i,output_vec[i],dump_byte);
                }
            }*/
            
            //println!("read nice file width: {}", width);
            //println!("read nice file height: {}", height);
            //println!("read nice file channels: {}", channels);
            if !str::ends_with(&a_file_to, PNG_FILE_EXT){
                a_file_to.push_str(PNG_FILE_EXT);
            }
            //see https://docs.rs/png/latest/png/
            let file = fs::File::create(a_file_to).unwrap();
            let ref mut w = io::BufWriter::new(file);
            let now = Instant::now();
            let mut encoder = png::Encoder::new(w, image.width, image.height);
            encoder.set_color(png::ColorType::Rgb);
            //encoder.set_depth(png::BitDepth::One);
            //encoder.set_compression(png::Compression::Best);
            //encoder.set_filter(png::FilterType::Paeth);
            //encoder.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);

            /*encoder.set_trns(vec!(0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8));
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
            let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
                (0.31270, 0.32900),
                (0.64000, 0.33000),
                (0.30000, 0.60000),
                (0.15000, 0.06000)
            );
            encoder.set_source_chromaticities(source_chromaticities);*/
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&output_vec[..]).unwrap();
            
            println!("png{}", now.elapsed().as_millis());
        }
    }
    // Inspect more details of the last read frame.
    //TODO ignore animation
    //let in_animation = reader.info().frame_control.is_some();
    
}
