const NICE: &[u8] = "nice".as_bytes();
use crate::bitwriter;
use crate::image::{Image, self};
use crate::run::RunCountdown;
use std::cmp::Reverse;
use std::collections::{HashMap, BinaryHeap};
use std::{time::Instant, *};
use crate::hfe::{EncodedOutput, self, SymbolstreamLookup};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
//pub(crate) const PREFIX_RUN: u8 = 2;
pub(crate) const PREFIX_RED_RUN: u8 = 0;
pub(crate) const PREFIX_GREEN_RUN: u8 = 1;
pub(crate) const PREFIX_BLUE_RUN: u8 = 2;
pub(crate) const PREFIX_RGB: u8 = 3;
pub(crate) const PREFIX_COLOR_LUMA: u8 = 4;
pub(crate) const PREFIX_SMALL_DIFF: u8 = 5;
//pub(crate) const PREFIX_REF: u8 = 6;
//stream codes
pub(crate) const SC_RGB: u8 = 0;
pub(crate) const SC_PREFIXES: u8 = 1;
pub(crate) const SC_RUN_LENGTHS: u8 = 2;
pub(crate) const SC_LUMA_BASE_DIFF: u8 = 3;
pub(crate) const SC_LUMA_OTHER_DIFF: u8 = 4;
pub(crate) const SC_LUMA_BACK_REF: u8 = 5;
pub(crate) const SC_SMALL_DIFF: u8 = 6;
//pub(crate) const SC_REF: u8 = 7;
//pub(crate) const SC_PREV_INPUT: u8 = 9;

/*pub(crate) const PREV_INPUT_BACK_REF: u8 = 0;
pub(crate) const PREV_INPUT_LUMA: u8 = 1;
pub(crate) const PREV_INPUT_RUN: u8 = 2;*/


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
    //0==PREFIX_RGB
    data.add_output_type(256);
    //1==SC_PREFIXES
    data.add_output_type(6);
    //2==SC_RUN_LENGTHS
    data.add_output_type(8);
    //3==SC_LUMA_BASE_DIFF
    data.add_output_type(64);
    //4==SC_LUMA_OTHER_DIFF
    data.add_output_type(16);
    //5==SC_LUMA_BACK_REF
    data.add_output_type(8);
    //6==SC_SMALL_DIFF
    data.add_output_type(16);
    //7==SC_REF
    //data.add_output_type(8);
    let mut amount_of_refs=0;
    let mut amount_of_diffs=0;
    let mut prev_run_count=0;
    //16 size, with 16 spares
    //hold slice or actual values
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [(u8,u8,u8);64] = [(0,0,0);64];

    //main loop

    let mut rgb_cntr = 0;
    let mut run_cntr=0;
    let mut luma_occurences=0;
    let mut red_pixel_run_amount=0;
    let mut run_occurrences=[0;8];

    let rel_ref_lookup:[usize;8]=[channels,channels*image_header.width,channels*(1+image_header.width),2*channels,2*channels*image_header.width,channels*(2*image_header.width+1),channels*(image_header.width+2),channels*2*(image_header.width+1)];

    let mut vrun_lookup_red = vec![0;image_header.width];
    let mut vrun_lookup_green = vec![0;image_header.width];
    let mut vrun_lookup_blue = vec![0;image_header.width];
    //calc the top pixels
    /*let top_pixels:Vec<[u8; 3]>;
    let mut map : HashMap<[u8;3],usize>= HashMap::new();
    let mut dummy=[0;3];
    for x in input_bytes.chunks_exact(3)
    {
        dummy.copy_from_slice(x);
        *map.entry(dummy).or_default() += 1;
    }

    let mut heap = BinaryHeap::with_capacity(16 + 1);
    for (x, count) in map.into_iter()
    {
        heap.push(Reverse((count, x)));
        if heap.len() > 16
        {
            heap.pop();
        }
    }
    top_pixels=heap.into_sorted_vec().into_iter().map(|r| r.0.1).collect();*/
    //println!("top_pixels: {:?}",top_pixels);
    
    //TODO add symbols as normal, but also add the symbols for other streams
    // calculate aob for each stream at the end, based on preferred stream(same value as before)
    //reassign pixel to different preferred stream if smaller aob is found
    //repeat until satisfied with result
    //add result to output stream


    for loop_index in 0..image_size/channels
    {
        //TODO cache for calc_pos_from function when values are generated from run lookups
        //TODO move to end, only when run for color was not recetnly added
        //let is_not_in_run=run_count_red ==1 ||run_count_green==1||run_count_blue==1;
        let prev_position = position;
        position=loop_index*channels;
        let vrun_pos=loop_index%image_header.width;
        let vrun_red_item= vrun_lookup_red[vrun_pos];
        let vrun_green_item= vrun_lookup_green[vrun_pos];
        let vrun_blue_item= vrun_lookup_blue[vrun_pos];
        if vrun_red_item ==0 ||vrun_green_item==0||vrun_blue_item==0
        {          
            
            //check color diff
            //TODO only check for non run colors                
            
            
            //TODO check outputted encoded length and choose the one that has least aob total
            //TODO how to take the best possible combinations when this changes the outcome?
            /*if run_count_green >1&&run_count_red>1
            {data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);
                data.add_symbolu8(input_bytes[position+2].wrapping_sub(if position>0{input_bytes[prev_position+2]}else{0}), SC_RGB);
            }
            else
            {
            if run_count_red >1&&run_count_blue>1
            {data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);
                data.add_symbolu8(input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0}), SC_RGB);
            }
            else
            {
            if run_count_green >1&&run_count_blue>1
            {data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);
                data.add_symbolu8(input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0}), SC_RGB);
            }
            else
            {*/
            /*let prediction=0;
            for i in 0..16
            {
            }*/
            {
                let mut list_of_color_diffs=[0;3];
                //TODO try back,up,down,forward to calc diff
                //green_diff
                list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[prev_position+1] as i16;
                //red_diff
                list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[prev_position] as i16;
                //blue_diff
                list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[prev_position+2] as i16;

                if position>0&&(vrun_red_item ==0&&list_of_color_diffs[0]>=-8&&list_of_color_diffs[0]<8||vrun_red_item >0)&&
                   (vrun_green_item ==0&&list_of_color_diffs[1]>=-8&&list_of_color_diffs[1]<8||vrun_green_item >0)&&
                   (vrun_blue_item ==0&&list_of_color_diffs[2]>=-8&&list_of_color_diffs[2]<8||vrun_blue_item >0)
                {

                    data.add_symbolu8(PREFIX_SMALL_DIFF, SC_PREFIXES);
                    if vrun_red_item ==0
                    {                    
                        data.add_symbolu8((8+list_of_color_diffs[0]) as u8, SC_SMALL_DIFF);
                    }
                    if vrun_green_item ==0
                    {
                        data.add_symbolu8((8+list_of_color_diffs[1]) as u8, SC_SMALL_DIFF);
                    }
                    if vrun_blue_item ==0
                    {
                        data.add_symbolu8((8+list_of_color_diffs[2]) as u8, SC_SMALL_DIFF);
                    }
                    amount_of_diffs+=1;
                }
                else
                {
                    //let mut top_found=false;
                    /*for i in 0..top_pixels.len()
                    {
                        if input_bytes[position]==top_pixels[i][0]&&input_bytes[position+1]==top_pixels[i][1]&&input_bytes[position+2]==top_pixels[i][2]
                        {
                            /*sdbg!(i);
                            dbg!(top_pixels[i][0]);
                            dbg!(top_pixels[i][1]);
                            dbg!(top_pixels[i][2]);*/
                            data.add_symbolu8(PREFIX_MOST_COMMON, SC_PREFIXES);

                            data.add_symbolusize(i, SC_MOST_COMMON);
                            top_found=true;

                            break;
                        }
                    }*/

                    /*if !top_found
                    {*/
                    
                    //TODO move to start to generate offsets
                    /*let ymoveup_begin=2;
                    let mut ymoveup=2;
                    let ypos=position/(image_header.width*channels);
                    if ypos/ymoveup_begin<1
                    {
                        ymoveup-=ymoveup_begin-ypos;
                    }
                    /*if image_header.height - ypos<= ymoveup_begin
                    {
                        ymoveup+=ymoveup_begin+1 + ypos-image_header.height;
                    }*/
                    let xmoveback_begin=2;
                    let mut xmoveback=2;
                    let xpos=(position%(image_header.width*channels))/channels;
                    if xpos < xmoveback_begin
                    {
                        xmoveback-=xmoveback_begin-xpos;
                    }
                    /*if xpos>=image_header.width-xmoveback_begin
                    {
                        xmoveback+=xmoveback_begin+1+xpos-image_header.width
                    }*/
                    let begin_offset:usize=position-ymoveup*image_header.width*channels-xmoveback*channels;*/
                    let mut list_of_color_diffs=[0;3];
                    let mut is_luma=false;
                    for i in 0..=7
                    {
                        
                        if let Some(ref_pos)=position.checked_sub(rel_ref_lookup[i])
                        {

                            /*if offset != position&&input_bytes[position]==input_bytes[(offset )as usize]&&
                            input_bytes[position+1]==input_bytes[(offset+1)as usize]&&
                            input_bytes[position+2]==input_bytes[(offset+2)as usize]
                            {
                                data.add_symbolu8(PREFIX_REF, SC_PREFIXES);
        
                                data.add_symbolusize((y*3+x)-if offset>position{1}else{0}/*-(y*7+x)/25*/, SC_REF);
                                ref_found=true;
                                amount_of_refs+=1;
                                break 'outer;
                            }*/
                        
                            //green_diff
                            list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[ref_pos+1] as i16;
                        
                            //red_diff
                            list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[ref_pos] as i16;
                            //blue_diff
                            list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[ref_pos+2] as i16;
                            list_of_color_diffs[0]-=list_of_color_diffs[1];
                            list_of_color_diffs[2]-=list_of_color_diffs[1];
                            //TODO create luminosity field run, for rgb?
                            //when rgb or diff, calc lumo level, if not in +-8, go to other color layer(,write to output)

                            //TODO: wrap around 256/0 logic
                            //TODO check 4 backreference
                            //new algo: most used token in stream repeated
                            /*if (run_count_green==1&&prev_luma_base_diff==list_of_color_diffs[1]||run_count_green > 1)&&
                            (run_count_red==1&&prev_luma_other_diff1==list_of_color_diffs[0]||run_count_red > 1)&&
                            (run_count_blue==1&&prev_luma_other_diff2==list_of_color_diffs[2]||run_count_blue > 1)
                            {
                                data.add_symbolu8(PREFIX_PREV_INPUT, SC_PREFIXES);
                            }
                            else*/
                            //TODO special case when base high then other only low diff. must be branchless.
                            //TODO re Add flexible base
                            //TODO repeat until no RGB needed?use of repeat token needed
                            //or take most occurred result instead of first result when adding from list of backrefs. 
                            //use run type(s) code stream
                            if position>0&&
                            list_of_color_diffs[1]>=-32&&list_of_color_diffs[1]<32&&
                            (vrun_red_item ==0&&list_of_color_diffs[0]>=-8&&list_of_color_diffs[0]<8||vrun_red_item > 0)&&
                            (vrun_blue_item ==0&&list_of_color_diffs[2]>=-8&&list_of_color_diffs[2]<8||vrun_blue_item > 0)
                            {
                                data.add_symbolu8(PREFIX_COLOR_LUMA, SC_PREFIXES);
                                data.add_symbolusize(i, SC_LUMA_BACK_REF);

                                data.add_symbolu8((list_of_color_diffs[1]+32) as u8, SC_LUMA_BASE_DIFF);
                                if vrun_red_item == 0
                                {
                                    data.add_symbolu8((list_of_color_diffs[0]+8) as u8, SC_LUMA_OTHER_DIFF);
                                }
                                if vrun_blue_item == 0
                                {
                                    data.add_symbolu8((list_of_color_diffs[2]+8) as u8, SC_LUMA_OTHER_DIFF);
                                }
                                luma_occurences+=1;
                                is_luma=true;
                                break;
                            }
                            //TODO after loop is done
                            /*else
                            {
                            }*/
                        
                        }
                        else
                        {
                            continue;
                        }
                    }
                    //TODO update  previous16_pixels_unique when match is found
                    //write rgb
                    if is_luma==false
                    {
                        data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);

                        rgb_cntr+=1;
                        if vrun_red_item ==0
                        {
                            data.add_symbolu8(input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0}), SC_RGB);
                        }        
                        if vrun_green_item ==0
                        {
                            data.add_symbolu8(input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0}), SC_RGB);

                        }
                        if vrun_blue_item ==0
                        {
                            data.add_symbolu8(input_bytes[position+2].wrapping_sub(if position>0{input_bytes[prev_position+2]}else{0}), SC_RGB);

                        }
                        previous16_pixels_unique[previous16_pixels_unique_offset]=(input_bytes[position],input_bytes[position + 1],input_bytes[position + 2]);
                        previous16_pixels_unique_offset=(previous16_pixels_unique_offset+1)%8;
                    }/*
                    }*/
                }
            }
                    //TODO run: prev pixel cannot have same color
            //if close pixel(s) are more similar than other run type, try that.
            //Max diff, so when can take hrun as default?
            prev_run_count=0;
            //check for color run

            if vrun_red_item==0
            {
                let mut red_run_length = 0;
                let mut offset_step=1;
                //split to see if exists
                //TODO handle edge cases

                let offset1=input_bytes.get(position.saturating_sub(image_header.width*channels));
                let offset2=input_bytes.get(position.saturating_sub(channels));

                /*if offset1 != None&& offset2 != None &&offset1.unwrap().abs_diff(input_bytes[position])<
                offset2.unwrap().abs_diff(input_bytes[position])
                {
                    offset_step=image_header.width;
                }*/
                
                //let mut prev_red_run_loop_position=position;
                let mut red_run_loop_position=position+offset_step*channels;
                
                while red_run_loop_position<image_size&&
                    input_bytes[red_run_loop_position]==input_bytes[position]/*&&
                    input_bytes[red_run_loop_position+1]!=input_bytes[prev_red_run_loop_position+1]&&
                    input_bytes[red_run_loop_position+2]!=input_bytes[prev_red_run_loop_position+2]*/
                {
                    red_run_length+=1;
                    //prev_red_run_loop_position=red_run_loop_position;
                    red_run_loop_position=position+(red_run_length+1)*offset_step*channels;
                }

                if red_run_length > 2
                {
                    //Horizontal run
                    if offset_step==1
                    {
                        
                        //add red runlength
                        for i in 1..=red_run_length
                        {
                            vrun_lookup_red[(vrun_pos+i)%image_header.width]+=1;
                        }
                    }
                    else
                    {
                        vrun_lookup_red[vrun_pos]+=red_run_length;
                    }

                    //run_count_red+=red_run_length;
                    red_pixel_run_amount+=red_run_length;
                    red_run_length = red_run_length - 3;
                    run_cntr+=1;
                    loop
                    {
                        
                        data.add_symbolu8(PREFIX_RED_RUN, SC_PREFIXES);
                        data.add_symbolu8((red_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(red_run_length & 0b0000_0111)]+=1;
                        if red_run_length <8
                        {
                            break;
                        }
                        red_run_length = red_run_length >> 3;
                        
                    }
                }
            }

            if vrun_green_item==0
            {
                let mut green_run_length = 0;

                let mut offset_step=1;

                let offset1=input_bytes.get((position+1).saturating_sub(image_header.width*channels));
                let offset2=input_bytes.get((position+1).saturating_sub(channels));
                /*if offset1 != None&& offset2 != None &&offset1.unwrap().abs_diff(input_bytes[position+1])<
                offset2.unwrap().abs_diff(input_bytes[position+1])
                {
                    offset_step=image_header.width;
                }*/

                //let mut prev_green_run_loop_position=position;
                let mut green_run_loop_position=position+offset_step*channels;
                
                while green_run_loop_position<image_size&&
                    input_bytes[green_run_loop_position+1]==input_bytes[position+1]/*&&
                    input_bytes[green_run_loop_position]!=input_bytes[prev_green_run_loop_position]&&
                    input_bytes[green_run_loop_position+2]!=input_bytes[prev_green_run_loop_position+2]*/
                {
                    green_run_length+=1;
                    //prev_green_run_loop_position=green_run_loop_position;
                    green_run_loop_position=position+(green_run_length+1)*offset_step*channels;
                }

                if green_run_length > 2
                {
                    //Horizontal run
                    if offset_step==1
                    {
                        
                        //add red runlength
                        for i in 1..=green_run_length
                        {
                            vrun_lookup_green[(vrun_pos+i)%image_header.width]+=1;
                        }
                    }
                    else
                    {
                        vrun_lookup_green[vrun_pos]+=green_run_length;
                    }
                    //add green runlength
                    //loop
                    prev_run_count=green_run_length;
                    green_run_length = green_run_length - 3;
                    run_cntr+=1;
                    loop
                    {
                        data.add_symbolu8(PREFIX_GREEN_RUN, SC_PREFIXES);
                        data.add_symbolu8((green_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(green_run_length & 0b0000_0111)]+=1;
                        if green_run_length <8
                        {
                            break;
                        }
                        green_run_length = green_run_length >> 3;
                    }
                }
            }
            //TODO use iterator for hrun/vrun
            if vrun_blue_item==0
            {
                let mut blue_run_length = 0;

                let mut offset_step=1;

                let offset1=input_bytes.get((position+2).saturating_sub(image_header.width*channels));
                let offset2=input_bytes.get((position+2).saturating_sub(channels));
                /*if offset1 != None&& offset2 != None &&offset1.unwrap().abs_diff(input_bytes[position+2])<
                offset2.unwrap().abs_diff(input_bytes[position+2])
                {
                    offset_step=image_header.width;
                }*/

                let mut blue_run_loop_position=position+offset_step*channels;
                
                while blue_run_loop_position<image_size&&
                    input_bytes[blue_run_loop_position+2]==input_bytes[position+2]/*&&
                    input_bytes[blue_run_loop_position+1]!=input_bytes[prev_blue_run_loop_position+1]&&
                    input_bytes[blue_run_loop_position]!=input_bytes[prev_blue_run_loop_position]*/
                {
                    blue_run_length+=1;
                    //prev_blue_run_loop_position=blue_run_loop_position;
                    blue_run_loop_position=position+(blue_run_length+1)*offset_step*channels;
                }

                if blue_run_length > 2
                {


                    //Horizontal run
                    if offset_step==1
                    {
                        
                        //add red runlength
                        for i in 1..=blue_run_length
                        {
                            vrun_lookup_blue[(vrun_pos+i)%image_header.width]+=1;
                        }
                    }
                    else
                    {
                        vrun_lookup_blue[vrun_pos]+=blue_run_length;
                    }
                    //add blue runlength
                    //loop
                    blue_run_length = blue_run_length - 3;
                    run_cntr+=1;
                    loop
                    {
                        data.add_symbolu8(PREFIX_BLUE_RUN, SC_PREFIXES);
                        data.add_symbolu8((blue_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(blue_run_length & 0b0000_0111)]+=1;
                        if blue_run_length <8
                        {
                            break;
                        }
                        blue_run_length = blue_run_length >> 3;
                    }
                }
            }
        }

        


            if vrun_red_item >0
            {
                vrun_lookup_red[vrun_pos]-=1;
            }
            if vrun_green_item >0
            {
                vrun_lookup_green[vrun_pos]-=1;
            }
            if vrun_blue_item >0
            {
                vrun_lookup_blue[vrun_pos]-=1;
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
    dbg!(rgb_cntr);
    dbg!(run_cntr);
    dbg!(luma_occurences);
    dbg!(red_pixel_run_amount);
    dbg!(run_occurrences);
    dbg!(amount_of_refs);
    dbg!(amount_of_diffs);
    
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

    //0==PREFIX_RGB
    let mut rgb_lookup = SymbolstreamLookup::new(256);
    //1==SC_PREFIXES
    let mut prefix_lookup = SymbolstreamLookup::new(6);
    //2==SC_RUN_LENGTHS
    let mut runlength_lookup = SymbolstreamLookup::new(8);
    //3==SC_LUMA_BASE_DIFF
    let mut luma_base_diff_lookup = SymbolstreamLookup::new(64);
    //4==SC_LUMA_OTHER_DIFF
    let mut luma_other_diff_lookup = SymbolstreamLookup::new(16);
    //5==SC_LUMA_BACK_REF
    let mut luma_backref_lookup = SymbolstreamLookup::new(8);
    //6==SC_SMALL_DIFF
    let mut small_diff_lookup = SymbolstreamLookup::new(16);
    
    decoder.read_header_into_tree(&mut rgb_lookup).unwrap();
    decoder.read_header_into_tree(&mut prefix_lookup).unwrap();
    decoder.read_header_into_tree(&mut runlength_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_backref_lookup).unwrap();
    decoder.read_header_into_tree(&mut small_diff_lookup).unwrap();

    let rel_ref_lookup:[usize;8]=[channels,channels*image.width,channels*(1+image.width),2*channels,2*channels*image.width,channels*(2*image.width+1),channels*(image.width+2),channels*2*(image.width+1)];

    //let mut prefix_1bits=bitreader.read_bitsu8(1)?;
    //let mut prefix_2bits: u8=bitreader.read_bitsu8(1)?;

    let mut prefix1=decoder.read_next_symbol(&prefix_lookup)?;
    //let width = width as usize;
    let mut run_values=[0u8;3];

    let mut prev_luma_base_diff=0;
    let mut prev_luma_other_diff1=0;
    let mut prev_luma_other_diff2=0;

    
    //let mut temp_time=0;
    //curr_lengths[0] is red
    //curr_lengths[1] is green
    //curr_lengths[2] is blue
    //TODO snake + distinct cache + HFE

    #[cfg(debug_assertions)]
    let mut dump= Vec::<u8>::new();
    #[cfg(debug_assertions)]
    io::Read::read_to_end(&mut fs::File::open("dump.bin").unwrap(), &mut dump).ok();
    
    
    let mut prev_pos=0;

    #[cfg(debug_assertions)]
    let mut loopindex=0;
    let mut redhrun_color=0;
    let mut redhrun_iter=(0usize..0).rev();
    let mut greenhrun_color=0;
    let mut greenhrun_iter=(0usize..0).rev();
    let mut bluehrun_color=0;
    let mut bluehrun_iter=(0usize..0).rev();
    while position<image_size 
    {
    /*for y in 0..list_of_subblocks_in_heightblock.len()
    {
        for x in 0..list_of_subblocks_in_heightblock[y].len()
        {
            for i in 0..list_of_subblocks_in_heightblock[y][x].len()
            {*/
                //y, then x
                //position = channels*(y*image.width_block_size+x*image::SUBBLOCK_WIDTH_MAX)+list_of_subblocks_in_heightblock[y][x][i];
                let red_run_item_result = redhrun_iter.next();
                let green_run_item_result = greenhrun_iter.next();    
                let blue_run_item_result = bluehrun_iter.next();    
                /*if position<10||position>156044*3
                {
                    dbg!(prefix1);}*/
                if red_run_item_result == None || green_run_item_result == None ||blue_run_item_result == None
                {
                    //let headertime = Instant::now();
                    match prefix1
                    {
                        PREFIX_SMALL_DIFF=>
                        {
                            if red_run_item_result == None
                            {
                                output_vec[position]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos] as i16)as u8;
                            }
                            else
                            {
                                output_vec[position]=redhrun_color;
                            }
                            if green_run_item_result == None
                            {
                                output_vec[position+1]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos+1] as i16)as u8;
                            }
                            else
                            {
                                output_vec[position+1]=greenhrun_color;
                            }
                            if blue_run_item_result == None
                            {
                                output_vec[position+2]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos+2] as i16)as u8;
                            }
                            else
                            {
                                output_vec[position+2]=bluehrun_color;
                            }
                        }
                        PREFIX_COLOR_LUMA=>
                        {
                            let backref = rel_ref_lookup[decoder.read_next_symbol(&luma_backref_lookup)? as usize];
                            prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff_lookup)? as i16-32;

                            output_vec[position+1]=(prev_luma_base_diff + output_vec[position-backref+1] as i16) as u8;

                            if green_run_item_result != None
                            {
                                output_vec[position+1]=greenhrun_color;
                            }

                            if red_run_item_result == None
                            {
                                prev_luma_other_diff1=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position]=(prev_luma_other_diff1 + prev_luma_base_diff+(output_vec[position-backref] as i16)) as u8;
                            }
                            else
                            {
                                output_vec[position]=redhrun_color;
                            }
                            
                            if blue_run_item_result == None
                            {
                                prev_luma_other_diff2=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position+2]=(prev_luma_other_diff2 + prev_luma_base_diff+(output_vec[position-backref+2] as i16)) as u8;
                            }
                            else
                            {
                                output_vec[position+2]=bluehrun_color;
                            }
                        }
                        _=>
                        {
                            if red_run_item_result == None
                            {
                                output_vec[position]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos]);
                            }
                            else
                            {
                                output_vec[position]=redhrun_color;
                            }
                            if green_run_item_result == None
                            {
                                output_vec[position+1]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+1]);
                            }
                            else
                            {
                                output_vec[position+1]=greenhrun_color;
                            }
                            if blue_run_item_result == None
                            {
                                output_vec[position+2]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+2]);
                            }
                            else
                            {
                                output_vec[position+2]=bluehrun_color;
                            }
                        }

                    }
                    

                    //temp_time+=headertime.elapsed().as_nanos();
                    prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    let mut temp_curr_runcount: u8=0;
                    let mut red_run_length=0;
                    //dbg!(position);
                    //dbg!(prefix1);
                    while prefix1 == PREFIX_RED_RUN
                    {
                        //run lengths
                        red_run_length +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }
                    
                    if temp_curr_runcount>0
                    {

                        red_run_length += 3;
                        //dbg!(red_run_length);
                        redhrun_color=output_vec[position];
                        redhrun_iter=(0..red_run_length).rev();
                        temp_curr_runcount=0;
                    }

                    //dbg!(position);
                    //dbg!(prefix1);
                    let mut green_run_length=0;
                    while prefix1 == PREFIX_GREEN_RUN
                    {  
                        //run lengths
                        green_run_length +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }   
                
                    if temp_curr_runcount>0
                    {

                        green_run_length += 3;
                                    

                        greenhrun_color=output_vec[position+1];
                        greenhrun_iter=(0..green_run_length).rev();
                        temp_curr_runcount=0;
                    }

                    //dbg!(position);
                    //dbg!(prefix1);
                    let mut blue_run_length=0;
                    while prefix1 == PREFIX_BLUE_RUN
                    {  
                        //run lengths
                        blue_run_length +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }
                    
                    //dbg!(position);
                    //dbg!(prefix1);
                    if temp_curr_runcount>0
                    {

                        blue_run_length += 3;
                        //dbg!(blue_run_length);
                                    

                        bluehrun_color=output_vec[position+2];
                        bluehrun_iter=(0..blue_run_length).rev();
                    }


                    //dbg!(prefix1);
                }
                else
                {
                    output_vec[position]=redhrun_color;
                    output_vec[position+1]=greenhrun_color;
                    output_vec[position+2]=bluehrun_color;
                }

                #[cfg(debug_assertions)]
                {
                    debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}, index: {}",dump[position],output_vec[position],position,loopindex);
                    debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}, index: {}",dump[position+1],output_vec[position+1],position+1,loopindex);
                    debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}, index: {}",dump[position+2],output_vec[position+2],position+2,loopindex);

                }
                #[cfg(debug_assertions)]
                {

                    loopindex+=1;
                }
            
                prev_pos=position;
                position+=3;

            /* }
        }*/
    }
    //println!("temp_time:{}: ",temp_time);
    Ok(image)
}
