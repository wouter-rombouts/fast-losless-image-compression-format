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
pub(crate) const PREFIX_BACK_REF: u8 = 6;
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
    data.add_output_type(7);
    //2==SC_RUN_LENGTHS
    data.add_output_type(8);
    //3==SC_LUMA_BASE_DIFF
    data.add_output_type(64);
    //4==SC_LUMA_OTHER_DIFF
    data.add_output_type(16);
    //5==SC_LUMA_BACK_REF
    data.add_output_type(10);
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

    let rel_ref_lookup:[usize;10]=[channels,channels*image_header.width,channels*(1+image_header.width),2*channels,2*channels*image_header.width,channels*(2*image_header.width+1),channels*(image_header.width+2),channels*2*(image_header.width+1),3*channels,3*channels*image_header.width];
    
    
    let mut red_hrun_iter=(0..0).rev();
    let mut red_vrun_list_iter=vec![(0..0).rev();image_header.width];
    //let mut red_drun_list_iter=vec![(0..0).rev();image_header.width+1];
    let mut green_hrun_iter=(0..0).rev();
    let mut green_vrun_list_iter=vec![(0..0).rev();image_header.width];
    //let mut green_drun_list_iter=vec![(0..0).rev();image_header.width+1];
    let mut blue_hrun_iter=(0..0).rev();
    let mut blue_vrun_list_iter=vec![(0..0).rev();image_header.width];
    //let mut blue_drun_list_iter=vec![(0..0).rev();image_header.width+1];
    //let mut redrun=(0usize..4000).rev().map(|x| redrunstart+redrunstep * x*channels);

    //TODO create list like vertical run, but with mod width+1 instead of width for diagonal run
    //hrun breaks all other runs
    //vrun breaks drun
    //drun breaks vrun
    
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
        //let drun_pos=loop_index%(image_header.width+1);
        let testb=blue_vrun_list_iter[vrun_pos].next();
        let testr=red_vrun_list_iter[vrun_pos].next();
        let testg=green_vrun_list_iter[vrun_pos].next();
        let is_not_red_run_item = red_hrun_iter.next() == None && testr == None /*&& red_drun_list_iter[drun_pos].next() == None*/;
        let is_not_green_run_item = green_hrun_iter.next()==None &&  testg==None/* && green_drun_list_iter[drun_pos].next() == None*/;
        let is_not_blue_run_item = blue_hrun_iter.next()==None && testb==None/* && blue_drun_list_iter[drun_pos].next() == None*/;


        if is_not_red_run_item ||is_not_green_run_item||is_not_blue_run_item
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

                if position>0 &&(is_not_red_run_item && list_of_color_diffs[0]>=-8 && list_of_color_diffs[0]<8 || !is_not_red_run_item) &&
                   (is_not_green_run_item && list_of_color_diffs[1]>=-8 && list_of_color_diffs[1]<8 || !is_not_green_run_item) &&
                   (is_not_blue_run_item && list_of_color_diffs[2]>=-8 && list_of_color_diffs[2]<8 || !is_not_blue_run_item)
                {

                    if (is_not_red_run_item && list_of_color_diffs[0]==0 || !is_not_red_run_item) &&
                    (is_not_green_run_item && list_of_color_diffs[1]==0 || !is_not_green_run_item) &&
                    (is_not_blue_run_item && list_of_color_diffs[2]==0 || !is_not_blue_run_item)
                    {

                        data.add_symbolu8(PREFIX_BACK_REF, SC_PREFIXES);
                    }
                    else
                    {
                        data.add_symbolu8(PREFIX_SMALL_DIFF, SC_PREFIXES);
                        if is_not_red_run_item
                        {                    
                            data.add_symbolu8((8+list_of_color_diffs[0]) as u8, SC_SMALL_DIFF);
                        }
                        if is_not_green_run_item
                        {
                            data.add_symbolu8((8+list_of_color_diffs[1]) as u8, SC_SMALL_DIFF);
                        }
                        if is_not_blue_run_item
                        {
                            data.add_symbolu8((8+list_of_color_diffs[2]) as u8, SC_SMALL_DIFF);
                        }
                        amount_of_diffs+=1;
                    }
                        
                    

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
                    for i in 0..=9
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
                            (is_not_red_run_item && list_of_color_diffs[0]>=-8 && list_of_color_diffs[0]<8 || !is_not_red_run_item)&&
                            (is_not_blue_run_item && list_of_color_diffs[2]>=-8 && list_of_color_diffs[2]<8 || !is_not_blue_run_item)
                            {

                                
                                data.add_symbolu8(PREFIX_COLOR_LUMA, SC_PREFIXES);
                                data.add_symbolusize(i, SC_LUMA_BACK_REF);

                                data.add_symbolu8((list_of_color_diffs[1]+32) as u8, SC_LUMA_BASE_DIFF);
                                if is_not_red_run_item
                                {
                                    data.add_symbolu8((list_of_color_diffs[0]+8) as u8, SC_LUMA_OTHER_DIFF);
                                }
                                if is_not_blue_run_item
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
                        if is_not_red_run_item
                        {
                            data.add_symbolu8(input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0}), SC_RGB);
                        }        
                        if is_not_green_run_item
                        {
                            data.add_symbolu8(input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0}), SC_RGB);

                        }
                        if is_not_blue_run_item
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

            if is_not_red_run_item
            {
                let mut red_run_length = 0;
                let mut offset_step=1;
                //split to see if exists
                //TODO handle edge cases

                let offsety=input_bytes.get(position.saturating_sub(image_header.width*channels));
                let offsetx=input_bytes.get(position.saturating_sub(channels));
                //TODO diagonal run?2 new directions for run, how to pick right one?
                //after direction pick, determine which colors to run with.
                //rgbrun by default, exception to be color run only

                //dynamic run: color runs can be 0. main loop will have only red run as it is mandatory.
                //after first red run part, when 3 zero's in run length part are encountered, the run must stop
                //some numbers have 000 in their binary presentation, and must be excluded
                //can also repeat run for the entire run, 0 each time when already 0
                //this deletes colorrun prefixes, while allowing for partial color runs

                //let offsetxy=input_bytes.get(position.saturating_sub((image_header.width+1)*channels));

                let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(input_bytes[position])};
                let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(input_bytes[position])};
                //let xy_diff=if offsetxy == None {0} else {offsetxy.unwrap().abs_diff(input_bytes[position])};

                /*if y_diff<=x_diff
                {
                    offset_step=image_header.width;
                }*/
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
                    //TODO closure?
                    //Horizontal run
                    if offset_step==1
                    {
                        
                        //add red runlength
                        red_hrun_iter=(0..red_run_length).rev();
                    }
                    else
                    {
                        //if offset_step==image_header.width
                        //{
                            red_vrun_list_iter[vrun_pos]=(0..red_run_length).rev();
                        //}
                        //else
                        //{
                        //    red_drun_list_iter[drun_pos]=(0..red_run_length).rev();
                        //}
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

            if is_not_green_run_item
            {
                let mut green_run_length = 0;

                let mut offset_step=1;

                let offsety=input_bytes.get((position+1).saturating_sub(image_header.width*channels));
                let offsetx=input_bytes.get((position+1).saturating_sub(channels));
                //let offsetxy=input_bytes.get((position+1).saturating_sub((image_header.width+1)*channels));

                let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(input_bytes[position+1])};
                let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(input_bytes[position+1])};
                //let xy_diff=if offsetxy == None {0} else {offsetxy.unwrap().abs_diff(input_bytes[position+1])};
                /*if y_diff<=x_diff
                {
                    offset_step=image_header.width;
                }*/
                /*else
                {
                }
                if xy_diff.abs_diff(input_bytes[position+1])<=x_diff||
                   xy_diff.abs_diff(input_bytes[position+1])<=y_diff
                {
                    offset_step=image_header.width+1;
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
                        
                        //add green runlength
                        green_hrun_iter=(0..green_run_length).rev();
                    }
                    else
                    {
                        //if offset_step==image_header.width
                        //{
                            green_vrun_list_iter[vrun_pos]=(0..green_run_length).rev();
                        //}
                        //else
                        //{
                        //    green_drun_list_iter[drun_pos]=(0..green_run_length).rev();
                        //}
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
            if is_not_blue_run_item
            {
                let mut blue_run_length = 0;

                let mut offset_step=1;

                let offsety=input_bytes.get((position+2).saturating_sub(image_header.width*channels));
                let offsetx=input_bytes.get((position+2).saturating_sub(channels));
                //let offsetxy=input_bytes.get((position+2).saturating_sub((image_header.width+1)*channels));

                let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(input_bytes[position+2])};
                let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(input_bytes[position+2])};
                //let xy_diff=if offsetxy == None {0} else {offsetxy.unwrap().abs_diff(input_bytes[position+2])};
                /*if y_diff<=x_diff
                {
                    offset_step=image_header.width;
                }*/
                /*else
                {

                }
                //else?
                if xy_diff.abs_diff(input_bytes[position+2])<=x_diff||
                   xy_diff.abs_diff(input_bytes[position+2])<=y_diff
                {
                    offset_step=image_header.width+1;
                }*/

                let mut blue_run_loop_position=position+offset_step*channels;
                
                while blue_run_loop_position<image_size&&
                    input_bytes[blue_run_loop_position+2]==input_bytes[position+2]/*&&
                    input_bytes[blue_run_loop_position+1]!=input_bytes[prev_blue_run_loop_position+1]&&
                    input_bytes[blue_run_loop_position]!=input_bytes[prev_blue_run_loop_position]*/
                {
                    blue_run_length+=1;
                    blue_run_loop_position=position+(blue_run_length+1)*offset_step*channels;
                }

                if blue_run_length > 2
                {


                    //Horizontal run
                    if offset_step==1
                    {
                        
                        //add blue runlength
                        blue_hrun_iter=(0..blue_run_length).rev();
                    }
                    else
                    {
                        //if offset_step==image_header.width
                        //{
                            blue_vrun_list_iter[vrun_pos]=(0..blue_run_length).rev();
                        //}
                        //else
                        //{
                        //    blue_drun_list_iter[drun_pos]=(0..blue_run_length).rev();
                        //}
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
    let mut prefix_lookup = SymbolstreamLookup::new(7);
    //2==SC_RUN_LENGTHS
    let mut runlength_lookup = SymbolstreamLookup::new(8);
    //3==SC_LUMA_BASE_DIFF
    let mut luma_base_diff_lookup = SymbolstreamLookup::new(64);
    //4==SC_LUMA_OTHER_DIFF
    let mut luma_other_diff_lookup = SymbolstreamLookup::new(16);
    //5==SC_LUMA_BACK_REF
    let mut luma_backref_lookup = SymbolstreamLookup::new(10);
    //6==SC_SMALL_DIFF
    let mut small_diff_lookup = SymbolstreamLookup::new(16);
    
    decoder.read_header_into_tree(&mut rgb_lookup).unwrap();
    decoder.read_header_into_tree(&mut prefix_lookup).unwrap();
    decoder.read_header_into_tree(&mut runlength_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_backref_lookup).unwrap();
    decoder.read_header_into_tree(&mut small_diff_lookup).unwrap();

    let rel_ref_lookup:[usize;10]=[channels,channels*image.width,channels*(1+image.width),2*channels,2*channels*image.width,channels*(2*image.width+1),channels*(image.width+2),channels*2*(image.width+1),3*channels,3*channels*image.width];

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
    let mut redvrun_colors=vec![0;image.width];
    let mut red_vrun_list_iter=vec![(0..0).rev();image.width];
    let mut greenhrun_color=0;
    let mut greenhrun_iter=(0usize..0).rev();
    let mut greenvrun_colors=vec![0;image.width];
    let mut green_vrun_list_iter=vec![(0..0).rev();image.width];
    let mut bluehrun_color=0;
    let mut bluehrun_iter=(0usize..0).rev();
    let mut bluevrun_colors=vec![0;image.width];
    let mut blue_vrun_list_iter=vec![(0..0).rev();image.width];
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
                let vrun_pos=(position/channels)%image.width;
                //let drun_pos=loop_index%(image_header.width+1);

                let mut is_not_red_run_item=true;
                if redhrun_iter.next() != None
                {
                    output_vec[position]=redhrun_color;
                    is_not_red_run_item=false;
                }
                if red_vrun_list_iter[vrun_pos].next() != None
                {
                    output_vec[position]=redvrun_colors[vrun_pos];
                    is_not_red_run_item=false;
                }

                let mut is_not_green_run_item=true;
                if greenhrun_iter.next() != None
                {
                    output_vec[position+1]=greenhrun_color;
                    is_not_green_run_item=false;
                }
                if green_vrun_list_iter[vrun_pos].next() != None
                {
                    output_vec[position+1]=greenvrun_colors[vrun_pos];
                    is_not_green_run_item=false;
                }

                let mut is_not_blue_run_item=true;
                if bluehrun_iter.next() != None
                {
                    output_vec[position+2]=bluehrun_color;
                    is_not_blue_run_item=false;
                }
                if blue_vrun_list_iter[vrun_pos].next() != None
                {
                    output_vec[position+2]=bluevrun_colors[vrun_pos];
                    is_not_blue_run_item=false;
                }



                if is_not_red_run_item || is_not_green_run_item || is_not_blue_run_item
                {
                    //let headertime = Instant::now();
                    match prefix1
                    {
                        PREFIX_SMALL_DIFF=>
                        {
                            if is_not_red_run_item
                            {
                                output_vec[position]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos] as i16)as u8;
                            }
                            if is_not_green_run_item
                            {
                                output_vec[position+1]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos+1] as i16)as u8;
                            }
                            if is_not_blue_run_item
                            {
                                output_vec[position+2]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-8 +output_vec[prev_pos+2] as i16)as u8;
                            }
                        }
                        PREFIX_COLOR_LUMA=>
                        {
                            let backref = rel_ref_lookup[decoder.read_next_symbol(&luma_backref_lookup)? as usize];
                            prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff_lookup)? as i16-32;

                            output_vec[position+1]=(prev_luma_base_diff + output_vec[position-backref+1] as i16) as u8;


                            if is_not_red_run_item
                            {
                                prev_luma_other_diff1=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position]=(prev_luma_other_diff1 + prev_luma_base_diff+(output_vec[position-backref] as i16)) as u8;
                            }
                            
                            if is_not_blue_run_item
                            {
                                prev_luma_other_diff2=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position+2]=(prev_luma_other_diff2 + prev_luma_base_diff+(output_vec[position-backref+2] as i16)) as u8;
                            }
                        }
                        PREFIX_BACK_REF=>
                        {
                            if is_not_red_run_item
                            {
                                output_vec[position]=output_vec[prev_pos];
                            }
                            if is_not_green_run_item
                            {
                                output_vec[position+1]=output_vec[prev_pos+1];
                            }
                            if is_not_blue_run_item
                            {
                                output_vec[position+2]=output_vec[prev_pos+2];
                            }
                        }
                        _=>
                        {
                            if is_not_red_run_item
                            {
                                output_vec[position]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos]);
                            }
                            if is_not_green_run_item
                            {
                                output_vec[position+1]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+1]);
                            }
                            if is_not_blue_run_item
                            {
                                output_vec[position+2]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+2]);
                            }
                        }

                    }
                    

                    //temp_time+=headertime.elapsed().as_nanos();
                    prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    let mut temp_curr_runcount: u8=0;
                    let mut red_run_length=0;





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
                        
                        let offsety=output_vec.get(position.saturating_sub(image.width*channels));
                        let offsetx=output_vec.get(position.saturating_sub(channels));
                        let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(output_vec[position])};
                        let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(output_vec[position])};
                        if y_diff<=x_diff
                        {
                            red_vrun_list_iter[vrun_pos]=(0..red_run_length).rev();
                            redvrun_colors[vrun_pos]=output_vec[position];
                        }
                        else
                        {
                            redhrun_iter=(0..red_run_length).rev();
                            redhrun_color=output_vec[position];
                        }
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
                                    
                        let offsety=output_vec.get((position+1).saturating_sub(image.width*channels));
                        let offsetx=output_vec.get((position+1).saturating_sub(channels));
                        let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(output_vec[position+1])};
                        let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(output_vec[position+1])};
                        if y_diff<=x_diff
                        {
                            green_vrun_list_iter[vrun_pos]=(0..green_run_length).rev();
                            greenvrun_colors[vrun_pos]=output_vec[position+1];
                        }
                        else
                        {
                            greenhrun_iter=(0..green_run_length).rev();
                            greenhrun_color=output_vec[position+1];
                        }
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
                                    

                        
                        let offsety=output_vec.get((position+2).saturating_sub(image.width*channels));
                        let offsetx=output_vec.get((position+2).saturating_sub(channels));
                        
                        let y_diff=if offsety == None {0} else {offsety.unwrap().abs_diff(output_vec[position+2])};
                        let x_diff=if offsetx == None {0} else {offsetx.unwrap().abs_diff(output_vec[position+2])};
                        if y_diff<=x_diff
                        {
                            blue_vrun_list_iter[vrun_pos]=(0..blue_run_length).rev();
                            bluevrun_colors[vrun_pos]=output_vec[position+2];
                        }
                        else
                        {
                            bluehrun_iter=(0..blue_run_length).rev();
                            bluehrun_color=output_vec[position+2];
                        }
                    }


                    //dbg!(prefix1);
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
