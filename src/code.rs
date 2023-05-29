const NICE: &[u8] = "nice".as_bytes();
use crate::bitwriter;
use crate::image::{Image, self};
use std::{time::Instant, *};
use crate::hfe::{EncodedOutput, self};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
pub(crate) const PREFIX_RUN: u8 = 3;
pub(crate) const PREFIX_RED_RUN: u8 = 4;
pub(crate) const PREFIX_GREEN_RUN: u8 = 5;
pub(crate) const PREFIX_BLUE_RUN: u8 = 6;
pub(crate) const PREFIX_RGB: u8 = 0;
pub(crate) const PREFIX_BACK_REF: u8 = 2;
pub(crate) const PREFIX_COLOR_LUMA: u8 = 1;
const PREFIX_RUNS:[u8;3]=[PREFIX_RED_RUN,PREFIX_GREEN_RUN,PREFIX_BLUE_RUN];

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
            (image_header.width as u32).to_be_bytes(),
            (image_header.height as u32).to_be_bytes(),
        ]
        .concat(),
    )?;
    //write height
    //output_writer.write_all( & )?;
    //write channels outputted
    output_writer.write_all(&[channels_out])?;
    let image_size = image_header.height as usize * image_header.width as usize * channels;
    //pre entropy encoding and after output vector
    let mut data =EncodedOutput::new( image_size );

    //initialize all output streams
    //RGB must be 0==PREFIX_RGB
    data.add_output_type(256);

    //TODO fill in with most common colors
    //16 size, with 16 spares
    //hold slice or actual values
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [(u8,u8,u8);64] = [(0,0,0);64];

    //main loop
    let mut run_count_red = 1;
    let mut run_count_green = 1;
    let mut run_count_blue = 1;
    let mut back_ref_cntr = 0;
    let mut rgb_cntr = 0;
    let mut run_cntr=0;
    let mut luma_occurences=0;
    let mut red_pixel_run_amount=0;
    let mut run_occurrences=[0;8];
    //let mut run_lookup_table=[1,2,3,4,5,6,7,7];

    for loop_index in 0..image_size/channels
    {

        //TODO move to end, only when run for color was not recetnly added
        //let is_not_in_run=run_count_red ==1 ||run_count_green==1||run_count_blue==1;
        let prev_position = position;
        position=image_header.calc_pos_from(loop_index)*channels;  

        if position==468285{
            dbg!(position);
            dbg!(prev_position);
            dbg!(run_count_red);
            dbg!(run_count_green);
            dbg!(run_count_blue);
        }
        if run_count_red ==1 ||run_count_green==1||run_count_blue==1
        {
            //prev_position=position;      
            //TODO backreference remaining colors after runlength
            let mut ret_pos=99u8;

            for i in 0..=31
            {
                //check if non run bytes are equal
                if ((run_count_red==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].0==input_bytes[position])||run_count_red > 1)&&
                   ((run_count_green==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].1==input_bytes[position+1])||run_count_green > 1)&&
                   ((run_count_blue==1&&previous16_pixels_unique[i+previous16_pixels_unique_offset].2==input_bytes[position+2])||run_count_blue > 1)
                {
                        ret_pos=i as u8;
                        break;
                }
            }
            if ret_pos != 99
            {
                
                data.add_symbolu8(PREFIX_BACK_REF, PREFIX_RGB);
                data.add_symbolu8(ret_pos, PREFIX_RGB);
                back_ref_cntr+=1;
            }
            else
            {
                //write to unique previous n cache         
                previous16_pixels_unique[previous16_pixels_unique_offset+32]=(input_bytes[position],input_bytes[position + 1],input_bytes[position + 2]);
                previous16_pixels_unique_offset+=1;
                if previous16_pixels_unique_offset==32
                {
                    for i in 0..=31
                    {
                        previous16_pixels_unique[i]=previous16_pixels_unique[i+32];
                    }

                    previous16_pixels_unique_offset=0;
                }
                //TODO check color diff
                //TODO only check for non run colors

                let mut list_of_color_diffs=[i16::MAX;3];

                //must be smaller than +-2 = 2bits
                //red diff 0-8 = 3bits
                if run_count_green==1
                {
                    list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[prev_position+1] as i16;
                }
                
                if run_count_red==1
                {
                    //red_diff
                    list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[prev_position] as i16;
                };
                //if run_count_green==1
                {
                    //green_diff
                    
                };
                if run_count_blue==1
                {
                    //blue_diff
                    list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[prev_position+2] as i16;
                };
                let mut first=i16::MAX;
                let mut is_luma=true;
                for i in 0..=2
                {
                    if list_of_color_diffs[i]!=i16::MAX
                    {
                        if first==i16::MAX
                        {
                            first=list_of_color_diffs[i];
                            if !(first>=-16&&first<16)
                            {
                                is_luma=false;
                            }
                        }
                        else
                        {
                            list_of_color_diffs[i]=list_of_color_diffs[i]-first;
                            if !(list_of_color_diffs[i]>=-8&&list_of_color_diffs[i]<8)
                            {
                                is_luma=false;
                            }
                        }
                    }
                }
                //let no_first=list_of_color_diffs.iter().filter(|&&x| {if first==i16::MAX&& x !=i16::MAX {first=x}; x !=i16::MAX});
                //.map(|x| if first==i16::MAX&& *x !=i16::MAX{first=*x;*x}else{*x-first} );
                //TODO create luminosity field run, for rgb?
                //avoid short lumo, while having high accuracy. how to detect?upper limit for remainder?
                //when rgb or diff, calc lumo level, if not in +-8, go to other color layer(,write to output)
                //take color layer relative to each other to optimize?
                //TODO compare green with backref luma, no scope reference to distance

                //TODO color layer ,color run when diff is zero,break per 8?, always has color layer, so need for length.
                //take diff from previous color layer
                //TODO compare to color layer in back ref unique table,diff diff in color layer


                /*let mut mini=0;
                for i in 1..list_of_color_diffs.len()
                {
                    if list_of_color_diffs[i]<list_of_color_diffs[mini]
                    {
                        mini=i;
                    }
                }

                let min=list_of_color_diffs[mini];*/
                //let mut valsonly=list_of_color_diffs.iter().filter(|&&s| s !=i16::MAX&& s !=min ).map(|&x|x-min);
                //TODO: dynamic luma: the closer the luma, the bigger the diff allowed, leave rgb as is?
                //prevpix+- amount +leftover together max n bits
                //ttrytofix ,luma offset+diff, ...
                
                //TODO: wrap around 256/0 logic
                //TODO check 4 backreference

                /*if position>0&&is_luma
                {
                    //bitwriter.write_bits_u8(2, (dif_base+2) as u8)?;
                    data.add_symbolu8(PREFIX_COLOR_LUMA);
                    data.add_symbolu8((first+16) as u8);

                    for i in 0..=2
                    {
                        if list_of_color_diffs[i]!=i16::MAX&&list_of_color_diffs[i]!=first
                        {
                            data.add_symbolu8((list_of_color_diffs[i]+8) as u8);
                        }
                    }
                    

                    luma_occurences+=1;
                }
                else*/
                {
                    //write rgb
                    if position==468279{
                        println!("rgb");
                        dbg!(input_bytes[position]);
                        dbg!(input_bytes[position+1]);
                        dbg!(input_bytes[position+2]);
                        dbg!(run_count_red);
                        dbg!(run_count_green);
                        dbg!(run_count_blue);
                    }
                    data.add_symbolu8(PREFIX_RGB, PREFIX_RGB);

                    rgb_cntr+=1;
                    if run_count_red == 1
                    {
                        data.add_symbolu8(input_bytes[position], PREFIX_RGB);
                    }        
                    if run_count_green == 1
                    {
                        data.add_symbolu8(input_bytes[position+1], PREFIX_RGB);

                    }
                    if run_count_blue == 1
                    {
                        data.add_symbolu8(input_bytes[position+2], PREFIX_RGB);

                    }
                }
            }            
        }

        if run_count_red>1
        {
            run_count_red-=1;
        }
        if run_count_green>1
        {
            run_count_green-=1;
        }
        if run_count_blue>1
        {
            run_count_blue-=1;
        }
        
        /*if run_count_blue==1&&run_count_green==1&&run_count_red==1
        {
            let mut pixel_run_length = 0;
            let mut pixel_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
            
            while pixel_run_loop_position<image_size&&input_bytes[pixel_run_loop_position]==input_bytes[position]&&
                  input_bytes[pixel_run_loop_position+1]==input_bytes[position+1]&&
                  input_bytes[pixel_run_loop_position+2]==input_bytes[position+2]
            {
                pixel_run_length+=1;
                pixel_run_loop_position=image_header.calc_pos_from(loop_index+pixel_run_length+1)*channels;
            }
            // these pixels will be skipped in the next iterations
            //TODO test if actually skipping all at once is faster
            run_count_red+=pixel_run_length;
            run_count_green+=pixel_run_length;
            run_count_blue+=pixel_run_length;

            if pixel_run_length > 0
            {
                if loop_index==00
                {
                    dbg!(pixel_run_loop_position);
                    debug_assert_eq!(pixel_run_length,140352);
                }
                run_cntr+=pixel_run_length;
                pixel_run_length=pixel_run_length-1;
                //position = pixel_run_loop_position;
                loop
                {
                    bitwriter.write_bits_u8( 8, (PREFIX_PIXEL_RUN<<4)+((pixel_run_length & 0b0000_1111) as u8 ) )?;
                    
                    if pixel_run_length <16
                    {
                    break;
                    }
                    pixel_run_length = pixel_run_length >> 4;
                }

                //position+=min*channels;


            }
        }*/

            //check for color run
            if run_count_red==1
            {
                let mut red_run_length = 0;
                let mut prev_red_run_loop_position=position;
                let mut red_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while red_run_loop_position<image_size&&
                      input_bytes[red_run_loop_position]==input_bytes[position]/*&&
                      input_bytes[red_run_loop_position+1]!=input_bytes[prev_red_run_loop_position+1]&&
                      input_bytes[red_run_loop_position+2]!=input_bytes[prev_red_run_loop_position+2]*/
                {
                    red_run_length+=1;
                    prev_red_run_loop_position=red_run_loop_position;
                    red_run_loop_position=image_header.calc_pos_from(loop_index+red_run_length+1)*channels;
                }

                if red_run_length > 0
                {
                    //add red runlength
                    //loop
                    run_count_red+=red_run_length;
                    red_pixel_run_amount+=red_run_length;
                    red_run_length = red_run_length - 1;
                    run_cntr+=1;
                    loop
                    {
                        
                        data.add_symbolu8(PREFIX_RUN, PREFIX_RGB);
                        data.add_symbolu8(PREFIX_RED_RUN, PREFIX_RGB);
                        data.add_symbolu8((red_run_length & 0b0000_0111).try_into().unwrap(), PREFIX_RGB);
                        run_occurrences[(red_run_length & 0b0000_0111)]+=1;
                        if red_run_length <8
                        {
                            break;
                        }
                        red_run_length = red_run_length >> 3;
                    }
                }
            }

            if run_count_green==1
            {
                let mut green_run_length = 0;
                let mut prev_green_run_loop_position=position;
                let mut green_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while green_run_loop_position<image_size&&
                      input_bytes[green_run_loop_position+1]==input_bytes[position+1]/*&&
                      input_bytes[green_run_loop_position]!=input_bytes[prev_green_run_loop_position]&&
                      input_bytes[green_run_loop_position+2]!=input_bytes[prev_green_run_loop_position+2]*/
                {
                    green_run_length+=1;
                    prev_green_run_loop_position=green_run_loop_position;
                    green_run_loop_position=image_header.calc_pos_from(loop_index+green_run_length+1)*channels;
                }

                if green_run_length > 0
                {
                    //add green runlength
                    //loop
                    run_count_green+=green_run_length;
                    green_run_length = green_run_length - 1;
                    run_cntr+=1;
                    loop
                    {
                        data.add_symbolu8(PREFIX_RUN, PREFIX_RGB);
                        data.add_symbolu8(PREFIX_GREEN_RUN, PREFIX_RGB);
                        data.add_symbolu8((green_run_length & 0b0000_0111).try_into().unwrap(), PREFIX_RGB);
                        run_occurrences[(green_run_length & 0b0000_0111)]+=1;
                        if green_run_length <8
                        {
                            break;
                        }
                        green_run_length = green_run_length >> 3;
                    }
                }
            }

            if run_count_blue==1
            {
                let mut blue_run_length = 0;
                let mut prev_blue_run_loop_position=position;
                let mut blue_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while blue_run_loop_position<image_size&&
                      input_bytes[blue_run_loop_position+2]==input_bytes[position+2]/*&&
                      input_bytes[blue_run_loop_position+1]!=input_bytes[prev_blue_run_loop_position+1]&&
                      input_bytes[blue_run_loop_position]!=input_bytes[prev_blue_run_loop_position]*/
                {
                    blue_run_length+=1;
                    prev_blue_run_loop_position=blue_run_loop_position;
                    blue_run_loop_position=image_header.calc_pos_from(loop_index+blue_run_length+1)*channels;
                }

                if blue_run_length > 0
                {
                    //add blue runlength
                    //loop
                    run_count_blue+=blue_run_length;
                    blue_run_length = blue_run_length - 1;
                    run_cntr+=1;
                    loop
                    {
                        data.add_symbolu8(PREFIX_RUN, PREFIX_RGB);
                        data.add_symbolu8(PREFIX_BLUE_RUN, PREFIX_RGB);
                        data.add_symbolu8((blue_run_length & 0b0000_0111).try_into().unwrap(), PREFIX_RGB);
                        run_occurrences[(blue_run_length & 0b0000_0111)]+=1;
                        if blue_run_length <8
                        {
                            break;
                        }
                        blue_run_length = blue_run_length >> 3;
                    }
                }
            }

            
        //after adding non run colors

        //#[cfg(debug_assertions)]
        //{
        //loop_index+=1;
        //}

        //position = loop_index;
        
    }
    //TODO merge into 1 output
    //dbg!(data.data_vec.len());

    let mut bitwriter=Bitwriter::new(output_writer);
    data.to_encoded_output(&mut bitwriter)?;
    //TODO: write cache to writer?
    let cache=bitwriter.cache.to_be_bytes();
    bitwriter.writer.write_all(&cache)?;
    //data.data_vec.extend_from_slice(&cache[..]);
    //dbg!(data.data_vec.len());
    //handle in hfe
    //output_writer.write_all(&data.data_vec)?;

    //}
    println!("back_ref_cntr: {}",back_ref_cntr);
    println!("rgb_cntr: {}",rgb_cntr);
    println!("run_cntr: {}",run_cntr);
    println!("luma_occurences: {}",luma_occurences);
    println!("red_pixel_run_amount:{}",red_pixel_run_amount);
    println!("run_occurrences:{:?}",run_occurrences);
    
     //not used, but to make the dceoder dosen't crash at the end
     //output_writer.write_all(&[255])?;
     
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
    dbg!(width);
    image_reader.read(&mut buf)?;
    let height = u32::from_be_bytes(buf);

    dbg!( height);
    
    let height = height as usize;
    let mut channels_buf = [0; 1];
    image_reader.read(&mut channels_buf)?;
    let channels = u8::from_be_bytes(channels_buf) as usize;
    dbg!( channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);
    let image_size = width as usize * height as usize * channels;
    let image =Image::new(
        width as usize,
        height,
         channels as u8,
    );
    let mut position = 0;
    dbg!(image_size);
    *output_vec = Vec::with_capacity(image_size);
    unsafe
    {
        output_vec.set_len(image_size);
    }

    dbg!(output_vec.len());
    //TODO push output to Vec
    //let mut bitreader = Bitreader::new(image_reader);
    let mut decoder=  hfe::DecodeInput::new(Bitreader::new(image_reader));
    decoder.read_header_into_tree(256).unwrap();

    //let mut prefix_1bits=bitreader.read_bitsu8(1)?;
    //let mut prefix_2bits: u8=bitreader.read_bitsu8(1)?;

    let mut prefix1=decoder.read_next_symbol()?;
    let width = width as usize;
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [[u8;3];64] = [[0,0,0];64];
    let mut prev_position = 0;
    //curr_lengths[0] is red
    //curr_lengths[1] is green
    //curr_lengths[2] is blue
    //TODO snake + distinct cache + HFE

    #[cfg(debug_assertions)]
    let mut dump= Vec::<u8>::new();
    #[cfg(debug_assertions)]
    io::Read::read_to_end(&mut fs::File::open("dump.bin").unwrap(), &mut dump).ok();
    let pos_subblock_lookup = [0,channels,2*channels,3*channels,4*channels,
                               (4+image.width)*channels,(3+image.width)*channels,(2+image.width)*channels,(1+image.width)*channels,(image.width)*channels,
                               (2*image.width)*channels,(1+2*image.width)*channels,(2+2*image.width)*channels,(3+2*image.width)*channels,(4+2*image.width)*channels,
                               (4+3*image.width)*channels,(3+3*image.width)*channels,(2+3*image.width)*channels,(1+3*image.width)*channels,(3*image.width)*channels,
                               (4*image.width)*channels,(1+4*image.width)*channels,(2+4*image.width)*channels,(3+4*image.width)*channels,(4+4*image.width)*channels];
    let mut pos_subblock_xleftover_lookup: Vec<usize>;
    let mut pos_subblock_yleftover_lookup: Vec<usize>;
    let mut list_of_subblocks_in_widthblock:Vec<&[usize]>=Vec::with_capacity(image.subblocks_in_width+1);
    let mut list_of_subblocks_in_bottom_widthblock:Vec<&[usize]>=Vec::with_capacity(image.subblocks_in_width+1);
    let mut pos_subblock_xyleftover_lookup: Vec<usize>;
    for i in 0..image.subblocks_in_width
    {
        list_of_subblocks_in_widthblock.push(&pos_subblock_lookup);
    }
    if image.width_subblock_leftover>0
    {
        pos_subblock_xleftover_lookup=Vec::with_capacity(image.width_subblock_leftover*image::SUBBLOCK_HEIGHT_MAX);
        for h in 0..image::SUBBLOCK_HEIGHT_MAX
        {
            //pos_subblock_xleftover_lookup.extend((0..image.width_subblock_leftover).map(|i|(h*image.width+if h%2==1{image.width_subblock_leftover-i-1}else{ i})*channels));
            for i in 0..image.width_subblock_leftover
            {
                pos_subblock_xleftover_lookup.push((h*image.width+if h%2==1{image.width_subblock_leftover-i-1}else{ i})*channels);
            }
        }
        list_of_subblocks_in_widthblock.push(&pos_subblock_xleftover_lookup);
    }
    //TODO push list_of_subblocks_in_widthblock over the height except leftover
    let mut list_of_subblocks_in_heightblock:Vec<&Vec<&[usize]>>=Vec::with_capacity(image.subblocks_in_height+1);
    //add top width blocks
    for i in 0..image.subblocks_in_height
    {
        list_of_subblocks_in_heightblock.push(&list_of_subblocks_in_widthblock);
    }
    //bottom width block
    if image.height_subblock_leftover>0
    {
        pos_subblock_yleftover_lookup=Vec::with_capacity(image.height_subblock_leftover*image::SUBBLOCK_WIDTH_MAX);
        for h in 0..image.height_subblock_leftover
        {
            //pos_subblock_xleftover_lookup.extend((0..image.width_subblock_leftover).map(|i|(h*image.width+if h%2==1{image.width_subblock_leftover-i-1}else{ i})*channels));
            for i in 0..image::SUBBLOCK_WIDTH_MAX
            {
                pos_subblock_yleftover_lookup.push((h*image.width+if h%2==1{image::SUBBLOCK_WIDTH_MAX-i-1}else{ i})*channels);
            }
        }
        list_of_subblocks_in_bottom_widthblock.push(&pos_subblock_yleftover_lookup);
        if image.width_subblock_leftover>0
        {
            pos_subblock_xyleftover_lookup=Vec::with_capacity(image.height_subblock_leftover*image.width_subblock_leftover);
            for h in 0..image.height_subblock_leftover
            {
                //pos_subblock_xleftover_lookup.extend((0..image.width_subblock_leftover).map(|i|(h*image.width+if h%2==1{image.width_subblock_leftover-i-1}else{ i})*channels));
                for i in 0..image.width_subblock_leftover
                {
                    pos_subblock_xyleftover_lookup.push((h*image.width+if h%2==1{image.width_subblock_leftover-i-1}else{ i})*channels);
                }
            }
            list_of_subblocks_in_bottom_widthblock.push(&pos_subblock_xyleftover_lookup);
            
        }
        list_of_subblocks_in_heightblock.push(&list_of_subblocks_in_bottom_widthblock);

    }



                
            
    let mut curr_lengths: [usize;3]=[0;3];
    //while position < image_size
    /*let mut pos_offset=0;
    for i in 0..image.subblocks_in_width
    {
        position=pos_offset+pos_subblock_lookup[i];
    }
    pos_offset+=crate::image::SUBBLOCK_WIDTH_MAX;*/
    /*for px_i in 0..image_size/channels
    {*/
    #[cfg(debug_assertions)]
    let mut loopindex=0;
    
    for y in 0..list_of_subblocks_in_heightblock.len()
    {
        for x in 0..list_of_subblocks_in_heightblock[y].len()
        {
            for i in 0..list_of_subblocks_in_heightblock[y][x].len()
            { 
                //y, then x
                prev_position=position;
                position = channels*(y*image.width_block_size+x*image::SUBBLOCK_WIDTH_MAX)+list_of_subblocks_in_heightblock[y][x][i];





                if curr_lengths.iter().any(|&x| x == 0)
                {

                    //only in backref
                    //only in rgb+colorluma
                    /*match prefix_2bits {
                        PREFIX_BACK_REF=>{

                        },
                        default=>{

                        }
                    }*/
                    if prefix1==PREFIX_COLOR_LUMA
                    {
                        
                        /*let mut is_first=true;
                        let mut first_diff=0;
                        for i in 0..=2
                        {
                            if curr_lengths[i] == 0
                            {
                                if is_first
                                {
                                    first_diff=((prefix_2bits<<5)+bitreader.read_bitsu8(4)?) as i16-16;
                                    output_vec[position+i]=(first_diff+((output_vec[prev_position+i]) as i16)) as u8;
                                    is_first=false;
                                }
                                else
                                {
                                    output_vec[position+i]=((bitreader.read_bitsu8(4)? as i16)+first_diff+((output_vec[prev_position+i]) as i16)-8) as u8;
                                } 
                            }
                        }*/

                    }
                    else
                    {
                        //backref 4 bits!
                        //call after run when needed to get full 4 bits
                        let mut back_ref=0;
                        if prefix1==PREFIX_BACK_REF
                        {
                            back_ref = decoder.read_next_symbol()? as usize;

                        }
                        for i in 0..=2
                        {
                            if curr_lengths[i] == 0
                            {                            
                                
                                if prefix1==PREFIX_BACK_REF
                                {
                                    output_vec[position+i]=previous16_pixels_unique[previous16_pixels_unique_offset+back_ref][i];
                                }
                                else
                                {
                                    if prefix1==PREFIX_RGB
                                    {
    
                                        //RGB
                                        output_vec[position+i]=decoder.read_next_symbol()?;

    
                                    }
    
                                }
                            }
                            else {
                                curr_lengths[i] -= 1;
                                output_vec[position+i]=output_vec[prev_position+i];

                            }
                        }
                        if prefix1==PREFIX_RGB
                        {
                            

                            previous16_pixels_unique[previous16_pixels_unique_offset+32]=[output_vec[position],output_vec[position + 1],output_vec[position + 2]];
                            previous16_pixels_unique_offset+=1;
                            if previous16_pixels_unique_offset==32
                            {
                                for i in 0..=31
                                {
                                    previous16_pixels_unique[i]=previous16_pixels_unique[i+32];
                                }
    
                                previous16_pixels_unique_offset=0;
                            }

                            /*if previous16_pixels_unique[31+previous16_pixels_unique_offset][0]==201&&previous16_pixels_unique[31+previous16_pixels_unique_offset][1]==0&&previous16_pixels_unique[31+previous16_pixels_unique_offset][2]==250
                            {
                                dbg!(position);
                            }*/
                        }
                    }
                    




                    //get run length
                    //if color_check 
                    //{
                        //can't read 2 as only 1 may be needed
                        prefix1 = decoder.read_next_symbol()?;
                    //}
                    
                    /*for i in 0..=2
                    {
                        if curr_lengths[i]>0
                        {
                            curr_lengths[i] -= 1;
                            output_vec[position+i]=output_vec[prev_position+i];
                        }
                    }*/

                        //read full run for 1 run type up to 3 times

                    //TODO while loop without if faster???
                    if prefix1 == PREFIX_RUN
                    {
                        let mut run_prefix = decoder.read_next_symbol()?;
                        let mut temp_curr_runcount: u8;

                        for i in 0..=2
                        {
                            temp_curr_runcount=0;
                            while prefix1 == PREFIX_RUN && run_prefix == PREFIX_RUNS[i]
                            {
                                curr_lengths[i] +=(decoder.read_next_symbol()? as usize) << temp_curr_runcount;
                                prefix1 = decoder.read_next_symbol()?;
                                if prefix1 == PREFIX_RUN
                                {
                                    run_prefix = decoder.read_next_symbol()?;
                                }
                                temp_curr_runcount += 3;

                            }                   
                                    
                            if temp_curr_runcount > 0
                            {
                                curr_lengths[i] += 1;
                            }
                        }

                        
                    }
                    
                }
                else
                {
                    for i in 0..=2
                    {
                        curr_lengths[i] -= 1;
                        output_vec[position+i]=output_vec[prev_position+i];
                    }
                }

                #[cfg(debug_assertions)]
                {
                    //let dump_res=dump.next().unwrap().unwrap();


                    debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}, index: {}",dump[position],output_vec[position],position,loopindex);
                    debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}, index: {}",dump[position+1],output_vec[position+1],position+1,loopindex);
                    debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}, index: {}",dump[position+2],output_vec[position+2],position+2,loopindex);

                }
                #[cfg(debug_assertions)]
                {

                    loopindex+=1;
                }
            

            }
        }
    }
    //bitreader.read_bits(8)?;
    println!("{}", now.elapsed().as_millis());
    Ok(image)
}
