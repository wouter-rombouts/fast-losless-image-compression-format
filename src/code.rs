const NICE : &[u8] = "nice".as_bytes();
use std::{*, time::Instant};

pub struct Image
{
      pub width : u32,
     pub height : u32,
   pub channels : u8,
}

pub fn encode<W : io::Write>(   input_bytes : & [u8],
                               image_header : Image,
                              channels_out  : u8,
                              output_writer : &mut W )
         -> Result< (), io::Error >
{
    /*current position of the input image.*/
    //let mut pos = 0;
    //previous pixel
    //let mut prev_pixel;
    //size of the image in bytes
    //let image_size = height as usize * width as usize * channels_in as usize;
    //let mut buffer = Vec::<u8>::with_capacity(image_size);
    //write file header
    //write format header
    output_writer.write_all(NICE)?;
    //write width
    output_writer.write_all( &image_header.width.to_be_bytes() )?;
    //write height
    output_writer.write_all( &image_header.height.to_be_bytes() )?;
    //write channels outputted
    output_writer.write_all( &[channels_out] )?;

    let now = Instant::now();

    let mut grey_total=0u64;
    let mut grey_avg=0u8;
    let mut i = 0usize;
    //main loop
    for pixel in input_bytes.chunks_exact(image_header.channels as usize)
    {
        //calculate greyscale value
        //let pixel_grey = pixel[0..3].iter().min().unwrap();
        i+=1;
        //grey_avg=(grey_total/i) as u8
        //optimum will be at avg of previous grey values
        //calc diff for each color with grey avg,further is less compressed
        let red_diff=grey_avg.wrapping_sub(pixel[0]);
        let green_diff=grey_avg.wrapping_sub(pixel[1]);
        let blue_diff=grey_avg.wrapping_sub(pixel[2]);
        let smallest_diff;
        //calc min, TODO use iterator?
        if red_diff < green_diff && red_diff < blue_diff
        {
            smallest_diff=red_diff;
        }
        else{
            if blue_diff < green_diff && blue_diff < red_diff
            {
                smallest_diff=blue_diff;
            }
            else{
                smallest_diff=green_diff;
            }
        }
        //round to the upper power of 2 
        let smallest_diff_rounded = smallest_diff.next_power_of_two();
        //reverse to get rgb remainder values
        //wrapping needed when both directions implemented?
        let red_remainder = smallest_diff_rounded.wrapping_sub(red_diff);
        let green_remainder = smallest_diff_rounded.wrapping_sub(green_diff);
        let blue_remainder = smallest_diff_rounded.wrapping_sub(blue_diff);



        //something to write the greyscale
        //buffer.push(*pixel_greyscale);
        output_writer.write_all( &[smallest_diff] )?;
        output_writer.write_all( &[red_remainder] )?;
        output_writer.write_all( &[green_remainder] )?;
        output_writer.write_all( &[blue_remainder] )?;
        
        grey_total=grey_total+(grey_avg.wrapping_sub(smallest_diff)) as u64;


        //output_writer.write_all( &[*pixel_greyscale] )?;
        /*output_writer.write_all( &[channels_out] )?;*/
        //set the previous byte for next loop iteration
        //needed if last one??
        //prev_pixel = pixel;
    }
    println!("{}", now.elapsed().as_millis());
    //output_writer.write_all(&buffer)?;
    Ok(())
}


pub struct ImageBytes
{
    pub image:Image,
    pub bytes:Vec<u8>
}
//read from file or ...
pub fn decode<R : io::Read>( mut image_reader : R,
                             channels_out : u8 )
                                         -> std::io::Result< ImageBytes >
{
    image_reader.read( &mut [0; 4] )?;
    let mut buf = [0; 4];
    image_reader.read( &mut buf )?;
    let width = u32::from_be_bytes(buf);
    image_reader.read( &mut buf )?;
    let height = u32::from_be_bytes(buf);
    let mut channels = [0; 1];
    image_reader.read( &mut channels )?;
    let channels = u8::from_be_bytes(channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);

    let mut output_vec : Vec<u8> = Vec::with_capacity(width as usize * height as usize * channels as usize);
            /*unsafe
            {
                my_out_vec.set_len(image_size);
            }*/
    //dummy
    output_vec.push(0);
    Ok( ImageBytes{image : Image { width,
                height,
                channels,
              },bytes : output_vec} )
}