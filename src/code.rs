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
    //let mut buffer = Vec::<u8>::with_capacity(image_size);
    //write file header
    let now = Instant::now();
    let channels = image_header.channels as usize;
    let mut position =0usize;
    //write format header
    output_writer.write_all(NICE)?;
    //write width
    output_writer.write_all( &image_header.width.to_be_bytes() )?;
    //write height
    output_writer.write_all( &image_header.height.to_be_bytes() )?;
    //write channels outputted
    output_writer.write_all( &[channels_out] )?;
    let image_size = image_header.height as usize * image_header.width as usize * channels;
    
    
    //main loop
    while position<image_size
    {

        let mut run_pos=position;

        output_writer.write_all( &input_bytes[position..position+3] )?;
        
        loop 
        {
            run_pos+=channels;
            if run_pos>=image_size||input_bytes[position..position+3] != input_bytes[run_pos..run_pos+3]
            {
                break;
            }
        }
        //loop ends on first pixel outside the run
        if run_pos-position>channels
        {
            //write runlength
            output_writer.write_all( &((run_pos-position)/channels).to_be_bytes()[0..1] )?;
            position=run_pos;
        }
        position+=channels;

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
    let now = Instant::now();
    image_reader.read( &mut [0; 4] )?;
    let mut buf = [0; 4];
    image_reader.read( &mut buf )?;
    let width = u32::from_be_bytes(buf);
    println!("width:{}", width);
    image_reader.read( &mut buf )?;
    let height = u32::from_be_bytes(buf);
    
    println!("height:{}", height);
    let mut channels_buf = [0; 1];
    image_reader.read( &mut channels_buf )?;
    let channels = u8::from_be_bytes(channels_buf);
    println!("channels:{}", channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);
    let image_size  =width as usize * height as usize * channels as usize;
    
    println!("image_size:{}", image_size);
    let mut output_vec : Vec<u8> = Vec::with_capacity(image_size);
    let mut pos =0;
    let mut buffer = [0; 3];
    while image_reader.read_exact(&mut buffer).is_ok()
    {
        
        output_vec.push(buffer[0]);
        output_vec.push(buffer[1]);
        output_vec.push(buffer[2]);
    }
    println!("{}", now.elapsed().as_millis());
    Ok( ImageBytes{image : Image { width,
                height,
                channels,
              },bytes : output_vec} )
}