const NICE: &[u8] = "nice".as_bytes();
//use itertools::Itertools;

use crate::image::Image;
//use crate::pixel::Pixel;
//use std::cmp::Reverse;
//use std::collections::HashMap;
use std::{io, fs};
//use std::{time::Instant, *};
use crate::hfe::{EncodedOutput, self, SymbolstreamLookup, SymbolstreamLookupUsize};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
//pub(crate) const PREFIX_RUN: u8 = 2;

//pub(crate) const PREFIX_RUN: u8 = 0;
pub(crate) const PREFIX_RGB: u8 = 0;
pub(crate) const PREFIX_COLOR_LUMA: u8 = 1;
pub(crate) const PREFIX_SMALL_DIFF: u8 = 2;
pub(crate) const PREFIX_COLOR_LUMA2: u8 = 3;
pub(crate) const PREFIX_RUN_1: u8 = 4;
pub(crate) const PREFIX_RUN_2: u8 = 5;
pub(crate) const PREFIX_RUN_3: u8 = 6;
pub(crate) const PREFIX_RUN_4: u8 = 7;
pub(crate) const PREFIX_RUN_5: u8 = 8;
pub(crate) const PREFIX_RUN_6: u8 = 9;
pub(crate) const PREFIX_RUN_7: u8 = 10;
pub(crate) const PREFIX_RUN_8: u8 = 11;
pub(crate) const PREFIX_BACK_REF1: u8 = 12;
pub(crate) const PREFIX_BACK_REF2: u8 = 13;
pub(crate) const PREFIX_BACK_REF3: u8 = 14;
pub(crate) const PREFIX_BACK_REF4: u8 = 15;
pub(crate) const PREFIX_BACK_REF5: u8 = 16;

/*pub(crate) const PREFIX_VRUN_1: u8 = 13;
pub(crate) const PREFIX_VRUN_2: u8 = 14;
pub(crate) const PREFIX_VRUN_3: u8 = 15;
pub(crate) const PREFIX_VRUN_4: u8 = 16;
pub(crate) const PREFIX_VRUN_5: u8 = 17;
pub(crate) const PREFIX_VRUN_6: u8 = 18;
pub(crate) const PREFIX_VRUN_7: u8 = 19;
pub(crate) const PREFIX_VRUN_8: u8 = 20;*/
//pub(crate) const PREFIX_PREDICT: u8 = 5;
//pub(crate) const PREFIX_REF: u8 = 6;
//stream codes
pub(crate) const SC_RGB: u8 = 0;
pub(crate) const SC_PREFIXES: u8 = 1;
//pub(crate) const SC_RUN_LENGTHS: u8 = 2;
pub(crate) const SC_LUMA_BASE_DIFF: u8 = 2;
pub(crate) const SC_LUMA_OTHER_DIFF: u8 = 3;
pub(crate) const SC_LUMA_BACK_REF: u8 = 4;
pub(crate) const SC_SMALL_DIFF1: u8 = 5;
//pub(crate) const SC_BLOCK_DIFF: u8 = 7;
pub(crate) const SC_LUMA_BASE_DIFF2: u8 = 6;
pub(crate) const SC_LUMA_OTHER_DIFF2: u8 = 7;
//pub(crate) const SC_SMALL_DIFF2: u8 = 9;
//pub(crate) const SC_SMALL_DIFF3: u8 = 10;
pub(crate) const SC_LUMA_OTHER_DIFFB2: u8 = 8;
//pub(crate) const SC_BACK_REF: u8 = 9;
pub(crate) const SC_RGB2: u8 = 9;
//pub(crate) const SC_LUMA_BACK_REF2: u8 = 9;


//pub(crate) const BLOCK_SIZE: usize = 4;
//pub(crate) const BLOCK_RANGE: u8 = 8;
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
    let mut data_rgb =EncodedOutput::new( 512,image_size/100 );
    let mut data_prefixes =EncodedOutput::new( 17,image_size/100 );
    let mut data_luma_base_diff =EncodedOutput::new( 64,image_size/100 );
    let mut data_luma_other_diff =EncodedOutput::new( 32,image_size/100 );
    let mut data_luma_back_ref =EncodedOutput::new( 11,image_size/100 );
    let mut data_small_diff1 =EncodedOutput::new( 343,image_size/100 );
    let mut data_luma_base_diff2 =EncodedOutput::new( 64,image_size/100 );
    let mut data_luma_other_diff2 =EncodedOutput::new( 32,image_size/100 );
    let mut data_luma_other_diffb2 =EncodedOutput::new( 32,image_size/100 );
    let mut data_rgb2 =EncodedOutput::new( 32,image_size/100 );

    //initialize all output streams
    //0==SC_RGB
    /*data.add_output_type(512);
    //1==SC_PREFIXES
    data.add_output_type(17);
    //3==SC_LUMA_BASE_DIFF
    data.add_output_type(64);
    //4==SC_LUMA_OTHER_DIFF
    data.add_output_type(32);
    //5==SC_LUMA_BACK_REF
    data.add_output_type(11);
    //6==SC_SMALL_DIFF1
    data.add_output_type(343);
    
    //7==SC_LUMA_BASE_DIFF2
    data.add_output_type(64);
    //8==SC_LUMA_OTHER_DIFF2
    data.add_output_type(32);
    //9==SC_LUMA_OTHER_DIFFB2
    data.add_output_type(32);
    //10==SC_BACK_REF
    //data.add_output_type(5);
    //10==SC_RGB2
    data.add_output_type(32);*/

    //let mut prev_run_length=0;
    //7==SC_PREFIX_COUNT
    //data.add_output_type(4);
    //7==SC_ADJ_BLOCK
    //data.add_output_type(79);
    #[cfg(debug_assertions)]
    let mut amount_of_diffs=0;
    //keeps track of value that need to be written for the key(references the 3 pixels)
    //let mut block_adj_lookup : HashMap<([u8;3],[u8;6]),u8>=HashMap::with_capacity(79);
    #[cfg(debug_assertions)]
    let mut rgb_cntr = 0;
    #[cfg(debug_assertions)]
    let mut run_cntr=0;
    #[cfg(debug_assertions)]
    let mut luma_occurences=0;
    #[cfg(debug_assertions)]
    let mut luma_occurences2=0;
    #[cfg(debug_assertions)]
    let mut pixel_run_amount=0;
    #[cfg(debug_assertions)]
    let mut back_ref_amount=0;
    #[cfg(debug_assertions)]
    let mut run_occurrences=[0;8];
    //TODO auto-vectorization, after backref?

    let rel_ref_lookup:[usize;11]=[channels,channels*image_header.width,channels*(image_header.width-1),channels*(image_header.width-3),3*channels,
    channels*(3*image_header.width-1),3*channels*image_header.width,channels*(3*image_header.width+1),channels*(image_header.width+3),channels*3*(image_header.width+1),channels*3*(image_header.width-1)];

    
    let back_ref_lookup:[usize;5]=[channels,channels*image_header.width,channels*(image_header.width-1),2*channels,2*channels*image_header.width];
    
    let mut vrun_iters=vec![(0usize..0).rev();image_header.width];
    //let mut blocks : Vec<Option<BlockDef>> = vec![None;image_header.width / BLOCK_SIZE + usize::from(image_header.width % BLOCK_SIZE != 0)];

    //TODO BWT before main loop, to test with correct backrefences
    //let mut most_used_patterns : HashMap<(Pixel,Pixel,Pixel),usize> = HashMap::new();
    //backref,greendiff,reddiff,bluediff
    //let mut most_used_lumadiff : HashMap<(u8,u8,u8),usize> = HashMap::new();
    //main loop
    let mut prev_position=0;
    //let mut positions_cache:[[u8; 3]; 8]=[[0,0,0];8];
    //let mut pos_cache_index=7;
    
    while position < image_size
    {
        //TODO test new with  auto vectorization
            if vrun_iters[position%image_header.width].next()==None{

            
            let mut is_backref=false;
            for i in 0..back_ref_lookup.len()
            {
                
                    let ref_pos=position.saturating_sub(back_ref_lookup[i]);
                    if input_bytes[position]==input_bytes[ref_pos]&&input_bytes[position+1]==input_bytes[ref_pos+1]&&input_bytes[position+2]==input_bytes[ref_pos+2]&&position>0
                    {
                        data_prefixes.add_symbolu8(i as u8+12);
                        //data.add_symbolusize(i, SC_BACK_REF);
                        is_backref=true;
                        #[cfg(debug_assertions)]
                        {back_ref_amount+=1;}
                        break;
                    }
                
            }

            if !is_backref
            {
                let mut list_of_color_diffs=[0;3];
                /*for ((el,curr_col),prev_col) in list_of_color_diffs.iter_mut().zip(input_bytes[position..].iter()).zip(input_bytes[prev_position..].iter())
                {
                    *el=*curr_col as i16-*prev_col as i16;
                }*/
                list_of_color_diffs[0]=input_bytes[position] as i16-input_bytes[prev_position] as i16;
                list_of_color_diffs[1]=input_bytes[position+1] as i16-input_bytes[prev_position+1] as i16;
                list_of_color_diffs[2]=input_bytes[position+2] as i16-input_bytes[prev_position+2] as i16;
                if position>=channels*image_header.width
                {
                    list_of_color_diffs[0]=input_bytes[position] as i16-(input_bytes[position-channels*image_header.width] as i16+input_bytes[prev_position] as i16)/2;
                    list_of_color_diffs[1]=input_bytes[position+1] as i16-(input_bytes[position-channels*image_header.width+1] as i16+input_bytes[prev_position+1] as i16)/2;
                    list_of_color_diffs[2]=input_bytes[position+2] as i16-(input_bytes[position-channels*image_header.width+2] as i16+input_bytes[prev_position+2] as i16)/2;
                }

                /*let mut key=(list_of_color_diffs[0] as u8,list_of_color_diffs[1] as u8,list_of_color_diffs[2] as u8);
                    
                if let Some(amount)=most_used_lumadiff.get_mut(&mut key)
                {
                    *amount+=1;
                }
                else
                {
                    most_used_lumadiff.insert(key, 1);
                }*/
                if position>0 &&list_of_color_diffs.iter().all(|&x| x >= -3 &&x<=3)
                {
                    data_prefixes.add_symbolu8(PREFIX_SMALL_DIFF);
                    #[cfg(debug_assertions)]
                    {amount_of_diffs+=1;}
                    
                    data_small_diff1.add_symbolu16((3+list_of_color_diffs[0]) as u16+7*(3+list_of_color_diffs[1]) as u16+49*(3+list_of_color_diffs[2]) as u16);
                }
                else
                {


                    let mut is_luma2=false;
                    let mut list_of_color_diffs=[0;3];
                        
                        if let Some(ref_pos)=position.checked_sub(channels*image_header.width)
                        {
                        
                            //green_diff
                            
                            list_of_color_diffs[1]=input_bytes[position+1].wrapping_sub(((input_bytes[ref_pos+1] as u16 + input_bytes[prev_position+1] as u16) /2) as u8);
                            
                        
                            //red_diff
                            list_of_color_diffs[0]=input_bytes[position].wrapping_sub(((input_bytes[ref_pos] as u16 + input_bytes[prev_position] as u16) /2) as u8);
                            //blue_diff
                            list_of_color_diffs[2]=input_bytes[position+2].wrapping_sub(((input_bytes[ref_pos+2] as u16 + input_bytes[prev_position+2] as u16) /2) as u8);
                            list_of_color_diffs[0]=list_of_color_diffs[0].wrapping_sub(list_of_color_diffs[1]);
                            list_of_color_diffs[2]=list_of_color_diffs[2].wrapping_sub(list_of_color_diffs[1]);


                            //or take most occurred result instead of first result when adding from list of backrefs. 
                            //use run type(s) code stream
                            
                            if position>0&&
                            (list_of_color_diffs[1]>=224||list_of_color_diffs[1]<32)&&
                            ((list_of_color_diffs[0]>=240 || list_of_color_diffs[0]<16) )&&
                            ((list_of_color_diffs[2]>=240 || list_of_color_diffs[2]<16))
                            {



                                data_prefixes.add_symbolu8(PREFIX_COLOR_LUMA2);
                                #[cfg(debug_assertions)]
                                {luma_occurences2+=1;}

                                data_luma_base_diff2.add_symbolu8((list_of_color_diffs[1].wrapping_add(32)) as u8);
                                data_luma_other_diff2.add_symbolu8((list_of_color_diffs[0].wrapping_add(16)) as u8);
                                data_luma_other_diffb2.add_symbolu8((list_of_color_diffs[2].wrapping_add(16)) as u8);
                                is_luma2=true;
                            }
                        
                        }
                    if !is_luma2
                    {
                        let mut is_luma=false;
                        for i in 0..rel_ref_lookup.len()
                        {
                            
                            if let Some(ref_pos)=position.checked_sub(rel_ref_lookup[i])
                            {
                            
                                //green_diff
                                list_of_color_diffs[1]=input_bytes[position+1].wrapping_sub(input_bytes[ref_pos+1]);
                                
                            
                                //red_diff
                                list_of_color_diffs[0]=input_bytes[position].wrapping_sub(input_bytes[ref_pos]).wrapping_sub(list_of_color_diffs[1]);
                                //blue_diff
                                list_of_color_diffs[2]=input_bytes[position+2].wrapping_sub(input_bytes[ref_pos+2]).wrapping_sub(list_of_color_diffs[1]);


                                //or take most occurred result instead of first result when adding from list of backrefs. 
                                //use run type(s) code stream
                                
                                if position>0&&
                                (list_of_color_diffs[1]>=224||list_of_color_diffs[1]<32)&&
                                ((list_of_color_diffs[0]>=240 || list_of_color_diffs[0]<16))&&
                                ((list_of_color_diffs[2]>=240 || list_of_color_diffs[2]<16))
                                {



                                    data_prefixes.add_symbolu8(PREFIX_COLOR_LUMA);
                                    #[cfg(debug_assertions)]
                                    {luma_occurences+=1;}

                                    data_luma_back_ref.add_symbolusize(i);

                                    data_luma_base_diff.add_symbolu8((list_of_color_diffs[1].wrapping_add(32)) as u8);
                                    data_luma_other_diff.add_symbolu8((list_of_color_diffs[0].wrapping_add(16)) as u8);
                                    data_luma_other_diff.add_symbolu8((list_of_color_diffs[2].wrapping_add(16)) as u8);                                    
                                    if position==468279
                                    {
                                            dbg!(list_of_color_diffs[1].wrapping_add(32));
                                            dbg!(list_of_color_diffs[0].wrapping_add(16));
                                            dbg!(list_of_color_diffs[2].wrapping_add(16));
                                        dbg!(i);
                                        dbg!(input_bytes[position+2]);
                                        dbg!(input_bytes[prev_position+2]);
                                        dbg!(ref_pos);
                                    }
                                    is_luma=true;
                                    break;
                                }
                            
                            }
                        }
                        //write rgb
                        //TODO r,g,b as x,y,z of discrete cube
                        //discrete exact distance can only be referring to 3 points(1 on r=g=b axis).
                        //close points have similar distance + same ref, or same distance + different ref
                        //alt: use distance between 2 pixels +8 directions for diffing.(can still use avg logic between pixels)
                        //TODO  MERGE algo's into single prefix
                        if is_luma==false
                        {
                            data_prefixes.add_symbolu8(PREFIX_RGB);
                            #[cfg(debug_assertions)]
                            {rgb_cntr+=1;}
                            let r_code=input_bytes[position].wrapping_sub(if position ==0{0}else{input_bytes[prev_position]});
                            let g_code=input_bytes[position+1].wrapping_sub(if position ==0{0}else{input_bytes[prev_position+1]}).wrapping_sub(r_code);
                            let b_code=input_bytes[position+2].wrapping_sub(if position ==0{0}else{input_bytes[prev_position+2]}).wrapping_sub(r_code);

                            data_rgb.add_symbolu16((r_code/32) as u16 +(((g_code as u16)/32)*8)+(((b_code as u16)/32) )*64);
                            data_rgb2.add_symbolu8(r_code%32);
                            data_rgb2.add_symbolu8(g_code%32);
                            data_rgb2.add_symbolu8(b_code%32);
                                /*data.add_symbolu8(((input_bytes[position] as i16).wrapping_sub((if position >= channels*image_header.width
                                    {
                                      input_bytes[position-channels*image_header.width]
                                    }
                                    else
                                    { 
                                      if position>0{input_bytes[prev_position]}else{0}
                                    } as i16+(if position>0{input_bytes[prev_position]}else{0}) as i16)/2)) as u8, SC_RGB);
                                
                                data.add_symbolu8(((input_bytes[position+1] as i16).wrapping_sub((if position >= channels*image_header.width
                                    { 
                                      input_bytes[position-channels*image_header.width+1]
                                    }
                                    else
                                    { 
                                      if position>0{input_bytes[prev_position+1]}else{0}
                                    } as i16+(if position>0{input_bytes[prev_position+1]}else{0}) as i16)/2)) as u8, SC_RGB);
                                data.add_symbolu8(((input_bytes[position+2] as i16).wrapping_sub((if position >= channels*image_header.width
                                    { 
                                      input_bytes[position-channels*image_header.width+2]
                                    }
                                    else
                                    { 
                                      if position>0{input_bytes[prev_position+2]}else{0}
                                    } as i16+(if position>0{input_bytes[prev_position+2]}else{0}) as i16)/2)) as u8, SC_RGB);*/

                        }
                    }
                }
            }

            let mut offset=channels;
            /*if position> channels*image_header.width&&input_bytes[position].abs_diff(input_bytes[position-channels])>=input_bytes[position].abs_diff(input_bytes[position-channels*image_header.width])
            {
                offset*=image_header.width;
            }*/
            let mut run_length=0;
                //let mut offset=channels;
                let mut run_loop_position=position+offset;
                
                //let mut run_length=input_bytes[position+channels..].chunks_exact(8).take_while(|&run_chunk|*run_chunk==input_bytes[position..position+3]).count();
                
                /*while run_loop_position<image_size&&
                    input_bytes[run_loop_position..run_loop_position+2].eq(&input_bytes[position..position+2])                {
                    run_length+=1;

                    run_loop_position+=offset;
                }*/
                run_length=input_bytes[position+channels..].chunks_exact(3).take_while(|&run_chunk|*run_chunk==input_bytes[position..position+3]).count();

                if run_length > 1
                {
                    
                    //run_count_red+=red_run_length;
                    #[cfg(debug_assertions)]
                    {pixel_run_amount+=run_length;}
                    //let prefix_offset;
                    if offset==channels
                    {
                        //prefix_offset=5;
                    position+=run_length*channels;
                    }
                    else {
                        vrun_iters[position%image_header.width]=(0..run_length).rev();
                       // prefix_offset=13;
                    }
                    run_length = run_length - 2;

                    loop
                    {
                        data_prefixes.add_symbolusize(run_length%8+4);
                        #[cfg(debug_assertions)]
                        {run_cntr+=1;}

                        #[cfg(debug_assertions)]
                        {run_occurrences[run_length%8]+=1;}
                        if run_length <8
                        {
                            break;
                        }
                        run_length = run_length /8;
                        
                    }
                }
                /*else
                {
                    let mut run_length=0;
                    //let mut offset=channels;
                    let mut run_loop_position=position+channels*image_header.width;
                    
                    //let mut run_length=input_bytes[position+channels..].chunks_exact(8).take_while(|&run_chunk|*run_chunk==input_bytes[position..position+3]).count();
                    while run_loop_position<image_size&&
                        input_bytes[run_loop_position]==input_bytes[position]&&
                        input_bytes[run_loop_position+1]==input_bytes[position+1]&&
                        input_bytes[run_loop_position+2]==input_bytes[position+2]
                    {
                        run_length+=1;
                        run_loop_position+=channels*image_header.width;
                    }
                    if run_length > 1
                    {
                        //run_count_red+=red_run_length;
                        #[cfg(debug_assertions)]
                        {pixel_run_amount+=run_length;}
                        //position+=run_length*channels*image_header.width;
                        vrun_iters[position%image_header.width]=(0..run_length).rev();
                        run_length = run_length - 2;
                        loop
                        {
                            data.add_symbolusize(run_length%8+13, SC_PREFIXES);
                            #[cfg(debug_assertions)]
                            {run_cntr+=1;}
    
                            #[cfg(debug_assertions)]
                            {run_occurrences[run_length%8]+=1;}
                            if run_length <8
                            {
                                break;
                            }
                            run_length = run_length /8;
                            
                        }
                    }
                }*/
            }
                
            
          
        
        prev_position=position;
        position+=channels;
    }
    //dbg!(data.data_vec.len());

    //let mut bitwriter=Bitwriter::new(output_writer);
    //let mut output_vec : Vec<u8>=Vec::new();

    let mut enc_rgb_data : Vec<u8>=Vec::new();
    let mut enc_prefixes_data : Vec<u8>=Vec::new();
    let mut enc_luma_base_diff_data : Vec<u8>=Vec::new();
    let mut enc_luma_other_diff_data : Vec<u8>=Vec::new();
    let mut enc_luma_back_ref_data : Vec<u8>=Vec::new();
    let mut enc_small_diff1_data : Vec<u8>=Vec::new();
    let mut enc_luma_base_diff2_data : Vec<u8>=Vec::new();
    let mut enc_luma_other_diff2_data : Vec<u8>=Vec::new();
    let mut enc_luma_other_diffb2_data : Vec<u8>=Vec::new();
    let mut enc_rgb2_data : Vec<u8>=Vec::new();
    //TODO opti for small: store amount of symbols for output
    data_rgb.to_encoded_output(&mut Bitwriter::new(&mut enc_rgb_data))?;
    data_prefixes.to_encoded_output(&mut Bitwriter::new(&mut enc_prefixes_data))?;
    data_luma_base_diff.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_base_diff_data))?;
    data_luma_other_diff.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_other_diff_data))?;
    data_luma_back_ref.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_back_ref_data))?;
    data_small_diff1.to_encoded_output(&mut Bitwriter::new(&mut enc_small_diff1_data))?;
    data_luma_base_diff2.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_base_diff2_data))?;
    data_luma_other_diff2.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_other_diff2_data))?;
    data_luma_other_diffb2.to_encoded_output(&mut Bitwriter::new(&mut enc_luma_other_diffb2_data))?;
    data_rgb2.to_encoded_output(&mut Bitwriter::new(&mut enc_rgb2_data))?;
    //length should be < u32::MAX based on image dimensions, TODO can this be made smaller?
    output_writer.write_all(&(enc_rgb_data.len() as u32).to_be_bytes())?;
    //dbg!(&enc_rgb_data[0..1000]);
    output_writer.write_all(&enc_rgb_data)?;
    output_writer.write_all(&(enc_prefixes_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_prefixes_data)?;
    output_writer.write_all(&(enc_luma_base_diff_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_base_diff_data)?;
    output_writer.write_all(&(enc_luma_other_diff_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_other_diff_data)?;
    output_writer.write_all(&(enc_luma_back_ref_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_back_ref_data)?;
    output_writer.write_all(&(enc_small_diff1_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_small_diff1_data)?;
    output_writer.write_all(&(enc_luma_base_diff2_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_base_diff2_data)?;
    output_writer.write_all(&(enc_luma_other_diff2_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_other_diff2_data)?;
    output_writer.write_all(&(enc_luma_other_diffb2_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_luma_other_diffb2_data)?;
    output_writer.write_all(&(enc_rgb2_data.len() as u32).to_be_bytes())?;
    output_writer.write_all(&enc_rgb2_data)?;
    //write cache to writer?
    //let cache=bitwriter.cache.to_be_bytes();
    //bitwriter.writer.write_all(&cache)?;
    //data.data_vec.extend_from_slice(&cache[..]);
    //dbg!(data.data_vec.len());
    //handle in hfe
    //output_writer.write_all(&data.data_vec)?;

    //}
    #[cfg(debug_assertions)]
    {
        dbg!(rgb_cntr);
        dbg!(run_cntr);
        dbg!(luma_occurences);
        dbg!(luma_occurences2);
        dbg!(pixel_run_amount);
        dbg!(run_occurrences);
        dbg!(amount_of_diffs);
        dbg!(back_ref_amount);
    }
    
    

    //println!("{}", now.elapsed().as_millis());
    Ok(())
}

/*pub struct ImageBytes {
    pub image: Image,
    pub bytes: Vec<u8>,
}*/
//read from file or ...
pub fn decode(
    image_data: std::rc::Rc<Vec<u8>>,
    channels_out: u8,
    output_vec: &mut Vec<u8>,
) -> std::io::Result<Image> {
    
    //image_reader.read(&mut [0; 4])?;
    //let mut buf = [0; 4];

    //image_reader.read(&mut buf)?;
    let width : u32;
    let mut buf : [u8; 4]=[0; 4];
    buf.clone_from_slice(&image_data[4..8]);
    width=u32::from_be_bytes(buf);
    #[cfg(debug_assertions)]
    dbg!(width);
    buf.clone_from_slice(&image_data[8..12]);
    let height = u32::from_be_bytes(buf);
    #[cfg(debug_assertions)]
    dbg!( height);
    
    let height = height as usize;
    let mut channels_buf = [0; 1];
    //image_reader.read(&mut channels_buf)?;
    channels_buf.clone_from_slice(&image_data[12..13]);
    let channels = u8::from_be_bytes(channels_buf) as usize;
    #[cfg(debug_assertions)]
    dbg!( channels);
    //let bitreader = BitReader::<R, BigEndian>::new(reader);
    let image_size = width as usize * height as usize * channels;
    let image =Image::new(
        width as usize,
        height,
         channels as u8,
    );
    let mut position = 0;
    #[cfg(debug_assertions)]
    dbg!(image_size);
    *output_vec = Vec::with_capacity(image_size);
    unsafe
    {
        output_vec.set_len(image_size);
    }
    #[cfg(debug_assertions)]
    dbg!(output_vec.len());
    //TODO push output to Vec
    //let mut bitreader = Bitreader::new(image_reader);

    buf.clone_from_slice(&image_data[13..17]);
    let rgb_offset=17;
    
    let mut prefix_offset = 17+u32::from_be_bytes(buf);    
    buf.clone_from_slice(&image_data[prefix_offset as usize..(prefix_offset+4) as usize]);
    prefix_offset+=4;

    let mut luma_base_diff_offset=prefix_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_base_diff_offset as usize..(luma_base_diff_offset+4) as usize]);
    luma_base_diff_offset+=4;
    
    let mut luma_other_diff_offset=luma_base_diff_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_other_diff_offset as usize..(luma_other_diff_offset+4) as usize]);
    luma_other_diff_offset+=4;
    
    let mut luma_backref_offset=luma_other_diff_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_backref_offset as usize..(luma_backref_offset+4) as usize]);
    luma_backref_offset+=4;
    
    let mut small_diff_offset=luma_backref_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[small_diff_offset as usize..(small_diff_offset+4) as usize]);
    small_diff_offset+=4;
    
    let mut luma_base_diff2_offset=small_diff_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_base_diff2_offset as usize..(luma_base_diff2_offset+4) as usize]);
    luma_base_diff2_offset+=4;
    
    let mut luma_other_diff2_offset=luma_base_diff2_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_other_diff2_offset as usize..(luma_other_diff2_offset+4) as usize]);
    luma_other_diff2_offset+=4;
    
    let mut luma_other_diffb2_offset=luma_other_diff2_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[luma_other_diffb2_offset as usize..(luma_other_diffb2_offset+4) as usize]);
    luma_other_diffb2_offset+=4;
    
    let mut rgb2_offset=luma_other_diffb2_offset+u32::from_be_bytes(buf);
    buf.clone_from_slice(&image_data[rgb2_offset as usize..(rgb2_offset+4) as usize]);
    rgb2_offset+=4;

    //dbg!(rgb_offset);
    //dbg!(prefix_offset);
    //dbg!(&image_data.clone()[17..1017 ]);
    let rgb_data = hfe::DecodeInput::new(&image_data.clone()[rgb_offset as usize..(prefix_offset-4) as usize]).read_header_into_tree(512)?;
    let prefix_data=hfe::DecodeInput::new(&image_data.clone()[prefix_offset as usize..(luma_base_diff_offset-4) as usize]).read_header_into_tree(17)?;
    let luma_base_diff_data=hfe::DecodeInput::new(&image_data.clone()[luma_base_diff_offset as usize..(luma_other_diff_offset-4) as usize]).read_header_into_tree(64)?;
    let luma_other_diff_data=hfe::DecodeInput::new(&image_data.clone()[luma_other_diff_offset as usize..(luma_backref_offset-4) as usize]).read_header_into_tree(32)?;
    let luma_backref_data=hfe::DecodeInput::new(&image_data.clone()[luma_backref_offset as usize..(small_diff_offset-4) as usize]).read_header_into_tree(11)?;
    let small_diff_data=hfe::DecodeInput::new(&image_data.clone()[small_diff_offset as usize..(luma_base_diff2_offset-4) as usize]).read_header_into_tree(343)?;
    let luma_base_diff2_data=hfe::DecodeInput::new(&image_data.clone()[luma_base_diff2_offset as usize..(luma_other_diff2_offset-4) as usize]).read_header_into_tree(64)?;
    let luma_other_diff2_data=hfe::DecodeInput::new(&image_data.clone()[luma_other_diff2_offset as usize..(luma_other_diffb2_offset-4) as usize]).read_header_into_tree(32)?;
    let luma_other_diffb2_data=hfe::DecodeInput::new(&image_data.clone()[luma_other_diffb2_offset as usize..(rgb2_offset-4) as usize]).read_header_into_tree(32)?;
    let rgb2_data=hfe::DecodeInput::new(&image_data.clone()[rgb2_offset as usize..]).read_header_into_tree(32)?;

    //let headertime = Instant::now();
    //let mut decoder=  hfe::DecodeInput::new(Bitreader::new(image_reader));

    
    /*let back_ref_lookup_table:[usize;5]=[channels,channels*image.width,channels*(image.width-1),2*channels,2*channels*image.width];
    for i in 0..back_ref_lookup.symbol_lookup.len()
    {
        back_ref_lookup.symbol_lookup[i].symbol=back_ref_lookup_table[back_ref_lookup.symbol_lookup[i].symbol];
    }*/
    //decoder.read_header_into_tree(&mut adj_block_lookup).unwrap();

    let rel_ref_lookup:[usize;11]=[channels,channels*image.width,channels*(image.width-1),channels*(image.width-3),3*channels,
    channels*(3*image.width-1),3*channels*image.width,channels*(3*image.width+1),channels*(image.width+3),channels*3*(image.width+1),channels*3*(image.width-1)];
    let back_ref_lookup:[usize;5]=[channels,channels*image.width,channels*(image.width-1),2*channels,2*channels*image.width];
    /*for i in 0..luma_backref_lookup.symbol_lookup.len()
    {
        luma_backref_lookup.symbol_lookup[i].symbol=rel_ref_lookup[luma_backref_lookup.symbol_lookup[i].symbol];
    }*/
    let mut prefix_data_offset=0;
    let mut prefix1=prefix_data[prefix_data_offset] as u8;
    prefix_data_offset+=1;

    
    let mut luma_base_diff2_data_offset=0;
    let mut luma_other_diff2_data_offset=0;
    let mut luma_other_diffb2_data_offset=0;
    let mut small_diff_data_offset=0;
    let mut luma_backref_data_offset=0;
    let mut luma_base_diff_data_offset=0;
    let mut luma_other_diff_data_offset=0;
    let mut rgb_data_offset=0;
    let mut rgb2_data_offset=0;

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
    let run_prefixes=[PREFIX_RUN_1,PREFIX_RUN_2,PREFIX_RUN_3,PREFIX_RUN_4,PREFIX_RUN_5,PREFIX_RUN_6,PREFIX_RUN_7,PREFIX_RUN_8];
    //#[cfg(debug_assertions)]
    //let mut loopindex=0;

    //println!("time:{}:",headertime.elapsed().as_millis());
    while position<image_size 
    {

        match prefix1
        {
            
            PREFIX_COLOR_LUMA2=>
            {
                let prev_luma_base_diff=luma_base_diff2_data[luma_base_diff2_data_offset].wrapping_sub(32) as u8;
                luma_base_diff2_data_offset+=1;

                output_vec[position+1]=prev_luma_base_diff.wrapping_add(((output_vec[prev_pos+1] as u16 + output_vec[position-channels*image.width+1] as u16)/2) as u8);
                
                output_vec[position]=(luma_other_diff2_data[luma_other_diff2_data_offset] as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(((output_vec[prev_pos] as u16 + output_vec[position-channels*image.width] as u16)/2) as u8));
                luma_other_diff2_data_offset+=1;
                output_vec[position+2]=(luma_other_diffb2_data[luma_other_diffb2_data_offset] as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(((output_vec[prev_pos+2] as u16 + output_vec[position-channels*image.width+2] as u16)/2) as u8));
                luma_other_diffb2_data_offset+=1;
            }
            PREFIX_SMALL_DIFF=>
            {
                let small_diff=small_diff_data[small_diff_data_offset] as i16;
                small_diff_data_offset+=1;
                let red_diff=small_diff%7;
                //small_diff=(small_diff-red_diff)/7;
                let green_diff=((small_diff-red_diff)/7)%7;
                //small_diff=;
                let blue_diff=(((small_diff-red_diff)/7)-green_diff)/7;
                let ref_red;
                let ref_green;
                let ref_blue;
                if position>=channels*image.width
                {
                    let v_pos=position-channels*image.width;
                    ref_red=(output_vec[v_pos] as i16+output_vec[prev_pos] as i16)/2;
                    ref_green=(output_vec[v_pos+1] as i16+output_vec[prev_pos+1] as i16)/2;
                    ref_blue=(output_vec[v_pos+2] as i16+output_vec[prev_pos+2] as i16)/2;
                }
                else
                {
                    ref_red=output_vec[prev_pos] as i16;
                    ref_green=output_vec[prev_pos+1] as i16;
                    ref_blue=output_vec[prev_pos+2] as i16;
                };

                output_vec[position]=(red_diff-3 +ref_red) as u8;
                output_vec[position+1]=(green_diff-3 +ref_green) as u8;
                output_vec[position+2]=(blue_diff-3 +ref_blue) as u8;

            }
            PREFIX_COLOR_LUMA=>
            {
                let backref = rel_ref_lookup[luma_backref_data[luma_backref_data_offset] as usize];

                luma_backref_data_offset+=1;
                
                let prev_luma_base_diff=luma_base_diff_data[luma_base_diff_data_offset].wrapping_sub(32) as u8;
                luma_base_diff_data_offset+=1;

                output_vec[position+1]=prev_luma_base_diff.wrapping_add(output_vec[position-backref+1]);
                
                output_vec[position]=(luma_other_diff_data[luma_other_diff_data_offset] as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff).wrapping_add(output_vec[position-backref]);
                luma_other_diff_data_offset+=1;
                
                output_vec[position+2]=(luma_other_diff_data[luma_other_diff_data_offset] as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff).wrapping_add(output_vec[position-backref+2]);
                luma_other_diff_data_offset+=1;
            }
            PREFIX_BACK_REF1=>
            {
                let backref=back_ref_lookup[0];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
                //output_vec.copy_within(position..position+3, position-back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize]);
            }
            PREFIX_BACK_REF2=>
            {
                let backref=back_ref_lookup[1];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
                //output_vec.copy_within(position..position+3, position-back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize]);
            }
            PREFIX_BACK_REF3=>
            {
                let backref=back_ref_lookup[2];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
                //output_vec.copy_within(position..position+3, position-back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize]);
            }
            PREFIX_BACK_REF4=>
            {
                let backref=back_ref_lookup[3];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
                //output_vec.copy_within(position..position+3, position-back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize]);
            }
            PREFIX_BACK_REF5=>
            {
                let backref=back_ref_lookup[4];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
                //output_vec.copy_within(position..position+3, position-back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize]);
            }
            PREFIX_RGB=>
            {
                
                    let mut r_code=rgb_data[rgb_data_offset];
                    
                    rgb_data_offset+=1;
                    let b_code=r_code/64*32;
                    r_code=r_code%64;
                    let g_code=r_code/8*32;
                    r_code=32*(r_code%8)+rgb2_data[rgb2_data_offset];
                    rgb2_data_offset+=1;
                    output_vec[position]=(r_code as u8).wrapping_add(if position ==0{0}else {output_vec[prev_pos]}) as u8;
                    output_vec[position+1]=((g_code+rgb2_data[rgb2_data_offset]).wrapping_add(r_code) as u8).wrapping_add(if position ==0{0}else {output_vec[prev_pos+1]});
                    rgb2_data_offset+=1;
                    output_vec[position+2]=((b_code+rgb2_data[rgb2_data_offset]).wrapping_add(r_code) as u8).wrapping_add(if position ==0{0}else {output_vec[prev_pos+2]});
                    rgb2_data_offset+=1;

            }
            
            _=>
            {
                eprintln!("error unkown token");
            }

        }
        #[cfg(debug_assertions)]
        {
            debug_assert!(dump[position]==output_vec[position],"expected: {}, output: {} at position {}",dump[position],output_vec[position],position);
            debug_assert!(dump[position+1]==output_vec[position+1],"expected: {}, output: {} at position {}",dump[position+1],output_vec[position+1],position+1);
            debug_assert!(dump[position+2]==output_vec[position+2],"expected: {}, output: {} at position {}",dump[position+2],output_vec[position+2],position+2);
        }
        prev_pos=position;
        position+=3;

        prefix1=prefix_data[prefix_data_offset] as u8;
        prefix_data_offset+=1;
        //dbg!("test1");
        if run_prefixes.iter().any(|&x| x == prefix1)
        {

            let mut temp_curr_runcount: u8=0;
            let mut run_length:usize=0;
            while let Some(&prefix_result)=run_prefixes.iter().find(|&&x| x == prefix1)
            {
                //run lengths
                run_length+=prefix_result as usize-4 << temp_curr_runcount;
                temp_curr_runcount += 3;
                prefix1=prefix_data[prefix_data_offset] as u8;
                prefix_data_offset+=1;
            }

            run_length += 2;
            //if position==3
            {
               // dbg!(run_length);
            }
            for i in 0..run_length
            {
                output_vec.copy_within(prev_pos..=prev_pos+2, position+i*channels);
            }
            #[cfg(debug_assertions)]
            for (i,&el) in (&output_vec[position..position+run_length*channels]).iter().enumerate()
            {
                debug_assert!(dump[position+i]==el,"expected: {}, output: {} at position {},based on pos {},runlength {} ",dump[position+i],el,position+i,prev_pos,run_length);
            }
            position+=run_length*channels;
        }
        //temp_time+=headertime.elapsed().as_nanos();
                
    
    }
    //println!("temp_time:{}: ",temp_time);
    Ok(image)
}
