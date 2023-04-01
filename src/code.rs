const NICE: &[u8] = "nice".as_bytes();
use std::{time::Instant, *};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
pub(crate) const PREFIX_RUN: u8 = 0b00;
pub(crate) const PREFIX_PIXEL_RUN: u8 = 0b0011;
pub(crate) const PREFIX_RED_RUN: u8 = 0b0000;
pub(crate) const PREFIX_GREEN_RUN: u8 = 0b0001;
pub(crate) const PREFIX_BLUE_RUN: u8 = 0b0010;
pub(crate) const PREFIX_RGB: u8 = 0b01;
pub(crate) const PREFIX_BACK_REF: u8 = 0b10;
const PREFIX_RUNS:[u8;3]=[PREFIX_RED_RUN,PREFIX_GREEN_RUN,PREFIX_BLUE_RUN];
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
}

pub fn encode<W: io::Write>(
    input_bytes: &[u8],
    image_header: Image,
    channels_out: u8,
    output_writer: &mut W,
) -> Result<(), io::Error> {
    //write file header
    //let now = Instant::now();
    //bit_writer

    let channels = image_header.channels as usize;
    let mut position = 0usize;
    //write format header
    output_writer.write_all(NICE)?;
    //write width
    output_writer.write_all(
        &[
            image_header.width.to_be_bytes(),
            image_header.height.to_be_bytes(),
        ]
        .concat(),
    )?;
    //write height
    //output_writer.write_all( & )?;
    //write channels outputted
    output_writer.write_all(&[channels_out])?;
    let width = image_header.width as usize;
    let image_size = image_header.height as usize * image_header.width as usize * channels;
    let mut bitwriter = Bitwriter {
        writer: output_writer,
        bit_offset: 0,
        cache: 0,
    };
    //generate lookup table for 16 backref
    /*let lookup_16 = [channels,channels+width*channels,width*channels,width*channels-channels,
    2*channels,2*channels+width*channels,2*channels+width*channels*2,channels+width*channels*2,width*channels*2,width*channels*2-channels,width*channels*2-2*channels,
    3*channels,3*channels+width*channels,3*channels+width*channels*2,3*channels+width*channels*3,2*channels+width*channels*3];*/

    //TODO fill in with most common colors
    //16 size, with 16 spares
    //hold slice or actual values
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [(u8,u8,u8);32] = [(0,0,0);32];

    //main loop
    let mut run_count_red = 1;
    let mut run_count_green = 1;
    let mut run_count_blue = 1;
    let mut back_ref_cntr = 0;
    let mut rgb_cntr = 0;
    while position < image_size
    {
        //only write bytes that are not part of runlength
        //TODO rgb run add case or add as run type
        //TODO when all three have run, special case, or avoid by making pixel run.
        //
        //debug_assert!(run_count_red==1||run_count_green==1||run_count_blue==1);
        //
        let mut temp_count_red = 1;
        let mut temp_count_green = 1;
        let mut temp_count_blue = 1;            

        if run_count_red == 1 || run_count_green == 1 || run_count_blue == 1
        {
            //TODO backreference remaining colors after runlength

            let mut ret_pos=99u8;
            for i in 0..=15
            {
                //check if non run bytes are equal
                if ((run_count_red==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].0==input_bytes[position])||run_count_red != 1)&&
                   ((run_count_green==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].1==input_bytes[position+1])||run_count_green != 1)&&
                   ((run_count_blue==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].2==input_bytes[position+2])||run_count_blue != 1)
                {
                        ret_pos=i as u8;
                        break;
                }
            }
            if ret_pos != 99
            {            

                bitwriter.write_bits_u8(2, PREFIX_BACK_REF)?;
                bitwriter.write_bits_u8(4, ret_pos)?;
                back_ref_cntr+=1;
            }
            else
            {    
                previous16_pixels_unique[previous16_pixels_unique_offset+16]=(input_bytes[position],input_bytes[position + 1],input_bytes[position + 2]);
                previous16_pixels_unique_offset+=1;
                if previous16_pixels_unique_offset==16
                {
                    for i in 0..=15
                    {
                        previous16_pixels_unique[i]=previous16_pixels_unique[i+16];
                    }

                    previous16_pixels_unique_offset=0;
                }
                bitwriter.write_bits_u8(2, PREFIX_RGB)?;
                rgb_cntr+=1;
                if run_count_red == 1
                {
                    debug_assert_eq!(run_count_red, 1);
                    bitwriter.write_bits_u8(8, input_bytes[position])?;
                    //here a run can start

                }        
                if run_count_green == 1
                {
                    debug_assert_eq!(run_count_green, 1);
                    bitwriter.write_bits_u8(8, input_bytes[position + 1])?;

                }
                if run_count_blue == 1
                {
                    debug_assert_eq!(run_count_blue, 1);
                    bitwriter.write_bits_u8(8, input_bytes[position + 2])?;

                }
            }
        }

        if run_count_red > 1
        {
            //skip
            run_count_red -= 1;
        }
        else
        {
            while position + channels * temp_count_red < image_size && input_bytes[position] == input_bytes[position + channels * temp_count_red]
            {
                temp_count_red += 1;
            }
        }

        if run_count_green > 1
        {
            //skip
            run_count_green -= 1;
        }
        else
        {
            while position + channels * temp_count_green < image_size && input_bytes[position + 1] == input_bytes[position + 1 + channels * temp_count_green]
            {
                temp_count_green += 1;
            }
        }


        if run_count_blue > 1
        {
            //skip
            run_count_blue -= 1;
        }
        else 
        {
            while position + channels * temp_count_blue < image_size && input_bytes[position + 2] == input_bytes[position + 2 + channels * temp_count_blue]
            {
                temp_count_blue += 1;
            }
        }

        
        let mut min =temp_count_red.min(temp_count_green).min(temp_count_blue);

        //min is pixel run count offset 1

        if min > 1
        {
            let mut temp_pixel_run_length=min-2;
            min=min-1;
            loop
            {
                bitwriter.write_bits_u8( 8, (PREFIX_PIXEL_RUN<<4)+((temp_pixel_run_length & 0b0000_1111) as u8 ) )?;
                
                if temp_pixel_run_length <16
                {
                   break;
                }
                temp_pixel_run_length = temp_pixel_run_length >> 4;
            }
            temp_count_red-=min;
            temp_count_green-=min;
            temp_count_blue-=min;

            position+=min*channels;


        }

        if temp_count_red > 1
        {
            //add red runlength
            //loop
            run_count_red = temp_count_red - 2;
            loop
            {
                bitwriter.write_bits_u8( 4, PREFIX_RED_RUN )?;
                bitwriter.write_bits_u8( 3, (run_count_red & 0b0000_0111) as u8  )?;
                
                if run_count_red <8
                {
                    break;
                }
                run_count_red = run_count_red >> 3;
            }
            run_count_red = temp_count_red;
        }

        if temp_count_green > 1 {
            //add red runlength
            //loop
            run_count_green = temp_count_green - 2;
            loop {
                bitwriter.write_bits_u8( 4, PREFIX_GREEN_RUN )?;
                bitwriter.write_bits_u8( 3, (run_count_green & 0b0000_0111) as u8  )?;

                

                if run_count_green <8
                {
                    break;
                }
                run_count_green = run_count_green >> 3;
            }
            run_count_green = temp_count_green;
        }

        if temp_count_blue > 1 {
            //add red runlength
            //loop
            run_count_blue = temp_count_blue - 2;
            loop {

                bitwriter.write_bits_u8( 4, PREFIX_BLUE_RUN )?;
                bitwriter.write_bits_u8( 3, (run_count_blue & 0b0000_0111) as u8  )?;

                

                if run_count_blue <8
                {
                    break;
                }
                run_count_blue = run_count_blue >> 3;
            }
            run_count_blue = temp_count_blue;
        }

        //counts end up off by 1
        //let run_pos=run_count_blue*channels;

        //position=run_pos;
        position += channels;
    }
    println!("back_ref_cntr: {}",back_ref_cntr);
    println!("rgb_cntr: {}",rgb_cntr);
     //not used, but to make the dceoder dosen't crash at the end
    bitwriter.write_bits_u8(8, 255)?;
    //bitwriter.write_bits_u8( 8, 255 )?;
    //bitwriter.write_bits_u8( 8, 255 )?;
    //println!("{}", now.elapsed().as_millis());
    Ok(())
}

/*pub struct ImageBytes {
    pub image: Image,
    pub bytes: Vec<u8>,
}*/
//read from file or ...
pub fn decode<R: io::Read>(
    image_reader: &mut R,
    channels_out: u8,
    output_vec: &mut Vec<u8>,
) -> std::io::Result<Image> {
    let now = Instant::now();
    image_reader.read(&mut [0; 4])?;
    let mut buf = [0; 4];
    image_reader.read(&mut buf)?;
    let width = u32::from_be_bytes(buf);
    println!("width:{}", width);
    image_reader.read(&mut buf)?;
    let height = u32::from_be_bytes(buf);

    println!("height:{}", height);
    let mut channels_buf = [0; 1];
    image_reader.read(&mut channels_buf)?;
    let channels = u8::from_be_bytes(channels_buf) as usize;
    println!("channels:{}", channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);
    let image_size = width as usize * height as usize * channels;
    let mut position = 0;
    println!("image_size:{}", image_size);
    *output_vec = Vec::with_capacity(image_size);
    /*unsafe
    {
        output_vec.set_len(image_size);
    }*/

    //TODO write runs with array API
    let mut bitreader = Bitreader {
        reader: image_reader,
        bit_offset: 32,
        cache: 0,
    };
    let mut prefix_2bits: u8 = bitreader.read_bitsu8(2)?;
    let width = width as usize;
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [[u8;3];32] = [[0,0,0];32];
    //curr_lengths[0] is red
    //curr_lengths[1] is green
    //curr_lengths[2] is blue
    //TODO snake + distinct cache + HFE
    #[cfg(debug_assertions)]
    let mut dump= Vec::<u8>::new();
    #[cfg(debug_assertions)]
    io::Read::read_to_end(&mut fs::File::open("dump.bin").unwrap(), &mut dump).ok();
    

    let mut curr_lengths: [usize;3]=[0;3];
    while position < image_size
    {
        
        let color_check=curr_lengths.iter().any(|&x| x == 0);
        let mut back_ref=0;

        if prefix_2bits == PREFIX_BACK_REF && color_check
        {
            back_ref = bitreader.read_bitsu8(4)? as usize;
        }

        for i in 0..=2
        {
            if curr_lengths[i] > 0
            {
                curr_lengths[i] -= 1;
                //println!("position: {}"position);
                output_vec.push(output_vec[position-3+i]);
            }
            else
            {
                if prefix_2bits==PREFIX_BACK_REF
                {   
                    output_vec.push(previous16_pixels_unique[previous16_pixels_unique_offset+back_ref][i]);
                }
                else
                {
                    //if prefix_2bits==PREFIX_RGB
                    //{
                        output_vec.push(bitreader.read_bitsu8(8)?);
                    //}
                }
            }
        }
        
        if prefix_2bits==PREFIX_RGB
        {
            previous16_pixels_unique[previous16_pixels_unique_offset+16]=[output_vec[position],output_vec[position + 1],output_vec[position + 2]];
            previous16_pixels_unique_offset+=1;
            if previous16_pixels_unique_offset==16
            {
                for i in 0..=15
                {
                    previous16_pixels_unique[i]=previous16_pixels_unique[i+16];
                }

                previous16_pixels_unique_offset=0;
            }
        }

        //get run length
        if color_check 
        {
            prefix_2bits = bitreader.read_bitsu8(2)?;
        }

        #[cfg(debug_assertions)]
        {
            //let dump_res=dump.next().unwrap().unwrap();


            debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
            debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
            debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);

        }
            //read full run for 1 run type up to 3 times
        if prefix_2bits == PREFIX_RUN
        {
            let mut run_prefix = bitreader.read_bitsu8(2)?;
            //let mut temp_curr_runcounts:[u8;3]=[0;3];                
            //let mut temp_curr_runcount=0;
            let mut pixel_run_length: usize = 1;
            let mut temp_curr_runcount: u8 = 0;
            
            while prefix_2bits == PREFIX_RUN && run_prefix == PREFIX_PIXEL_RUN
            {
                pixel_run_length += ( bitreader.read_bitsu8(4)? as usize ) << temp_curr_runcount;
                prefix_2bits = bitreader.read_bitsu8(2)?;
                if prefix_2bits == PREFIX_RUN
                {
                    run_prefix = bitreader.read_bitsu8(2)?;
                }
                temp_curr_runcount += 4;

            }

            if temp_curr_runcount >0
            {
                for i in 0..=2
                {
                    curr_lengths[i] = pixel_run_length;
                }
            }

            for i in 0..=2
            {
                temp_curr_runcount=0;
                while prefix_2bits == PREFIX_RUN && run_prefix == PREFIX_RUNS[i]
                {
                    //why 36?
                    //println!("pos: {}, r: {}",position,temp_curr_runcounts[0]);

                    curr_lengths[i] +=(bitreader.read_bitsu8(3)? as usize) << temp_curr_runcount;
                    prefix_2bits = bitreader.read_bitsu8(2)?;
                    if prefix_2bits == PREFIX_RUN
                    {
                        run_prefix = bitreader.read_bitsu8(2)?;
                    }
                    temp_curr_runcount += 3;
                }                              
                if temp_curr_runcount > 0
                {
                    curr_lengths[i] += 1;
                }
            }
        }
        position += channels;
        //}


        //TODO output red run,green run,blue run

    }
    //bitreader.read_bits(8)?;
    println!("{}", now.elapsed().as_millis());
    Ok(Image {
        width:width as u32,
        height,
        channels: channels as u8,
    })
}
