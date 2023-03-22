const NICE : &[u8] = "nice".as_bytes();
use std::{*, time::Instant};

use crate::{bitwriter::Bitwriter, bitreader::Bitreader};
pub(crate) const PREFIX_RUN: u8 = 0b00;
pub(crate) const PREFIX_RED_RUN: u8 = 0b0000;
pub(crate) const PREFIX_GREEN_RUN: u8 = 0b0001;
pub(crate) const PREFIX_BLUE_RUN: u8 = 0b0010;
pub(crate) const PREFIX_RGB: u8 = 0b01;
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
    //write file header
    //let now = Instant::now();
    //bit_writer

    let channels = image_header.channels as usize;
    let mut position =0usize;
    //write format header
    output_writer.write_all(NICE)?;
    //write width
    output_writer.write_all( &[image_header.width.to_be_bytes(),image_header.height.to_be_bytes()].concat() )?;
    //write height
    //output_writer.write_all( & )?;
    //write channels outputted
    output_writer.write_all( &[channels_out] )?;
    let width=image_header.width as usize;
    let image_size = image_header.height as usize * image_header.width as usize * channels;
    let mut bitwriter = Bitwriter{ writer : output_writer,
        bit_offset : 0,
        cache : 0
     };
    //generate lookup table for 16 backref
    /*let lookup_16 = [channels,channels+width*channels,width*channels,width*channels-channels,
                     2*channels,2*channels+width*channels,2*channels+width*channels*2,channels+width*channels*2,width*channels*2,width*channels*2-channels,width*channels*2-2*channels,
                     3*channels,3*channels+width*channels,3*channels+width*channels*2,3*channels+width*channels*3,2*channels+width*channels*3];*/
    //main loop
    let mut run_count_red=1;
    let mut run_count_green=1;
    let mut run_count_blue=1;
    while position<image_size
    {


        //only write bytes that are not part of runlength
        //TODO rgb run add case or add as run type
        //TODO when all three have run, special case, or avoid by making pixel run.
        if run_count_red==1||run_count_green==1||run_count_blue==1
        {
            bitwriter.write_bits_u8( 2,PREFIX_RGB )?;
        }
        if run_count_red > 1
        {
            //skip
            run_count_red-=1;
        }
        else
        {
            debug_assert_eq!(run_count_red,1);
            bitwriter.write_bits_u8( 8,input_bytes[position] )?;
            //here a run can start
            while position+channels*run_count_red<image_size && input_bytes[position] == input_bytes[position+channels*run_count_red]
            {
                run_count_red+=1;
            }

            if run_count_red>1
            {
                //add red runlength
                //loop
                let mut temp_run_count_red=run_count_red-2;
                loop
                {
                    bitwriter.write_bits_u8( 8, (PREFIX_RED_RUN<<4)+((temp_run_count_red & 0b0000_1111) as u8 ) )?;               
                    
                    temp_run_count_red = temp_run_count_red >> 4;
    
                    if temp_run_count_red == 0
                    {
                       break;
                    }
                }
            }
        }
        
        if run_count_green > 1
        {
            //skip
            run_count_green-=1;
        }
        else
        {
            debug_assert_eq!(run_count_green,1);
            bitwriter.write_bits_u8( 8,input_bytes[position+1] )?;        
            while position+channels*run_count_red<image_size && input_bytes[position+1] == input_bytes[position+1+channels*run_count_green]
            {
                run_count_green+=1;
            }

            if run_count_green>1
            {
                //add red runlength
                //loop
                let mut temp_run_count_green=run_count_green-2;
                loop
                {
                    bitwriter.write_bits_u8( 8, (PREFIX_GREEN_RUN<<4)+((temp_run_count_green & 0b0000_1111) as u8 ) )?;               
                    
                    temp_run_count_green = temp_run_count_green >> 4;
    
                    if temp_run_count_green == 0
                    {
                       break;
                    }
                }
            }
        }

        
        if run_count_blue > 1
        {
            //skip
            run_count_blue-=1;
        }
        else
        {
            debug_assert_eq!(run_count_blue,1);
            bitwriter.write_bits_u8( 8,input_bytes[position+2] )?;
            while position+channels*run_count_red<image_size && input_bytes[position+2] == input_bytes[position+2+channels*run_count_blue]
            {
                run_count_blue+=1;
            }

            if run_count_blue>1
            {
                //add red runlength
                //loop
                let mut temp_run_count_blue=run_count_blue-2;
                loop
                {
                    bitwriter.write_bits_u8( 8, (PREFIX_BLUE_RUN<<4)+((temp_run_count_blue & 0b0000_1111) as u8 ) )?;               
                    
                    temp_run_count_blue = temp_run_count_blue >> 4;
    
                    if temp_run_count_blue == 0
                    {
                       break;
                    }
                }
            }
        }


        //counts end up off by 1
        //let run_pos=run_count_blue*channels;

        //position=run_pos;
        position+=channels;

    }
    //not used, but to make the dceoder dosen't crash at the end
    bitwriter.write_bits_u8( 8, 255 )?;
    //bitwriter.write_bits_u8( 8, 255 )?;
    //bitwriter.write_bits_u8( 8, 255 )?;
    //println!("{}", now.elapsed().as_millis());
    Ok(())
}


pub struct ImageBytes
{
    pub image:Image,
    pub bytes:Vec<u8>
}
//read from file or ...
pub fn decode<R : io::Read>( image_reader : &mut R,
                             channels_out : u8,
                             output_vec : &mut Vec<u8>
                             )
                                         -> std::io::Result< Image >
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
    let channels = u8::from_be_bytes(channels_buf)as usize;
    println!("channels:{}", channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);
    let image_size  =width as usize * height as usize * channels;
    let mut position =0;
    println!("image_size:{}", image_size);
    *output_vec=Vec::with_capacity(image_size);
    let mut bitreader = Bitreader{reader:image_reader,bit_offset:32,cache:0};
    let mut prefix_2bits :u8=bitreader.read_bitsu8(2)?;
    while position < image_size
    {
        //if prefix_2bits ==

        let mut run_length: usize = 1;
        let mut curr_runcount: u8 = 0;
        //TODO: read more bits
        output_vec.extend_from_slice(&bitreader.read_bits3bytes()?.to_be_bytes()[1..=3]);


        //get run length
        prefix_2bits=bitreader.read_bitsu8(2)?;
        
        loop 
        {
            if prefix_2bits != PREFIX_RUN
            {
                if curr_runcount >0
                {
                    for _ in 0..run_length
                    { 
                        output_vec.extend_from_slice(&[output_vec[position],output_vec[position+1],output_vec[position+2]]);
                    }
        
                    position+=channels*run_length;
               }
               break;
            }

            run_length += ( bitreader.read_bitsu8(4)? as usize ) << curr_runcount;
            prefix_2bits = bitreader.read_bitsu8(2)?;
            curr_runcount += 4;

        }
        
        position+=channels;
    }
    //bitreader.read_bits(8)?;
    println!("{}", now.elapsed().as_millis());
    Ok(  Image { width,
                height,
                channels : channels as u8,
              } )
}