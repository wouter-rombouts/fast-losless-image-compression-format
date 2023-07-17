const NICE: &[u8] = "nice".as_bytes();
use crate::bitwriter;
use crate::image::{Image, self};
use crate::symbol_set::{SymbolSetGroup, DiffSet, SymbolSet, LumaSet, RGBSet, RunSet};
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
//stream codes
pub(crate) const SC_RGB: u8 = 0;
pub(crate) const SC_PREFIXES: u8 = 1;
pub(crate) const SC_RUN_LENGTHS: u8 = 2;
pub(crate) const SC_LUMA_BASE_DIFF: u8 = 3;
pub(crate) const SC_LUMA_OTHER_DIFF: u8 = 4;
pub(crate) const SC_LUMA_BACK_REF: u8 = 5;
pub(crate) const SC_SMALL_DIFF: u8 = 6;
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
    let mut rgb_encoder=EncodedOutput::new( 256 );
    let mut prefix_encoder=EncodedOutput::new( 6 );
    let mut runlength_encoder=EncodedOutput::new( 8 );
    let mut luma_base_encoder=EncodedOutput::new( 64 );
    let mut luma_other_encoder=EncodedOutput::new( 16 );
    let mut luma_backref_encoder=EncodedOutput::new( 8 );
    let mut small_diff_encoder=EncodedOutput::new( 8 );

    //  3 types: run, backref and luma
    //data.add_output_type(3);
    let mut prev_run_count=0;
    //TODO fill in with most common colors
    //16 size, with 16 spares
    //hold slice or actual values
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [(u8,u8,u8);64] = [(0,0,0);64];

    //main loop
    let mut run_count_red = 1;
    let mut run_count_green = 1;
    let mut run_count_blue = 1;
    let mut rgb_cntr = 0;
    let mut run_cntr=0;
    let mut luma_occurences=0;
    let mut red_pixel_run_amount=0;
    let mut run_occurrences=[0;8];
    let mut symbolsets:Vec<[Option<SymbolSet>;3]> = vec![];
    //let mut run_lookup_table=[1,2,3,4,5,6,7,7];

    //TODO add symbols as normal, but also add the symbols for other streams
    // calculate aob for each stream at the end, based on preferred stream(same value as before)
    //reassign pixel to different preferred stream if smaller aob is found
    //repeat until satisfied with result
    //add result to output stream
    
    for loop_index in 0..image_size/channels
    {
        //TODO cache for calc_pos_from function when values are generated from run lookups
        let prev_position = position;
        position=image_header.calc_pos_from(loop_index)*channels;
        if run_count_red ==1 ||run_count_green==1||run_count_blue==1
        {
            //prev_position=position;      
            //TODO backreference remaining colors after runlength

            /*for i in 0..=3
            {
                //check if non run bytes are equal
                if ((run_count_red==1&&previous16_pixels_unique[i].0==input_bytes[position])||run_count_red > 1)&&
                   ((run_count_green==1&&previous16_pixels_unique[i].1==input_bytes[position+1])||run_count_green > 1)&&
                   ((run_count_blue==1&&previous16_pixels_unique[i].2==input_bytes[position+2])||run_count_blue > 1)
                {
                        ret_pos=i as u8;
                        break;
                }
            }
            if ret_pos != 99
            {

                data.add_symbolu8(PREFIX_BACK_REF, SC_PREFIXES);
                data.add_symbolu8(ret_pos, SC_BACK_REFS);
                back_ref_cntr+=1;
            }
            else*/
            {
                let mut symbol_set_group : [Option<SymbolSet>;3] = [None,None,None];


                //check color diff            
                
                let mut is_diff=false;

                let mut list_of_color_diffs=[0;3];
           
                    //green_diff
                    list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[prev_position+1] as i16;
                

                    //red_diff
                    list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[prev_position] as i16;


                    //blue_diff
                    list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[prev_position+2] as i16;

                if position>0&&(run_count_red == 1&&list_of_color_diffs[0]>=-4&&list_of_color_diffs[0]<4||run_count_red>1)&&
                   (run_count_green == 1&&list_of_color_diffs[1]>=-4&&list_of_color_diffs[1]<4||run_count_green>1)&&
                   (run_count_blue == 1&&list_of_color_diffs[2]>=-4&&list_of_color_diffs[2]<4||run_count_blue>1)
                {
                    let mut symbolset = DiffSet{ red_diff: None, green_diff: None, blue_diff: None };
                    //data.add_symbolu8(PREFIX_SMALL_DIFF, SC_PREFIXES);
                    if run_count_red == 1
                    {                    
                        symbolset.red_diff=Some((4+list_of_color_diffs[0]) as u8);
                        //data.add_symbolu8((4+list_of_color_diffs[0]) as u8, SC_SMALL_DIFF);
                    }
                    if run_count_green == 1
                    {
                        //data.add_symbolu8((4+list_of_color_diffs[1]) as u8, SC_SMALL_DIFF);
                        symbolset.green_diff=Some((4+list_of_color_diffs[1]) as u8);
                    }
                    if run_count_blue == 1
                    {
                        //data.add_symbolu8((4+list_of_color_diffs[2]) as u8, SC_SMALL_DIFF);
                        symbolset.blue_diff=Some((4+list_of_color_diffs[2]) as u8);
                    }
                    is_diff=true;
                    symbol_set_group[0]=Some(SymbolSet::DiffSet(symbolset));
                }
                //else
                {

                let mut symbolset = LumaSet{ red_diff: None, green_diff: 0, blue_diff: None, back_ref : 0 };
                let mut list_of_color_diffs=[0;3];
                let mut is_luma=false;
                for i in 0..=7
                {

                    //green_diff
                    list_of_color_diffs[1]=input_bytes[position+1] as i16-previous16_pixels_unique[i].1 as i16;
                

                    //red_diff
                    list_of_color_diffs[0]=input_bytes[position] as i16-previous16_pixels_unique[i].0 as i16;


                    //blue_diff
                    list_of_color_diffs[2]=input_bytes[position+2] as i16-previous16_pixels_unique[i].2 as i16;

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
                    (run_count_red==1&&list_of_color_diffs[0]>=-8&&list_of_color_diffs[0]<8||run_count_red > 1)&&
                    (run_count_blue==1&&list_of_color_diffs[2]>=-8&&list_of_color_diffs[2]<8||run_count_blue > 1)
                    {
                        symbolset.green_diff=(list_of_color_diffs[1]+32) as u8;
                        symbolset.back_ref=i as u8;
                        //data.add_symbolu8(PREFIX_COLOR_LUMA, SC_PREFIXES);
                        //data.add_symbolusize(i, SC_LUMA_BACK_REF);
                        
                        //data.add_symbolu8((list_of_color_diffs[1]+32) as u8, SC_LUMA_BASE_DIFF);
                        if run_count_red==1
                        {
                            symbolset.red_diff=Some((list_of_color_diffs[0]+8) as u8);
                            //data.add_symbolu8((list_of_color_diffs[0]+8) as u8, SC_LUMA_OTHER_DIFF);
                        }
                        if run_count_blue==1
                        {
                            symbolset.blue_diff=Some((list_of_color_diffs[2]+8) as u8);
                            //data.add_symbolu8((list_of_color_diffs[2]+8) as u8, SC_LUMA_OTHER_DIFF);
                        }
                        //symbol_set_group[0]=Some(SymbolSet::LumaSet(symbolset));
                        for el in symbol_set_group.iter_mut()
                        {//el=Some(SymbolSet::Rgbset(symbolset));
                            if *el==None
                            {
                                *el=Some(SymbolSet::LumaSet(symbolset));
                                break;
                            }
                        }
                        luma_occurences+=1;
                        is_luma=true;
                        //previous16_pixels_unique[previous16_pixels_unique_offset]=(input_bytes[position],input_bytes[position + 1],input_bytes[position + 2]);
                        //previous16_pixels_unique_offset=(previous16_pixels_unique_offset+1)%8;
                        break;
                    }
                    //TODO after loop is done
                    /*else
                    {

                    }*/
                
                }

                //TODO update  previous16_pixels_unique when match is found
                //write rgb
                //if is_luma==false
                {
                    let mut symbolset = RGBSet{ red: None, green: None, blue: None };
                    //data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);

                    rgb_cntr+=1;
                    if run_count_red == 1
                    {
                        symbolset.red=Some(input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0}));
                        //data.add_symbolu8(input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0}), SC_RGB);
                    }        
                    if run_count_green == 1
                    {
                        symbolset.green=Some(input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0}));
                        //data.add_symbolu8(input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0}), SC_RGB);

                    }
                    if run_count_blue == 1
                    {
                        symbolset.blue=Some(input_bytes[position+2].wrapping_sub(if position>0{input_bytes[prev_position+2]}else{0}));
                        //data.add_symbolu8(input_bytes[position+2].wrapping_sub(if position>0{input_bytes[prev_position+2]}else{0}), SC_RGB);

                    }
                    if is_diff==false
                    {
                        previous16_pixels_unique[previous16_pixels_unique_offset]=(input_bytes[position],input_bytes[position + 1],input_bytes[position + 2]);
                        previous16_pixels_unique_offset=(previous16_pixels_unique_offset+1)%8;
                    }
                    
                    for el in symbol_set_group.iter_mut()
                    {//el=Some(SymbolSet::Rgbset(symbolset));
                        if *el==None
                        {
                            *el=Some(SymbolSet::Rgbset(symbolset));
                            break;
                        }
                    }
                    
                }
                
                }
                /* }
                }}*/
                //write to unique previous n cache
                
            
                symbolsets.push(symbol_set_group);
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
        
        
        prev_run_count=0;
        let mut symbol_set_group : [Option<SymbolSet>;3] = [None,None,None];
        let mut runset=RunSet{ red_run: None, green_run: None, blue_run: None };
            //check for color run
            if run_count_red==1
            {
                let mut red_run_length = 0;
                //let mut prev_red_run_loop_position=position;
                let mut red_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while red_run_loop_position<image_size&&
                      input_bytes[red_run_loop_position]==input_bytes[position]/*&&
                      input_bytes[red_run_loop_position+1]!=input_bytes[prev_red_run_loop_position+1]&&
                      input_bytes[red_run_loop_position+2]!=input_bytes[prev_red_run_loop_position+2]*/
                {
                    red_run_length+=1;
                    //prev_red_run_loop_position=red_run_loop_position;
                    red_run_loop_position=image_header.calc_pos_from(loop_index+red_run_length+1)*channels;
                }

                if red_run_length > 2
                {
                    //add red runlength
                    //loop
                    
                    /*if loop_index<=140480
                    {dbg!(position);dbg!(red_run_length);
                        dbg!(input_bytes[(position)]);
                    dbg!(red_run_loop_position);
                    dbg!(input_bytes[(red_run_loop_position)]);}*/
                    prev_run_count=red_run_length;
                    run_count_red+=red_run_length;
                    red_pixel_run_amount+=red_run_length;
                    red_run_length = red_run_length - 3;
                    run_cntr+=1;
                    runset.red_run=Some( red_run_length );
                    /*loop
                    {
                        
                        //runset.runs.push(( PREFIX_RED_RUN,(red_run_length & 0b0000_0111).try_into().unwrap()) );
                        //symbol_set_group[0] = Some(SymbolSet::RunSet());
                        //data.add_symbolu8(PREFIX_RED_RUN, SC_PREFIXES);
                        //data.add_symbolu8((red_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                        run_occurrences[(red_run_length & 0b0000_0111)]+=1;
                        if red_run_length <8
                        {
                            break;
                        }
                        red_run_length = red_run_length >> 3;
                        
                    }*/
                }
            }

            if run_count_green==1
            {
                let mut green_run_length = 0;
                //let mut prev_green_run_loop_position=position;
                let mut green_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while green_run_loop_position<image_size&&
                      input_bytes[green_run_loop_position+1]==input_bytes[position+1]/*&&
                      input_bytes[green_run_loop_position]!=input_bytes[prev_green_run_loop_position]&&
                      input_bytes[green_run_loop_position+2]!=input_bytes[prev_green_run_loop_position+2]*/
                {
                    green_run_length+=1;
                    //prev_green_run_loop_position=green_run_loop_position;
                    green_run_loop_position=image_header.calc_pos_from(loop_index+green_run_length+1)*channels;
                }

                if green_run_length > 2
                {
                    /*if loop_index<=140480
                    {dbg!(position);dbg!(green_run_length);
                        dbg!(input_bytes[(position)+1]);
                    dbg!(green_run_loop_position);
                    dbg!(input_bytes[(green_run_loop_position)+1]);}*/
                    /*if prev_run_count==green_run_length
                    {
                        data.add_symbolu8(PREFIX_PREV_INPUT, SC_PREFIXES);
                    }
                    else*/
                    {
                        //add green runlength
                        //loop
                        prev_run_count=green_run_length;
                        run_count_green+=green_run_length;
                        green_run_length = green_run_length - 3;
                        run_cntr+=1;
                        runset.green_run=Some( green_run_length );
                        /*loop
                        {
                            
                            runset.runs.push(( PREFIX_GREEN_RUN,(green_run_length & 0b0000_0111).try_into().unwrap()) );
                            
                             
                            //data.add_symbolu8(PREFIX_GREEN_RUN, SC_PREFIXES);
                            //data.add_symbolu8((green_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                            run_occurrences[(green_run_length & 0b0000_0111)]+=1;
                            if green_run_length <8
                            {
                                break;
                            }
                            green_run_length = green_run_length >> 3;
                        }*/
                    }
                }
            }

            if run_count_blue==1
            {
                let mut blue_run_length = 0;
                //let mut prev_blue_run_loop_position=position;
                let mut blue_run_loop_position=image_header.calc_pos_from(loop_index+1)*channels;
                
                while blue_run_loop_position<image_size&&
                      input_bytes[blue_run_loop_position+2]==input_bytes[position+2]/*&&
                      input_bytes[blue_run_loop_position+1]!=input_bytes[prev_blue_run_loop_position+1]&&
                      input_bytes[blue_run_loop_position]!=input_bytes[prev_blue_run_loop_position]*/
                {
                    blue_run_length+=1;
                    //prev_blue_run_loop_position=blue_run_loop_position;
                    blue_run_loop_position=image_header.calc_pos_from(loop_index+blue_run_length+1)*channels;
                }

                if blue_run_length > 2
                {/*if loop_index<=140480
                    {dbg!(position);dbg!(blue_run_length);
                        dbg!(input_bytes[(position)+2]);
                    dbg!(blue_run_loop_position);
                    dbg!(input_bytes[(blue_run_loop_position)+2]);}*/
                    /*if prev_run_count==blue_run_length
                    {
                        data.add_symbolu8(PREFIX_PREV_INPUT, SC_PREFIXES);
                    }
                    else*/
                    {
                        //add blue runlength
                        //loop
                        //prev_run_count=blue_run_length;
                        run_count_blue+=blue_run_length;
                        blue_run_length = blue_run_length - 3;
                        run_cntr+=1;
                        runset.blue_run=Some( blue_run_length );
                        /*loop
                        {
                            runset.runs.push(( PREFIX_BLUE_RUN,(blue_run_length & 0b0000_0111).try_into().unwrap()) );
                            //data.add_symbolu8(PREFIX_BLUE_RUN, SC_PREFIXES);
                            //data.add_symbolu8((blue_run_length & 0b0000_0111).try_into().unwrap(), SC_RUN_LENGTHS);
                            run_occurrences[(blue_run_length & 0b0000_0111)]+=1;
                            if blue_run_length <8
                            {
                                break;
                            }
                            blue_run_length = blue_run_length >> 3;
                        }*/
                    }
                }
            }
            if runset.red_run != None || runset.green_run != None || runset.blue_run != None
            {
                symbol_set_group[0]=Some(SymbolSet::RunSet(runset));
            }
            if symbol_set_group.iter().any(|x|*x != None)
            {
                symbolsets.push(symbol_set_group);
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
    //symbolsets
    for choices in &symbolsets
    {
        
            match &choices[0]
            {
                Some(symbolset)=>
                {
                    match symbolset
                    {
                        SymbolSet::Rgbset(rgbset) =>
                        {
                            //TODO don't add to data_vec
                            prefix_encoder.add_symbolu8(PREFIX_RGB);
                            if let Some(red)=rgbset.red
                            {
                                rgb_encoder.add_symbolu8(red);
                            }

                            if let Some(green)=rgbset.green
                            {
                                rgb_encoder.add_symbolu8(green);
                            }

                            if let Some(blue)=rgbset.blue
                            {
                                rgb_encoder.add_symbolu8(blue);
                            }
                            //break;
                        },
                        SymbolSet::DiffSet(diffset) => 
                        {
                            prefix_encoder.add_symbolu8(PREFIX_SMALL_DIFF);
                            if let Some(red_diff) = diffset.red_diff
                            {
                                small_diff_encoder.add_symbolu8(red_diff);
                            }
                            if let Some(green_diff) = diffset.green_diff
                            {
                                small_diff_encoder.add_symbolu8(green_diff);
                            }
                            if let Some(blue_diff) = diffset.blue_diff
                            {
                                small_diff_encoder.add_symbolu8(blue_diff);
                            }
                            
                            //break;
                        },
                        SymbolSet::LumaSet(lumaset) =>
                        {
                            prefix_encoder.add_symbolu8(PREFIX_COLOR_LUMA);
                            
                            luma_backref_encoder.add_symbolu8(lumaset.back_ref);
                            if let Some(red_diff) = lumaset.red_diff
                            {
                                luma_other_encoder.add_symbolu8(red_diff);
                            }

                            luma_base_encoder.add_symbolu8(lumaset.green_diff);

                            if let Some(blue_diff) = lumaset.blue_diff
                            {
                                luma_other_encoder.add_symbolu8(blue_diff);
                            }
                            //break;
                        },
                        SymbolSet::RunSet(runset) => 
                        {
                            if let Some( red_run_len)=runset.red_run
                            {
                                let mut red_run_length=red_run_len;
                                loop
                                {
                                    //type
                                    prefix_encoder.add_symbolu8(PREFIX_RED_RUN);
                                    //runlength
                                    runlength_encoder.add_symbolu8((red_run_length & 0b0000_0111).try_into().unwrap());

                                    if red_run_length <8
                                    {
                                        break;
                                    }

                                    red_run_length = red_run_length >> 3;
                                }
                            }
                            if let Some(green_run_len)=runset.green_run
                            {
                                let mut green_run_length=green_run_len;
                                loop
                                {
                                    //type
                                    prefix_encoder.add_symbolu8(PREFIX_GREEN_RUN);
                                    //runlength
                                    runlength_encoder.add_symbolu8((green_run_length & 0b0000_0111).try_into().unwrap());

                                    if green_run_length <8
                                    {
                                        break;
                                    }

                                    green_run_length = green_run_length >> 3;
                                }
                            }
                            if let Some( blue_run_len)=runset.blue_run
                            {
                                let mut blue_run_length=blue_run_len;
                                loop
                                {
                                    //type
                                    prefix_encoder.add_symbolu8(PREFIX_BLUE_RUN);
                                    //runlength
                                    runlength_encoder.add_symbolu8((blue_run_length & 0b0000_0111).try_into().unwrap());

                                    if blue_run_length <8
                                    {
                                        break;
                                    }

                                    blue_run_length = blue_run_length >> 3;
                                }
                            }
                            
                        },
                    }
                },
                None=>
                {
                    //break;
                }
            }
    }
    //calc aob for each stream(add to enum types?)
    let bcodes_rgb=rgb_encoder.to_bcodes();
    let bcodes_prefix=prefix_encoder.to_bcodes();
    let bcodes_diff=small_diff_encoder.to_bcodes();
    let bcodes_luma_base=luma_base_encoder.to_bcodes();
    let bcodes_luma_other=luma_other_encoder.to_bcodes();
    let bcodes_luma_backref=luma_backref_encoder.to_bcodes();
    let bcodes_runlength=runlength_encoder.to_bcodes();
    let mut bitwriter=Bitwriter::new(output_writer);
    EncodedOutput::writebcodes(&bcodes_rgb,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_prefix,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_runlength,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_luma_base,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_luma_other,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_luma_backref,&mut bitwriter)?;
    EncodedOutput::writebcodes(&bcodes_diff,&mut bitwriter)?;
    let get_total_aob=| symbolset : &SymbolSet |
    ->u8
    {
        match symbolset
        {
            SymbolSet::Rgbset(rgbset) => 
            {
                bcodes_prefix[PREFIX_RGB as usize].aob+
                if let Some(red) = rgbset.red
                {
                    bcodes_rgb[ red as usize].aob
                }
                else
                {
                    0
                }+
                if let Some(green) = rgbset.green
                {
                    bcodes_rgb[ green as usize].aob
                }
                else
                {
                    0
                }+
                if let Some(blue) = rgbset.blue
                {
                    bcodes_rgb[ blue as usize].aob
                }
                else
                {
                    0
                }
            },
            SymbolSet::DiffSet(diffset) => 
            {
                bcodes_prefix[PREFIX_SMALL_DIFF as usize].aob+
                if let Some(red_diff) = diffset.red_diff
                {
                    bcodes_diff[ red_diff as usize].aob
                }
                else
                {
                    0
                }+
                if let Some(green_diff) = diffset.green_diff
                {
                    bcodes_diff[ green_diff as usize].aob
                }
                else
                {
                    0
                }+
                if let Some(blue_diff) = diffset.blue_diff
                {
                    bcodes_diff[ blue_diff as usize].aob
                }
                else
                {
                    0
                }
                
            },
            SymbolSet::LumaSet(lumaset) => 
            {
                bcodes_prefix[PREFIX_COLOR_LUMA as usize].aob+
                if let Some(red_diff)=lumaset.red_diff
                {
                    bcodes_luma_other[red_diff as usize].aob
                }
                else
                {
                    0
                }
                +
                bcodes_luma_base[lumaset.green_diff as usize].aob+bcodes_luma_backref[lumaset.back_ref as usize].aob+
                if let Some(blue_diff)=lumaset.blue_diff
                {
                    bcodes_luma_other[blue_diff as usize].aob
                }
                else
                {
                    0
                }
            },
            SymbolSet::RunSet(_) => {0},
        }
    };





    //this ok?
    for mut choices in symbolsets
    {
        choices.sort_by_key( |a| if let Some(symset) = a{ get_total_aob(symset) } else{ u8::MAX } );
        match &choices[0]
        {
            Some(SymbolSet::Rgbset(rgbset)) => 
            {
                bcodes_prefix[PREFIX_RGB as usize].write_data(&mut bitwriter)?;
                if let Some(red) = rgbset.red
                {
                    bcodes_rgb[ red as usize].write_data(&mut bitwriter)?;
                }
                if let Some(green) = rgbset.green
                {
                    bcodes_rgb[ green as usize].write_data(&mut bitwriter)?;
                }
                if let Some(blue) = rgbset.blue
                {
                    bcodes_rgb[ blue as usize].write_data(&mut bitwriter)?;
                }
            },
            Some(SymbolSet::DiffSet(diffset)) => 
            {
                //TODO don't write when None
                bcodes_prefix[PREFIX_SMALL_DIFF as usize].write_data(&mut bitwriter)?;
                if let Some(red_diff) = diffset.red_diff
                {
                    bcodes_diff[ red_diff as usize].write_data(&mut bitwriter)?;
                }

                if let Some(green_diff) = diffset.green_diff
                {
                    bcodes_diff[ green_diff as usize].write_data(&mut bitwriter)?;
                }

                if let Some(blue_diff) = diffset.blue_diff
                {
                    bcodes_diff[ blue_diff as usize].write_data(&mut bitwriter)?;
                }
            },
            Some(SymbolSet::LumaSet(lumaset)) => 
            {
                bcodes_prefix[PREFIX_COLOR_LUMA as usize].write_data(&mut bitwriter)?;

                bcodes_luma_backref[lumaset.back_ref as usize].write_data(&mut bitwriter)?;
                bcodes_luma_base[lumaset.green_diff as usize].write_data(&mut bitwriter)?;

                if let Some(red_diff)=lumaset.red_diff
                {
                    bcodes_luma_other[red_diff as usize].write_data(&mut bitwriter)?;
                }

                if let Some(blue_diff)=lumaset.blue_diff
                {
                    bcodes_luma_other[blue_diff as usize].write_data(&mut bitwriter)?;
                }
            },
            Some(SymbolSet::RunSet(runset)) => 
            {
                if let Some(mut red_run_length)=runset.red_run
                {
                    loop
                    {
                        
                        bcodes_prefix[PREFIX_RED_RUN as usize].write_data(&mut bitwriter)?;
                        bcodes_runlength[red_run_length & 0b0000_0111].write_data(&mut bitwriter)?;

                        if red_run_length <8
                        {
                            break;
                        }

                        red_run_length = red_run_length >> 3;
                    }
                }
                if let Some(mut green_run_length)=runset.green_run
                {
                    loop
                    {
                        bcodes_prefix[PREFIX_GREEN_RUN as usize].write_data(&mut bitwriter)?;
                        bcodes_runlength[green_run_length & 0b0000_0111].write_data(&mut bitwriter)?;

                        if green_run_length <8
                        {
                            break;
                        }

                        green_run_length = green_run_length >> 3;
                    }
                }
                if let Some(mut blue_run_length)=runset.blue_run
                {
                    loop
                    {
                        bcodes_prefix[PREFIX_BLUE_RUN as usize].write_data(&mut bitwriter)?;
                        bcodes_runlength[blue_run_length & 0b0000_0111].write_data(&mut bitwriter)?;

                        if blue_run_length <8
                        {
                            break;
                        }

                        blue_run_length = blue_run_length >> 3;
                    }
                }
                        
                    
            },
            None => {},
        }
    }
    //reorder symbolsets choices to get least aob

    //use iter to process new values to hfe


    //data.to_encoded_output(&mut bitwriter)?;
    
    //TODO: write cache to writer?
    bitwriter.writer.write_all(&[(bitwriter.cache>>24).try_into().unwrap()])?;

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
    let mut small_diff_lookup = SymbolstreamLookup::new(8);
    
    decoder.read_header_into_tree(&mut rgb_lookup).unwrap();
    decoder.read_header_into_tree(&mut prefix_lookup).unwrap();
    decoder.read_header_into_tree(&mut runlength_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_backref_lookup).unwrap();
    decoder.read_header_into_tree(&mut small_diff_lookup).unwrap();


    //let mut prefix_1bits=bitreader.read_bitsu8(1)?;
    //let mut prefix_2bits: u8=bitreader.read_bitsu8(1)?;

    let mut prefix1=decoder.read_next_symbol(&prefix_lookup)?;
    //let width = width as usize;
    let mut previous16_pixels_unique_offset = 0;
    let mut previous16_pixels_unique : [[u8;3];64] = [[0,0,0];64];
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
    
    let mut pos_subblock_lookup =Vec::<usize>::with_capacity(image::SUBBLOCK_HEIGHT_MAX*image::SUBBLOCK_WIDTH_MAX);
    for y in 0..image::SUBBLOCK_HEIGHT_MAX
    {
        for x in 0..image::SUBBLOCK_WIDTH_MAX
        {
            pos_subblock_lookup.push(channels*(y*image.width+if y%2==1 {image::SUBBLOCK_WIDTH_MAX-x-1}else{x}));
        }
    }
    let mut pos_subblock_lookup_alt =Vec::<usize>::with_capacity(image::SUBBLOCK_HEIGHT_MAX*image::SUBBLOCK_WIDTH_MAX);
    for y in 0..image::SUBBLOCK_HEIGHT_MAX
    {
        for x in 0..image::SUBBLOCK_WIDTH_MAX
        {
            pos_subblock_lookup_alt.push(channels*((image::SUBBLOCK_HEIGHT_MAX-y-1) * image.width+if y%2==1 {image::SUBBLOCK_WIDTH_MAX-x-1}else{x}));
        }
    }

    let mut pos_subblock_xleftover_lookup: Vec<usize>;
    let mut pos_subblock_yleftover_lookup: Vec<usize>;
    let mut list_of_subblocks_in_widthblock:Vec<&[usize]>=Vec::with_capacity(image.subblocks_in_width+1);
    let mut list_of_subblocks_in_bottom_widthblock:Vec<&[usize]>=Vec::with_capacity(image.subblocks_in_width+1);
    let mut pos_subblock_xyleftover_lookup: Vec<usize>;
    for n in 0..image.subblocks_in_width
    {
        list_of_subblocks_in_widthblock.push(if n%2==0{&pos_subblock_lookup}else{&pos_subblock_lookup_alt});
    }
    if image.width_subblock_leftover>0
    {
        pos_subblock_xleftover_lookup=Vec::with_capacity(image.width_subblock_leftover*image::SUBBLOCK_HEIGHT_MAX);
        for h in 0..image::SUBBLOCK_HEIGHT_MAX
        {
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
    for _ in 0..image.subblocks_in_height
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
    let mut prev_pos=0;
    let mut curr_lengths: [usize;3]=[0;3];
    #[cfg(debug_assertions)]
    let mut loopindex=0;
    
    for y in 0..list_of_subblocks_in_heightblock.len()
    {
        for x in 0..list_of_subblocks_in_heightblock[y].len()
        {
            for i in 0..list_of_subblocks_in_heightblock[y][x].len()
            {
                //y, then x
                prev_pos=position;
                position = channels*(y*image.width_block_size+x*image::SUBBLOCK_WIDTH_MAX)+list_of_subblocks_in_heightblock[y][x][i];

                if curr_lengths.iter().any(|&x| x == 0)
                {


                    //let headertime = Instant::now();

                    match prefix1
                    {
                        PREFIX_SMALL_DIFF=>
                        {
                            if curr_lengths[0]==0
                            {
                                output_vec[position]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos] as i16)as u8;

                            }
                            else
                            {
                                curr_lengths[0] -= 1;
                                output_vec[position]=run_values[0];
                            }
                            if curr_lengths[1]==0
                            {
                                output_vec[position+1]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos+1] as i16)as u8;
                            }
                            else
                            {
                                curr_lengths[1] -= 1;
                                output_vec[position+1]=run_values[1];
                            }
                            if curr_lengths[2]==0
                            {
                                output_vec[position+2]=(decoder.read_next_symbol(&small_diff_lookup)? as i16-4 +output_vec[prev_pos+2] as i16)as u8;
                            }
                            else
                            {
                                curr_lengths[2] -= 1;
                                output_vec[position+2]=run_values[2];
                            }
                        }
                        PREFIX_COLOR_LUMA=>
                        {
                            let backref = decoder.read_next_symbol(&luma_backref_lookup)?;

                            prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff_lookup)? as i16-32;
                            
                            output_vec[position+1]=(prev_luma_base_diff + (previous16_pixels_unique[backref as usize][1] as i16)) as u8;

                            if curr_lengths[1]>0
                            {
                                curr_lengths[1] -= 1;
                                output_vec[position+1]=run_values[1];
                            }
                            if curr_lengths[0]>0
                            {
                                
                                curr_lengths[0] -= 1;
                                output_vec[position]=run_values[0];
                            }
                            else
                            {
                                prev_luma_other_diff1=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position]=(prev_luma_other_diff1 + prev_luma_base_diff+(previous16_pixels_unique[backref as usize][0] as i16)) as u8;
                                
                            }
                            if curr_lengths[2]>0
                            {
                                
                                curr_lengths[2] -= 1;
                                output_vec[position+2]=run_values[2];
                            }
                            else
                            {
                                prev_luma_other_diff2=decoder.read_next_symbol(&luma_other_diff_lookup)? as i16-8;
                                output_vec[position+2]=(prev_luma_other_diff2 + prev_luma_base_diff+(previous16_pixels_unique[backref as usize][2] as i16)) as u8;
                            }
                            previous16_pixels_unique[previous16_pixels_unique_offset]=[output_vec[position],output_vec[position + 1],output_vec[position + 2]];
                            previous16_pixels_unique_offset=(previous16_pixels_unique_offset+1)%8;
                        }
                        _=>
                        {
                            for i in 0..=2
                            {
                                if curr_lengths[i] == 0
                                {              
                                    //RGB
                                    output_vec[position+i]=decoder.read_next_symbol(&rgb_lookup)?.wrapping_add(output_vec[prev_pos+i]);
                                }
                                else 
                                {
                                    curr_lengths[i] -= 1;
                                    output_vec[position+i]=run_values[i];

                                }
                            }
                            previous16_pixels_unique[previous16_pixels_unique_offset]=[output_vec[position],output_vec[position + 1],output_vec[position + 2]];
                            previous16_pixels_unique_offset=(previous16_pixels_unique_offset+1)%8;
                        }

                    }

                    //temp_time+=headertime.elapsed().as_nanos();

                    prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    let mut temp_curr_runcount: u8=0;

                    while prefix1 == PREFIX_RED_RUN
                    {
                        //run lengths
                        curr_lengths[PREFIX_RED_RUN as usize] +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }
                    
                    if temp_curr_runcount>0
                    {

                        curr_lengths[PREFIX_RED_RUN as usize] += 3;
                        run_values[0]=output_vec[position];
                        temp_curr_runcount=0;
                    }

                    while prefix1 == PREFIX_GREEN_RUN
                    {  
                        //run lengths
                        curr_lengths[PREFIX_GREEN_RUN as usize] +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }   
                
                    if temp_curr_runcount>0
                    {

                        curr_lengths[PREFIX_GREEN_RUN as usize] += 3;
                                    
                        run_values[1]=output_vec[position+1];
                        temp_curr_runcount=0;
                    }

                    while prefix1 == PREFIX_BLUE_RUN
                    {  
                        //run lengths
                        curr_lengths[PREFIX_BLUE_RUN as usize] +=(decoder.read_next_symbol(&runlength_lookup)? as usize) << temp_curr_runcount;
                        temp_curr_runcount += 3;
                        prefix1 = decoder.read_next_symbol(&prefix_lookup)?;
                    }

                    if temp_curr_runcount>0
                    {

                        curr_lengths[PREFIX_BLUE_RUN as usize] += 3;
                                    
                        run_values[2]=output_vec[position+2];
                    }


                    //dbg!(prefix1);
                }
                else
                {
                    for i in 0..=2
                    {
                        curr_lengths[i] -= 1;
                        output_vec[position+i]=run_values[i];
                    }
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
            

            }
        }
    }
    //println!("temp_time:{}: ",temp_time);
    Ok(image)
}
