const NICE: &[u8] = "nice".as_bytes();
use itertools::Itertools;

use crate::bitwriter;
use crate::block::BlockDef;
use crate::image::{Image, self};
use crate::pixel::Pixel;
use crate::state::ColorState;
use std::cmp::Reverse;
use std::collections::{HashMap, BinaryHeap};
use std::{time::Instant, *};
use crate::hfe::{EncodedOutput, self, SymbolstreamLookup};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
//pub(crate) const PREFIX_RUN: u8 = 2;

pub(crate) const PREFIX_RGB: u8 = 1;
pub(crate) const PREFIX_COLOR_LUMA: u8 = 2;
pub(crate) const PREFIX_SMALL_DIFF: u8 = 3;
//pub(crate) const PREFIX_RUN_HORSE: u8 = 1;
//pub(crate) const PREFIX_RUN_HORSE2: u8 = 2;
pub(crate) const PREFIX_RUN: u8 = 0;
pub(crate) const PREFIX_ADJ_BLOCK: u8 = 4;
//pub(crate) const PREFIX_PREDICT: u8 = 5;
//pub(crate) const PREFIX_REF: u8 = 6;
//stream codes
pub(crate) const SC_RGB: u8 = 0;
pub(crate) const SC_PREFIXES: u8 = 1;
pub(crate) const SC_RUN_LENGTHS: u8 = 2;
pub(crate) const SC_LUMA_BASE_DIFF: u8 = 3;
pub(crate) const SC_LUMA_OTHER_DIFF: u8 = 4;
pub(crate) const SC_LUMA_BACK_REF: u8 = 5;
pub(crate) const SC_SMALL_DIFF: u8 = 6;
//pub(crate) const SC_BLOCK_DIFF: u8 = 7;
pub(crate) const SC_ADJ_BLOCK_PRESET: u8 = 7;
pub(crate) const SC_BLOCK_DIFF_SIZE: u8 = 8;


pub(crate) const BLOCK_SIZE: usize = 4;
pub(crate) const BLOCK_RANGE: u8 = 8;
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
    data.add_output_type(4);
    //2==SC_RUN_LENGTHS
    data.add_output_type(8);
    //3==SC_LUMA_BASE_DIFF
    data.add_output_type(64);
    //4==SC_LUMA_OTHER_DIFF
    data.add_output_type(16);
    //5==SC_LUMA_BACK_REF
    data.add_output_type(13);
    //6==SC_SMALL_DIFF
    data.add_output_type(8);
    //7==SC_ADJ_BLOCK
    //data.add_output_type(79);
    let amount_of_refs=0;
    let mut amount_of_diffs=0;
    let mut amount_of_block_diffs=0;
    //keeps track of value that need to be written for the key(references the 3 pixels)
    let mut block_adj_lookup : HashMap<([u8;3],[u8;6]),u8>=HashMap::with_capacity(79);
    //all pixels changed by {x}
    block_adj_lookup.insert( ([0,0,0],[0,0,0,0,0,0]), 0);
    block_adj_lookup.insert( ([1,1,1],[1,1,1,1,1,1]), 1);
    block_adj_lookup.insert( ([255,255,255],[255,255,255,255,255,255]), 2);

    block_adj_lookup.insert( ([1,1,0],[1,1,0,1,1,0]), 3);
    block_adj_lookup.insert( ([255,255,0],[255,255,0,255,255,0]), 4);
    block_adj_lookup.insert( ([0,255,255],[0,255,255,0,255,255]), 5);
    block_adj_lookup.insert( ([0,1,1],[0,1,1,0,1,1]), 6);

    block_adj_lookup.insert( ([0,0,0],[0,0,0,0,0,1]), 7);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,0,0,255]), 8);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,0,1,0]), 9);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,0,255,0]), 10);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,1,0,0]), 11);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,255,0,0]), 12);
    block_adj_lookup.insert( ([0,0,0],[0,0,1,0,0,0]), 13);
    block_adj_lookup.insert( ([0,0,0],[0,0,255,0,0,0]), 14);
    block_adj_lookup.insert( ([0,0,0],[0,1,0,0,0,0]), 15);
    block_adj_lookup.insert( ([0,0,0],[0,255,0,0,0,0]), 16);
    block_adj_lookup.insert( ([0,0,0],[1,0,0,0,0,0]), 17);
    block_adj_lookup.insert( ([0,0,0],[255,0,0,0,0,0]), 18);
    block_adj_lookup.insert( ([0,0,1],[0,0,0,0,0,0]), 19);
    block_adj_lookup.insert( ([0,0,255],[0,0,0,0,0,0]), 20);
    block_adj_lookup.insert( ([0,1,0],[0,0,0,0,0,0]), 21);
    block_adj_lookup.insert( ([0,255,0],[0,0,0,0,0,0]), 22);
    block_adj_lookup.insert( ([1,0,0],[0,0,0,0,0,0]), 23);
    block_adj_lookup.insert( ([255,0,0],[0,0,0,0,0,0]), 24);

    block_adj_lookup.insert( ([255,255,255],[0,0,0,0,0,0]), 25);
    block_adj_lookup.insert( ([1,1,1],[0,0,0,0,0,0]), 26);
    block_adj_lookup.insert( ([0,0,0],[1,1,1,0,0,0]), 27);
    block_adj_lookup.insert( ([0,0,0],[255,255,255,0,0,0]), 28);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,1,1,1]), 29);
    block_adj_lookup.insert( ([0,0,0],[0,0,0,255,255,255]), 30);

    block_adj_lookup.insert( ([0,0,0],[255,255,255,255,255,255]), 31);
    block_adj_lookup.insert( ([255,255,255],[0,0,0,255,255,255]), 32);
    block_adj_lookup.insert( ([255,255,255],[255,255,255,0,0,0]), 33);
    block_adj_lookup.insert( ([1,1,1],[0,0,0,1,1,1]), 34);
    block_adj_lookup.insert( ([0,0,0],[1,1,1,1,1,1]), 35);
    block_adj_lookup.insert( ([1,1,1],[1,1,1,0,0,0]), 36);

    
    block_adj_lookup.insert( ([1,0,0],[1,0,0,1,0,0]), 37);
    block_adj_lookup.insert( ([255,0,0],[255,0,0,255,0,0]), 38);
    block_adj_lookup.insert( ([0,1,0],[0,1,0,0,1,0]), 39);
    block_adj_lookup.insert( ([0,255,0],[0,255,0,0,255,0]), 40);
    block_adj_lookup.insert( ([0,0,1],[0,0,1,0,0,1]), 41);
    block_adj_lookup.insert( ([0,0,255],[0,0,255,0,0,255]), 42);

    block_adj_lookup.insert( ([0,0,255],[0,0,255,0,0,0]), 43);
    block_adj_lookup.insert( ([0,0,1],[0,0,1,0,0,0]), 44);
    block_adj_lookup.insert( ([0,1,0],[0,1,0,0,0,0]), 45);
    block_adj_lookup.insert( ([0,255,0],[0,255,0,0,0,0]), 46);
    block_adj_lookup.insert( ([1,0,0],[1,0,0,0,0,0]), 47);
    block_adj_lookup.insert( ([255,0,0],[255,0,0,0,0,0]), 48);
    block_adj_lookup.insert( ([0,0,255],[0,0,0,0,0,255]), 49);
    block_adj_lookup.insert( ([0,0,1],[0,0,0,0,0,1]), 50);
    block_adj_lookup.insert( ([0,1,0],[0,0,0,0,1,0]), 51);
    block_adj_lookup.insert( ([0,255,0],[0,0,0,0,255,0]), 52);
    block_adj_lookup.insert( ([1,0,0],[0,0,0,1,0,0]), 53);
    block_adj_lookup.insert( ([255,0,0],[0,0,0,255,0,0]), 54);
    block_adj_lookup.insert( ([0,0,0],[0,0,255,0,0,255]), 55);
    block_adj_lookup.insert( ([0,0,0],[0,0,1,0,0,1]), 56);
    block_adj_lookup.insert( ([0,0,0],[0,1,0,0,1,0]), 57);
    block_adj_lookup.insert( ([0,0,0],[0,255,0,0,255,0]), 58);
    block_adj_lookup.insert( ([0,0,0],[1,0,0,1,0,0]), 59);
    block_adj_lookup.insert( ([0,0,0],[255,0,0,255,0,0]), 60);

    block_adj_lookup.insert( ([0,0,0],[0,255,255,0,255,255]), 61);
    block_adj_lookup.insert( ([0,0,0],[0,1,1,0,1,1]), 62);
    block_adj_lookup.insert( ([0,0,0],[1,1,0,1,1,0]), 63);
    block_adj_lookup.insert( ([0,0,0],[255,255,0,255,255,0]), 64);
    block_adj_lookup.insert( ([0,0,0],[1,0,0,1,0,0]), 65);
    block_adj_lookup.insert( ([0,0,0],[255,0,255,255,0,255]), 66);
    block_adj_lookup.insert( ([0,255,255],[0,0,0,0,255,255]), 67);
    block_adj_lookup.insert( ([0,1,1],[0,0,0,0,1,1]), 68);
    block_adj_lookup.insert( ([1,1,0],[0,0,0,1,1,0]), 69);
    block_adj_lookup.insert( ([255,255,0],[0,0,0,255,255,0]), 70);
    block_adj_lookup.insert( ([1,0,1],[0,0,0,1,0,1]), 71);
    block_adj_lookup.insert( ([255,0,255],[0,0,0,255,0,255]), 72);
    block_adj_lookup.insert( ([0,255,255],[0,255,255,0,0,0]), 73);
    block_adj_lookup.insert( ([0,1,1],[0,1,1,0,0,0]), 74);
    block_adj_lookup.insert( ([1,1,0],[1,1,0,0,0,0]), 75);
    block_adj_lookup.insert( ([255,255,0],[255,255,0,0,0,0]), 76);
    block_adj_lookup.insert( ([1,0,1],[1,0,1,0,0,0]), 77);
    block_adj_lookup.insert( ([255,0,255],[255,0,255,0,0,0]), 78);
    //TODO test just this 3 lookup
    let mut rgb_cntr = 0;
    let mut run_cntr=0;
    let mut block_cntr=0;
    let mut luma_occurences=0;
    let mut red_pixel_run_amount=0;
    let mut run_occurrences=[0;8];

    //position;flipped position
    let rel_ref_lookup:[usize;13]=[channels,channels*image_header.width,channels*(1+image_header.width),2*channels,2*channels*image_header.width,channels*(2*image_header.width+1),channels*(image_header.width+2),channels*2*(image_header.width+1),3*channels,3*channels*image_header.width,
     channels*(image_header.width-1),channels*(image_header.width-2),channels*(2*image_header.width-1)];
    
    let mut color_states = vec![false/*ColorState::Initial*/;image_size];

    let mut same_color_diff_count=0;
    let mut predictor_count=0;
    //2 ways of doing: do subblock order, or normal order with blocks mod64
    //first Option to check if it already exists
    //None means it is not a block,  otherwise the block offset is stored
    //diff in block is diff to previous pixel or block base offset
    //image width divided rounded up as vector size
    //let mut blocks : Vec<Option<BlockDef>> = vec![None;image_header.width / BLOCK_SIZE + usize::from(image_header.width % BLOCK_SIZE != 0)];

    let mut most_used_patterns : collections::HashMap<(Pixel,Pixel,Pixel),usize> = collections::HashMap::new();
    let mut test_same_diff=0;
    //main loop
    for loop_index in 0..image_size/channels
    {
        let mut prev_position = position;
        position=loop_index*channels;
        //let vrun_pos=loop_index%image_header.width;

        //let pos_in_blocks=(loop_index%image_header.width)/BLOCK_SIZE;
        //TODO for r,g,b
        //3offsets for each color, only match when 3 colors have possible block. Alt: when using other algo's apply block instead of algo when applicable.
        //TODO clear block info when done with block, or when new.
        //calc begin block
        /*if loop_index%(image_header.width*BLOCK_SIZE)==0
        {
            blocks=vec![None;image_header.width / BLOCK_SIZE + usize::from(image_header.width % BLOCK_SIZE != 0)];
        }*/

        let is_not_red_run_item = !color_states[position] /*&& red_drun_list_iter[drun_pos].next() == None*/;
        let is_not_green_run_item = !color_states[position+1]/* && green_drun_list_iter[drun_pos].next() == None*/;
        let is_not_blue_run_item = !color_states[position+2]/* && blue_drun_list_iter[drun_pos].next() == None*/;

        

        if is_not_red_run_item ||is_not_green_run_item||is_not_blue_run_item
        {
            /*
            let mut avg_red=0;
            let mut avg_green=0;
            let mut avg_blue=0;
            let mut is_predict=false;let mut offset=1;
            if position>= channels*(1+image_header.width)
            {

                for i in [position-channels,position-channels*(image_header.width),position-channels*(1+image_header.width)]
                {
                    avg_red+=input_bytes[i];
                    avg_green+=input_bytes[i+1];
                    avg_blue+=input_bytes[i+2];
                }
                avg_red/=3;
                avg_green/=3;
                avg_blue/=3;
                if (input_bytes[position+1]-input_bytes[position+1-channels]==input_bytes[position+1-channels]-input_bytes[position+1-2*channels]
                                         &&input_bytes[position]-input_bytes[position-channels]==input_bytes[position-channels]-input_bytes[position-2*channels]
                                         &&input_bytes[position+2]-input_bytes[position+2-channels]==input_bytes[position+2-channels]-input_bytes[position+2-2*channels]
                                        ||input_bytes[position+1]==avg_green as u8&&input_bytes[position]==avg_red as u8&&input_bytes[position+2]==avg_blue as u8)
                {
                    predictor_count+=1;
                    
                    data.add_symbolu8(PREFIX_PREDICT, SC_PREFIXES);
                    is_predict=true;
                }
            }
            if !is_predict*/
            
            {
                
                let mut list_of_color_diffs=[0;3];
                /*if position>=channels*image_header.width&&input_bytes[prev_position+1]+input_bytes[prev_position]+input_bytes[prev_position+2]<=
                   input_bytes[position+1-channels*image_header.width]+input_bytes[position-channels*image_header.width]+input_bytes[position-channels*image_header.width+2]
                {
                    offset=channels*image_header.width;
                    prev_position=position-offset;
                }*/
                //green_diff
                list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[prev_position+1] as i16;
                if position>=channels*image_header.width
                {
                    list_of_color_diffs[1]=input_bytes[position+1] as i16-(input_bytes[position-channels*image_header.width+1] as i16+input_bytes[prev_position+1] as i16)/2;
                }
                //red_diff
                
                list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[prev_position] as i16;
                if position>=channels*image_header.width
                {
                    list_of_color_diffs[0]=input_bytes[position] as i16-(input_bytes[position-channels*image_header.width] as i16+input_bytes[prev_position] as i16)/2;
                }
                //blue_diff
                
                list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[prev_position+2] as i16;
                if position>=channels*image_header.width
                {
                    list_of_color_diffs[2]=input_bytes[position+2] as i16-(input_bytes[position-channels*image_header.width+2] as i16+input_bytes[prev_position+2] as i16)/2;
                }

                if position>0 &&(is_not_red_run_item && list_of_color_diffs[0]>=-4 && list_of_color_diffs[0]<4 || !is_not_red_run_item) &&
                   (is_not_green_run_item && list_of_color_diffs[1]>=-4 && list_of_color_diffs[1]<4 || !is_not_green_run_item) &&
                   (is_not_blue_run_item && list_of_color_diffs[2]>=-4 && list_of_color_diffs[2]<4 || !is_not_blue_run_item)
                {
                    //TODO add to BLOCK DIFF

                    {                    if list_of_color_diffs.iter().all_equal()
                        {same_color_diff_count+=1;}
                        data.add_symbolu8(PREFIX_SMALL_DIFF, SC_PREFIXES);
                        //if 
                        if is_not_red_run_item
                        {
                            data.add_symbolu8((4+list_of_color_diffs[0]) as u8, SC_SMALL_DIFF);
                        }
                        if is_not_green_run_item
                        {
                            data.add_symbolu8((4+list_of_color_diffs[1]) as u8, SC_SMALL_DIFF);
                        }
                        if is_not_blue_run_item
                        {
                            data.add_symbolu8((4+list_of_color_diffs[2]) as u8, SC_SMALL_DIFF);
                        }
                        amount_of_diffs+=1;
                    }
                        
                    

                }
                else
                {


                    let mut list_of_color_diffs=[0;3];
                    let mut is_luma=false;
                    for i in 0..=12
                    {
                        
                        if let Some(ref_pos)=position.checked_sub(rel_ref_lookup[i])
                        {
                        
                            //green_diff
                            list_of_color_diffs[1]=input_bytes[position+1].wrapping_sub(input_bytes[ref_pos+1]);
                            
                        
                            //red_diff
                            list_of_color_diffs[0]=input_bytes[position].wrapping_sub(input_bytes[ref_pos]);
                            //blue_diff
                            list_of_color_diffs[2]=input_bytes[position+2].wrapping_sub(input_bytes[ref_pos+2]);
                            list_of_color_diffs[0]=list_of_color_diffs[0].wrapping_sub(list_of_color_diffs[1]);
                            list_of_color_diffs[2]=list_of_color_diffs[2].wrapping_sub(list_of_color_diffs[1]);


                            //or take most occurred result instead of first result when adding from list of backrefs. 
                            //use run type(s) code stream
                            if list_of_color_diffs[0]==0&&list_of_color_diffs[2]==0
                            {test_same_diff+=1;}
                            
                            if position>0&&
                            (list_of_color_diffs[1]>=224||list_of_color_diffs[1]<32)&&
                            (is_not_red_run_item && (list_of_color_diffs[0]>=248 || list_of_color_diffs[0]<8) || !is_not_red_run_item)&&
                            (is_not_blue_run_item && (list_of_color_diffs[2]>=248 || list_of_color_diffs[2]<8) || !is_not_blue_run_item)
                            {

                                /*if position<=468360
                                {
                                    dbg!(position);
                                    dbg!(PREFIX_COLOR_LUMA);
                                }*/
                                data.add_symbolu8(PREFIX_COLOR_LUMA, SC_PREFIXES);
                                data.add_symbolusize(i, SC_LUMA_BACK_REF);

                                data.add_symbolu8((list_of_color_diffs[1].wrapping_add(32)) as u8, SC_LUMA_BASE_DIFF);
                                if is_not_red_run_item
                                {
                                    data.add_symbolu8((list_of_color_diffs[0].wrapping_add(8)) as u8, SC_LUMA_OTHER_DIFF);
                                }
                                if is_not_blue_run_item
                                {
                                    data.add_symbolu8((list_of_color_diffs[2].wrapping_add(8)) as u8, SC_LUMA_OTHER_DIFF);
                                }
                                luma_occurences+=1;
                                is_luma=true;
                                break;
                            }
                        
                        }
                    }
                    //write rgb
                    if is_luma==false
                    {
                        /*if position<=468360
                        {
                            dbg!(position);
                            dbg!(PREFIX_RGB);
                        }*/
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
                    }/*
                    }*/
                }
            }

            let mut run_length = 0;
            if is_not_red_run_item&&is_not_green_run_item&&is_not_blue_run_item
            {
                run_length = 0;
                //let mut offset_step=1;
                //split to see if exists
                //TODO handle edge cases
                let mut offset=channels;
                /*let prevxpos=position.saturating_sub(channels);
                let prevypos=position.saturating_sub(channels*image_header.width);
                if input_bytes[position].abs_diff(input_bytes[prevypos])+input_bytes[position+1].abs_diff(input_bytes[prevypos+1])+input_bytes[position+2].abs_diff(input_bytes[prevypos+2])<
                   input_bytes[position].abs_diff(input_bytes[prevxpos])+input_bytes[position+1].abs_diff(input_bytes[prevxpos+1])+input_bytes[position+2].abs_diff(input_bytes[prevxpos+2])
                {
                    offset=image_header.width*channels;
                }*/
                let mut run_loop_position=position+offset;
                
                while run_loop_position<image_size&&
                    input_bytes[run_loop_position]==input_bytes[position]&&!color_states[run_loop_position]&&
                    input_bytes[run_loop_position+1]==input_bytes[position+1]&&!color_states[run_loop_position+1]&&
                    input_bytes[run_loop_position+2]==input_bytes[position+2]&&!color_states[run_loop_position+2]
                {
                    run_length+=1;
                    //if run_length > 0
                    {

                        color_states[run_loop_position]=true;
                        color_states[run_loop_position+1]=true;
                        color_states[run_loop_position+2]=true;
                    }
                    run_loop_position+=offset;
                }
                if run_length > 0
                {
                    //run_count_red+=red_run_length;
                    red_pixel_run_amount+=run_length;
                    //run_length = run_length - 1;
                    let mut runlen_temp=run_length - 1;
                    /*color_states[position+channels]=true;
                    color_states[position+channels+1]=true;
                    color_states[position+channels+2]=true;*/
                    run_cntr+=1;
                    loop
                    {
                        data.add_symbolu8(PREFIX_RUN, SC_PREFIXES);
                        data.add_symbolusize(runlen_temp & 0b0000_0111, SC_RUN_LENGTHS);
                        run_occurrences[(runlen_temp & 0b0000_0111)]+=1;
                        if runlen_temp <8
                        {
                            break;
                        }
                        runlen_temp = runlen_temp >> 3;
                        
                    }
                }
                //test if horse worth when minimum >2
                //TODO identify patterns for block maps
                //+* operator,log,sqrt,?
                //change block size
                /*if run_length ==0
                {
                offset_step=3+image_header.width;
                //split to see if exists
                //TODO handle edge cases

                horse_run_loop_position=position+offset_step*channels;
                
                while horse_run_loop_position<image_size&&
                    input_bytes[horse_run_loop_position]==input_bytes[position]&&!color_states[horse_run_loop_position]&&
                    input_bytes[horse_run_loop_position+1]==input_bytes[position+1]&&!color_states[horse_run_loop_position+1]&&
                    input_bytes[horse_run_loop_position+2]==input_bytes[position+2]&&!color_states[horse_run_loop_position+2]
                {
                    run_length+=1;
                    color_states[horse_run_loop_position]=true;
                    color_states[horse_run_loop_position+1]=true;
                    color_states[horse_run_loop_position+2]=true;
                    horse_run_loop_position+=offset_step*channels;
                }
                if run_length > 0
                {

                    //run_count_red+=red_run_length;
                    red_pixel_run_amount+=run_length;
                    run_length = run_length - 1;
                    run_cntr+=1;
                    loop
                    {
                        /*if position<=468360
                        {
                            dbg!(position);
                            dbg!(PREFIX_RUN_HORSE);
                        }*/
                        data.add_symbolu8(PREFIX_RUN_HORSE, SC_PREFIXES);
                        data.add_symbolu8((run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(run_length & 0b0000_0111)]+=1;
                        if run_length <8
                        {
                            break;
                        }
                        run_length = run_length >> 3;
                        
                    }
                }

                run_length = 0;
                offset_step=1+3*image_header.width;
                //split to see if exists
                //TODO handle edge cases

                horse_run_loop_position=position+offset_step*channels;
                
                while horse_run_loop_position<image_size&&
                    input_bytes[horse_run_loop_position]==input_bytes[position]&&!color_states[horse_run_loop_position]&&
                    input_bytes[horse_run_loop_position+1]==input_bytes[position+1]&&!color_states[horse_run_loop_position+1]&&
                    input_bytes[horse_run_loop_position+2]==input_bytes[position+2]&&!color_states[horse_run_loop_position+2]
                {
                    run_length+=1;
                    color_states[horse_run_loop_position]=true;
                    color_states[horse_run_loop_position+1]=true;
                    color_states[horse_run_loop_position+2]=true;
                    horse_run_loop_position+=offset_step*channels;
                }
                if run_length > 0
                {

                    //run_count_red+=red_run_length;
                    red_pixel_run_amount+=run_length;
                    run_length = run_length - 1;
                    run_cntr+=1;
                    loop
                    {
                        /*if position<=468360
                        {
                            dbg!(position);
                            dbg!(PREFIX_RUN_HORSE2);
                        }*/
                        data.add_symbolu8(PREFIX_RUN_HORSE2, SC_PREFIXES);
                        data.add_symbolu8((run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(run_length & 0b0000_0111)]+=1;
                        if run_length <8
                        {
                            break;
                        }
                        run_length = run_length >> 3;
                        
                    }
                }
            }*/
               
                //TODO repeat prefix and add length to repeat pattern n times
                //TODO Block at end of run
                //TODO expand to all 2x2 with -1..1? 

                    let mut firstpix:[u8;3]=[69;3];
                    let mut pix23:[u8;6]=[69;6];
                    if position <= image_size-(2+image_header.width)*channels&&!color_states[position+3]&&!color_states[position+channels*image_header.width]&&!color_states[position+channels*image_header.width+3]
                    {
                        //TODO seeker algo to run on any shape, if it occurs enough
                        //limit in size to reduce how long it takes to compare all pixels


                                                                //

                        for i in 0..3
                        {
                            firstpix[i]=input_bytes[position+3+i].wrapping_sub(input_bytes[position+i]);
                        }
                        for i in 0..6
                        {
                            pix23[i]=input_bytes[(position+channels*image_header.width+i)].wrapping_sub(input_bytes[position+i%3]);
                        }
                    }
                    /*if let Some(&preset) = block_adj_lookup.get(&(firstpix,pix23))
                    {

                        data.add_symbolu8(PREFIX_ADJ_BLOCK, SC_PREFIXES);
                        data.add_symbolu8(preset, SC_ADJ_BLOCK_PRESET);
                        block_cntr+=1;
                        color_states[position+channels*image_header.width]=true;
                        color_states[position+channels*image_header.width+1]=true;
                        color_states[position+channels*image_header.width+2]=true;
                        color_states[position+channels]=true;
                        color_states[position+channels+1]=true;
                        color_states[position+channels+2]=true;
                        color_states[position+channels*image_header.width+3]=true;
                        color_states[position+channels*image_header.width+4]=true;
                        color_states[position+channels*image_header.width+5]=true;

                    }*/
                    /*else
                    {if position <= image_size-(2+image_header.width)*channels&&!color_states[position+3]&&!color_states[position+channels*image_header.width]&&!color_states[position+channels*image_header.width+3]

                        {
                            let mut key=(Pixel{   red : input_bytes[position+channels],
                                            green : input_bytes[position+1+channels],
                                            blue : input_bytes[position+2+channels] },
                                    Pixel{   red : input_bytes[position+image_header.width*channels],
                                            green : input_bytes[position+1+image_header.width*channels],
                                            blue : input_bytes[position+2+image_header.width*channels] },
                                    Pixel{   red : input_bytes[position+(1+image_header.width)*channels],
                                            green : input_bytes[position+1+(1+image_header.width)*channels],
                                            blue : input_bytes[position+2+(1+image_header.width)*channels] });
                            
                            if let Some(amount)=most_used_patterns.get_mut(&mut key)
                            {
                                *amount+=1;
                            }
                            else
                            {
                                most_used_patterns.insert(key, 1);
                            }
                        }
                    }*/
                
                
            }


            
            /*if red_run_length==1&& green_run_length==1&&blue_run_length==1
            {
                /*if position<=468360
                {
                    dbg!(position);
                    dbg!(PREFIX_BACK_REF);
                }*/
                data.add_symbolu8(PREFIX_BACK_REF, SC_PREFIXES);
                amount_of_refs+=1;
            }*/
            /*else
            {
                if red_run_length==1
                {
                    color_states[position]=false;
                }
                if 468357==position
                {
                    dbg!(red_run_length);
                    dbg!(color_states[position]);
                }
                if green_run_length==1
                {
                    color_states[position+1]=false;
                }
                if blue_run_length==1
                {
                    color_states[position+2]=false;
                }

            }*/
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
    dbg!(block_cntr);
    dbg!(amount_of_diffs);
    dbg!(predictor_count);
    
    dbg!(same_color_diff_count);
    let mut lijstje : Vec<(&(Pixel,Pixel,Pixel),&usize)>=most_used_patterns.iter().sorted_by(|a, b|  Reverse(a.1).cmp(&Reverse(b.1))).take(100).collect();
    //lijstje.sort_by(|a, b|  Reverse(a.1).cmp(&Reverse(b.1)));
    for i in 0..lijstje.len()
    {
        println!("{},{},{}",lijstje[i].0.0.red,lijstje[i].0.0.green,lijstje[i].0.0.blue);
        println!("{},{},{}",lijstje[i].0.1.red,lijstje[i].0.1.green,lijstje[i].0.1.blue);
        println!("{},{},{}",lijstje[i].0.2.red,lijstje[i].0.2.green,lijstje[i].0.2.blue);
        println!("amount:{}",lijstje[i].1);
    }
    let s:usize=lijstje.iter().map(|f|f.1).sum();
    dbg!(s);
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
    
    //let headertime = Instant::now();
    let mut decoder=  hfe::DecodeInput::new(Bitreader::new(image_reader));

    //0==PREFIX_RGB
    let mut rgb_lookup = SymbolstreamLookup::new(256);
    //1==SC_PREFIXES
    let mut prefix_lookup = SymbolstreamLookup::new(4);
    //2==SC_RUN_LENGTHS
    let mut runlength_lookup = SymbolstreamLookup::new(8);
    //3==SC_LUMA_BASE_DIFF
    let mut luma_base_diff_lookup = SymbolstreamLookup::new(64);
    //4==SC_LUMA_OTHER_DIFF
    let mut luma_other_diff_lookup = SymbolstreamLookup::new(16);
    //5==SC_LUMA_BACK_REF
    let mut luma_backref_lookup = SymbolstreamLookup::new(13);
    //6==SC_SMALL_DIFF
    let mut small_diff_lookup = SymbolstreamLookup::new(8);
    //6==SC_SMALL_DIFF
    //let mut adj_block_lookup = SymbolstreamLookup::new(79);
    
    decoder.read_header_into_tree(&mut rgb_lookup).unwrap();
    decoder.read_header_into_tree(&mut prefix_lookup).unwrap();
    decoder.read_header_into_tree(&mut runlength_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_backref_lookup).unwrap();
    decoder.read_header_into_tree(&mut small_diff_lookup).unwrap();
    //decoder.read_header_into_tree(&mut adj_block_lookup).unwrap();

    let rel_ref_lookup:[usize;13]=[channels,channels*image.width,channels*(1+image.width),2*channels,2*channels*image.width,channels*(2*image.width+1),channels*(image.width+2),channels*2*(image.width+1),3*channels,3*channels*image.width,
     channels*(image.width-1),channels*(image.width-2),channels*(2*image.width-1)];
    let mut prefix1=decoder.read_next_symbol(&prefix_lookup)?;

    
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
    /*let mut redhrun_color=0;
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
    let mut blue_vrun_list_iter=vec![(0..0).rev();image.width];*/

    let mut color_states = vec![false/*ColorState::Initial*/;image_size];

    //println!("time:{}:",headertime.elapsed().as_millis());
    while position<image_size 
    {
                if !color_states[position]
                {
                    
                    
                    //dbg!(position);
                    //dbg!(prefix1);
                    match prefix1
                    {
                        PREFIX_SMALL_DIFF=>
                        {
                            
                            if position>=channels*image.width
                            {
                                output_vec[position]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4+(output_vec[position-channels*image.width] as i16+output_vec[prev_pos] as i16)/2) as u8;
                                output_vec[position+1]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4+(output_vec[position+1-channels*image.width] as i16+output_vec[prev_pos+1] as i16)/2) as u8;
                                output_vec[position+2]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4+(output_vec[position+2-channels*image.width] as i16+output_vec[prev_pos+2] as i16)/2) as u8;

                            }
                            else
                            {
                                output_vec[position]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos] as i16)as u8;
                                output_vec[position+1]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos+1] as i16)as u8;
                                output_vec[position+2]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos+2] as i16)as u8;
                            }
                            #[cfg(debug_assertions)]
                            {
                                debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
                                debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
                                debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);
                            }
                        }
                        PREFIX_COLOR_LUMA=>
                        {
                            let backref = rel_ref_lookup[decoder.read_next_symbol(&luma_backref_lookup)? as usize];
                            let prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff_lookup)?.wrapping_sub(32);

                            output_vec[position+1]=prev_luma_base_diff .wrapping_add(output_vec[position-backref+1]);

                            let prev_luma_other_diff1=decoder.read_next_symbol(&luma_other_diff_lookup)?.wrapping_sub(8);
                            output_vec[position]=prev_luma_other_diff1.wrapping_add(prev_luma_base_diff.wrapping_add(output_vec[position-backref]));

                            let prev_luma_other_diff2=decoder.read_next_symbol(&luma_other_diff_lookup)?.wrapping_sub(8);
                            output_vec[position+2]=prev_luma_other_diff2.wrapping_add(prev_luma_base_diff.wrapping_add(output_vec[position-backref+2]));

                            #[cfg(debug_assertions)]
                            {
                                debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
                                debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
                                debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);
                            }
                        }
                        /*PREFIX_BACK_REF=>
                        {
                                output_vec[position]=output_vec[prev_pos];

                                output_vec[position+1]=output_vec[prev_pos+1];

                                output_vec[position+2]=output_vec[prev_pos+2];

                            #[cfg(debug_assertions)]
                            {
                                debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
                                debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
                                debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);
                            }
                            prev_pos=position;
                            position+=3;
                            prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                        }*/
                        PREFIX_RGB=>
                        {
                                output_vec[position]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos]);                            
                                output_vec[position+1]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+1]);
                                output_vec[position+2]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+2]);

                            #[cfg(debug_assertions)]
                            {
                                debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
                                debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
                                debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);
                            }

                        }
                        
                        _=>
                        {
                            eprintln!("error unkown token");
                        }

                    }

                    prev_pos=position;
                    position+=3;
                    prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    if prefix1==PREFIX_RUN
                    {
                        //TODO redrun
                        let mut temp_curr_runcount: u8=0;
                        let mut red_run_length=0;
                        while prefix1 == PREFIX_RUN
                        {
                            //run lengths
                            red_run_length+=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                            temp_curr_runcount += 3;
                            prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                        }

                        red_run_length += 1;
                        
                        //let pos=position;
                        
                        let mut curr_pix: [u8;3]=[0;3];
                        curr_pix.copy_from_slice(&output_vec[prev_pos..=prev_pos+2]);
                        for colors in output_vec[position..].chunks_exact_mut(3).take(red_run_length)
                        {
                            colors.copy_from_slice(&curr_pix);
                            /*#[cfg(debug_assertions)]
                            {
                                debug_assert!(dump[runpos]==output_vec[runpos],"expected: {}, output: {} at position {}",dump[runpos],output_vec[runpos],runpos);
                            }*/

                        }
                        position+=red_run_length*channels;
                        
                        //prev_pos=position;
                    }
                    //temp_time+=headertime.elapsed().as_nanos();
                }


                /*#[cfg(debug_assertions)]
                {

                    loopindex+=1;
                }*/
            

            /* }
        }*/
    
    }
    //println!("temp_time:{}: ",temp_time);
    Ok(image)
}
