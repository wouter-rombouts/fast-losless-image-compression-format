const NICE: &[u8] = "nice".as_bytes();
//use itertools::Itertools;

use crate::image::Image;
//use crate::pixel::Pixel;
//use std::cmp::Reverse;
//use std::collections::HashMap;
use std::{io, fs};
//use std::{time::Instant, *};
use crate::hfe::{EncodedOutput, self, SymbolstreamLookup};

use crate::{bitreader::Bitreader, bitwriter::Bitwriter};
//pub(crate) const PREFIX_RUN: u8 = 2;

//pub(crate) const PREFIX_RUN: u8 = 0;
pub(crate) const PREFIX_RGB: u8 = 1;
pub(crate) const PREFIX_COLOR_LUMA: u8 = 2;
pub(crate) const PREFIX_SMALL_DIFF: u8 = 3;
pub(crate) const PREFIX_COLOR_LUMA2: u8 = 4;
pub(crate) const PREFIX_BACK_REF: u8 = 0;
pub(crate) const PREFIX_RUN_1: u8 = 5;
pub(crate) const PREFIX_RUN_2: u8 = 6;
pub(crate) const PREFIX_RUN_3: u8 = 7;
pub(crate) const PREFIX_RUN_4: u8 = 8;
pub(crate) const PREFIX_RUN_5: u8 = 9;
pub(crate) const PREFIX_RUN_6: u8 = 10;
pub(crate) const PREFIX_RUN_7: u8 = 11;
pub(crate) const PREFIX_RUN_8: u8 = 12;
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
pub(crate) const SC_BACK_REF: u8 = 9;
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
    let mut data =EncodedOutput::new( image_size );

    //initialize all output streams
    //0==PREFIX_RGB
    data.add_output_type(256);
    //1==SC_PREFIXES
    data.add_output_type(13);
    //2==SC_RUN_LENGTHS
    //data.add_output_type(8);
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
    //9==SC_SMALL_DIFF2
    //data.add_output_type(7);
    //10==SC_SMALL_DIFF3
    //data.add_output_type(7);
    //9==SC_LUMA_OTHER_DIFFB2
    data.add_output_type(32);
    //10==SC_BACK_REF
    data.add_output_type(11);

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
    let mut run_occurrences=[0;8];
    //TODO auto-vectorization, after backref?

    let rel_ref_lookup:[usize;11]=[channels,channels*image_header.width,channels*(image_header.width-1),channels*(image_header.width-3),3*channels,
    channels*(3*image_header.width-1),3*channels*image_header.width,channels*(3*image_header.width+1),channels*(image_header.width+3),channels*3*(image_header.width+1),channels*3*(image_header.width-1)];

    
    let back_ref_lookup:[usize;5]=[channels,channels*image_header.width,channels*(image_header.width-1),2*channels,2*channels*image_header.width];
    

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
        //position=loop_index*channels;
        /*pos_cache_index=(pos_cache_index+1)%8;
        if pos_cache_index==0
        {
            let mut temp_pos_cache_list:[[[u8; 3]; 8];8]=[[[0,0,0];8];8];
            //initial list
            for i in 0..8
            {
                temp_pos_cache_list[0][i]=[input_bytes[position+channels*i],input_bytes[position+channels*i+1],input_bytes[position+channels*i+2]];
            }
            //rotations
            for i in 1..8
            {
                for j in 0..8
                {
                    temp_pos_cache_list[i][j]=temp_pos_cache_list[0][(j+i)%8];
                } 
            }

            //sort by sum of colors?
            temp_pos_cache_list.sort();
            //take last column
            for i in 0..8
            {
                positions_cache[i]=temp_pos_cache_list[i][7];
            }
            
            //output index of correct begin
        }*/
        
            let mut is_backref=false;
            for i in 0..back_ref_lookup.len()
            {
                
                if let Some(ref_pos)=position.checked_sub(back_ref_lookup[i])
                {
                    if input_bytes[position]==input_bytes[ref_pos]&&input_bytes[position+1]==input_bytes[ref_pos+1]&&input_bytes[position+2]==input_bytes[ref_pos+2]
                    {
                        data.add_symbolu8(PREFIX_BACK_REF, SC_PREFIXES);
                        data.add_symbolusize(i, SC_BACK_REF);
                        is_backref=true;
                        break;
                    }
                
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
                    data.add_symbolu8(PREFIX_SMALL_DIFF, SC_PREFIXES);
                    #[cfg(debug_assertions)]
                    {amount_of_diffs+=1;}
                    
                    //if 
                    let mut code=(3+list_of_color_diffs[0]) as u16;
                    code+=7*(3+list_of_color_diffs[1]) as u16;
                    code+=49*(3+list_of_color_diffs[2]) as u16;
                    data.add_symbolu16(code, SC_SMALL_DIFF1);
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



                                data.add_symbolu8(PREFIX_COLOR_LUMA2, SC_PREFIXES);
                                #[cfg(debug_assertions)]
                                {luma_occurences2+=1;}

                                data.add_symbolu8((list_of_color_diffs[1].wrapping_add(32)) as u8, SC_LUMA_BASE_DIFF2);
                                data.add_symbolu8((list_of_color_diffs[0].wrapping_add(16)) as u8, SC_LUMA_OTHER_DIFF2);
                                data.add_symbolu8((list_of_color_diffs[2].wrapping_add(16)) as u8, SC_LUMA_OTHER_DIFFB2);
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
                                list_of_color_diffs[0]=input_bytes[position].wrapping_sub(input_bytes[ref_pos]);
                                //blue_diff
                                list_of_color_diffs[2]=input_bytes[position+2].wrapping_sub(input_bytes[ref_pos+2]);
                                list_of_color_diffs[0]=list_of_color_diffs[0].wrapping_sub(list_of_color_diffs[1]);
                                list_of_color_diffs[2]=list_of_color_diffs[2].wrapping_sub(list_of_color_diffs[1]);


                                //or take most occurred result instead of first result when adding from list of backrefs. 
                                //use run type(s) code stream
                                
                                if position>0&&
                                (list_of_color_diffs[1]>=224||list_of_color_diffs[1]<32)&&
                                ((list_of_color_diffs[0]>=240 || list_of_color_diffs[0]<16))&&
                                ((list_of_color_diffs[2]>=240 || list_of_color_diffs[2]<16))
                                {



                                    data.add_symbolu8(PREFIX_COLOR_LUMA, SC_PREFIXES);
                                    #[cfg(debug_assertions)]
                                    {luma_occurences+=1;}

                                    data.add_symbolusize(i, SC_LUMA_BACK_REF);

                                    data.add_symbolu8((list_of_color_diffs[1].wrapping_add(32)) as u8, SC_LUMA_BASE_DIFF);
                                        data.add_symbolu8((list_of_color_diffs[0].wrapping_add(16)) as u8, SC_LUMA_OTHER_DIFF);
                                        data.add_symbolu8((list_of_color_diffs[2].wrapping_add(16)) as u8, SC_LUMA_OTHER_DIFF);
                                    is_luma=true;
                                    break;
                                }
                            
                            }
                        }
                        //write rgb
                        if is_luma==false
                        {
                            data.add_symbolu8(PREFIX_RGB, SC_PREFIXES);
                            #[cfg(debug_assertions)]
                            {rgb_cntr+=1;}
                                let mut red_code=input_bytes[position].wrapping_sub(if position>0{input_bytes[prev_position]}else{0});
                                if position>=channels*image_header.width
                                {
                                    red_code=((input_bytes[position] as i16).wrapping_sub((input_bytes[position-channels*image_header.width] as i16+input_bytes[prev_position] as i16)/2)) as u8;
                                }
                                data.add_symbolu8(red_code, SC_RGB);
                                
                                let mut green_code=input_bytes[position+1].wrapping_sub(if position>0{input_bytes[prev_position+1]}else{0});
                                if position>=channels*image_header.width
                                {
                                    green_code=((input_bytes[position+1] as i16).wrapping_sub((input_bytes[position+1-channels*image_header.width] as i16+input_bytes[prev_position+1] as i16)/2)) as u8;
                                }
                                data.add_symbolu8(green_code, SC_RGB);
                                let mut blue_code=input_bytes[position+2].wrapping_sub(if position>0{input_bytes[prev_position+2]}else{0});
                                if position>=channels*image_header.width
                                {
                                    blue_code=((input_bytes[position+2] as i16).wrapping_sub((input_bytes[position+2-channels*image_header.width] as i16+input_bytes[prev_position+2] as i16)/2)) as u8;
                                }
                                data.add_symbolu8(blue_code, SC_RGB);

                        }
                    }
                }
            }

            let mut run_length=0;
                //let mut offset=channels;
                let mut run_loop_position=position+channels;
                
                //let mut run_length=input_bytes[position+channels..].chunks_exact(8).take_while(|&run_chunk|*run_chunk==input_bytes[position..position+3]).count();
                while run_loop_position<image_size&&
                    input_bytes[run_loop_position]==input_bytes[position]&&
                    input_bytes[run_loop_position+1]==input_bytes[position+1]&&
                    input_bytes[run_loop_position+2]==input_bytes[position+2]
                {
                    run_length+=1;
                    run_loop_position+=channels;
                }

                if run_length > 0
                {
                    //run_count_red+=red_run_length;
                    #[cfg(debug_assertions)]
                    {pixel_run_amount+=run_length;}
                    position+=run_length*channels;
                    run_length = run_length - 1;
                    loop
                    {
                        data.add_symbolusize(run_length%8+5, SC_PREFIXES);
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
                
            
          
        
        prev_position=position;
        position+=channels;
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
    #[cfg(debug_assertions)]
    {
        dbg!(rgb_cntr);
        dbg!(run_cntr);
        dbg!(luma_occurences);
        dbg!(luma_occurences2);
        dbg!(pixel_run_amount);
        dbg!(run_occurrences);
        dbg!(amount_of_diffs);
    }
    
    
    /*let mut lijstje : Vec<(&(u8,u8,u8),&usize)>=most_used_lumadiff.iter().sorted_by(|a, b|  Reverse(a.1).cmp(&Reverse(b.1))).take(100).collect();
    //lijstje.sort_by(|a, b|  Reverse(a.1).cmp(&Reverse(b.1)));
    for i in 0..lijstje.len()
    {
        println!("{},{},{}",lijstje[i].0.0,lijstje[i].0.1,lijstje[i].0.2);
        println!("amount:{}",lijstje[i].1);
    }
    let s:usize=lijstje.iter().map(|f|f.1).sum();
    dbg!(s);*/
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
    let mut prefix_lookup = SymbolstreamLookup::new(13);
    //2==SC_RUN_LENGTHS
    //let mut runlength_lookup = SymbolstreamLookup::new(8);
    //3==SC_LUMA_BASE_DIFF
    let mut luma_base_diff_lookup = SymbolstreamLookup::new(64);
    //4==SC_LUMA_OTHER_DIFF
    let mut luma_other_diff_lookup = SymbolstreamLookup::new(32);
    //5==SC_LUMA_BACK_REF
    let mut luma_backref_lookup = SymbolstreamLookup::new(11);
    //6==SC_SMALL_DIFF
    let mut small_diff_lookup = SymbolstreamLookup::new(343);
    //7==SC_LUMA_BASE_DIFF2
    let mut luma_base_diff2_lookup = SymbolstreamLookup::new(64);
    //8==SC_LUMA_OTHER_DIFF2
    let mut luma_other_diff2_lookup = SymbolstreamLookup::new(32);
    //9==SC_SMALL_DIFF2
    //let mut small_diff2_lookup = SymbolstreamLookup::new(7);
    //10==SC_SMALL_DIFF3
    //let mut small_diff3_lookup = SymbolstreamLookup::new(7);
    //11==SC_LUMA_OTHER_DIFFB2
    let mut luma_other_diffb2_lookup = SymbolstreamLookup::new(32);
    //12==SC_BACK_REF
    let mut back_ref_lookup = SymbolstreamLookup::new(11);
    
    decoder.read_header_into_tree(&mut rgb_lookup).unwrap();
    decoder.read_header_into_tree(&mut prefix_lookup).unwrap();
    //decoder.read_header_into_tree(&mut runlength_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_backref_lookup).unwrap();
    decoder.read_header_into_tree(&mut small_diff_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_base_diff2_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diff2_lookup).unwrap();
    //decoder.read_header_into_tree(&mut small_diff2_lookup).unwrap();
    //decoder.read_header_into_tree(&mut small_diff3_lookup).unwrap();
    decoder.read_header_into_tree(&mut luma_other_diffb2_lookup).unwrap();
    decoder.read_header_into_tree(&mut back_ref_lookup).unwrap();
    //decoder.read_header_into_tree(&mut adj_block_lookup).unwrap();

    let rel_ref_lookup:[usize;11]=[channels,channels*image.width,channels*(image.width-1),channels*(image.width-3),3*channels,
    channels*(3*image.width-1),3*channels*image.width,channels*(3*image.width+1),channels*(image.width+3),channels*3*(image.width+1),channels*3*(image.width-1)];
    let mut prefix1=decoder.read_next_symbol(&prefix_lookup)? as u8;

    let back_ref_lookup_table:[usize;5]=[channels,channels*image.width,channels*(image.width-1),2*channels,2*channels*image.width];
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
    //TODO BWT for better runlength?
    //bwt for other algo?
    while position<image_size 
    {
                
        match prefix1
        {
            
            PREFIX_COLOR_LUMA2=>
            {
                let prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff2_lookup)?.wrapping_sub(32) as u8;

                output_vec[position+1]=prev_luma_base_diff.wrapping_add(((output_vec[prev_pos+1] as u16 + output_vec[position-channels*image.width+1] as u16)/2) as u8);

                output_vec[position]=(decoder.read_next_symbol(&luma_other_diff2_lookup)? as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(((output_vec[prev_pos] as u16 + output_vec[position-channels*image.width] as u16)/2) as u8));

                output_vec[position+2]=(decoder.read_next_symbol(&luma_other_diffb2_lookup)? as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(((output_vec[prev_pos+2] as u16 + output_vec[position-channels*image.width+2] as u16)/2) as u8));
            }
            PREFIX_SMALL_DIFF=>
            {
                let mut small_diff=decoder.read_next_symbol(&small_diff_lookup)? as i16;
                let red_diff=small_diff%7;
                small_diff=(small_diff-red_diff)/7;
                let green_diff=small_diff%7;
                //small_diff=;
                let blue_diff=(small_diff-green_diff)/7;
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
                let backref = rel_ref_lookup[decoder.read_next_symbol(&luma_backref_lookup)? as usize];
                let prev_luma_base_diff=decoder.read_next_symbol(&luma_base_diff_lookup)?.wrapping_sub(32) as u8;

                output_vec[position+1]=prev_luma_base_diff.wrapping_add(output_vec[position-backref+1]);

                output_vec[position]=(decoder.read_next_symbol(&luma_other_diff_lookup)? as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(output_vec[position-backref]));

                output_vec[position+2]=(decoder.read_next_symbol(&luma_other_diff_lookup)? as u8).wrapping_sub(16).wrapping_add(prev_luma_base_diff.wrapping_add(output_vec[position-backref+2]));
            }
            PREFIX_BACK_REF=>
            {
                let backref=back_ref_lookup_table[decoder.read_next_symbol(&back_ref_lookup)? as usize];
                
                output_vec[position]=output_vec[position-backref];
                output_vec[position+1]=output_vec[position-backref+1];
                output_vec[position+2]=output_vec[position-backref+2];
            }
            PREFIX_RGB=>
            {
                let v_pos=if position>=channels*image.width{position-channels*image.width}else{prev_pos};
                    output_vec[position]=((decoder.read_next_symbol(&rgb_lookup)? as i16).wrapping_add((output_vec[v_pos] as i16+output_vec[prev_pos] as i16)/2)) as u8;
                    output_vec[position+1]=((decoder.read_next_symbol(&rgb_lookup)? as i16).wrapping_add((output_vec[v_pos+1] as i16+output_vec[prev_pos+1] as i16)/2)) as u8;
                    output_vec[position+2]=((decoder.read_next_symbol(&rgb_lookup)? as i16).wrapping_add((output_vec[v_pos+2] as i16+output_vec[prev_pos+2] as i16)/2)) as u8;
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
        prefix1 = decoder.read_next_symbol(&prefix_lookup)? as u8;
        if run_prefixes.iter().any(|&x| x == prefix1)
        {
            let mut temp_curr_runcount: u8=0;
            let mut run_length:usize=0;
            while let Some(&prefix_result)=run_prefixes.iter().find(|&&x| x == prefix1)
            {
                //run lengths
                run_length+=prefix_result as usize-5 << temp_curr_runcount;
                temp_curr_runcount += 3;
                prefix1 = decoder.read_next_symbol(&prefix_lookup)? as u8;
            }

            run_length += 1;
            
            for i in 0..run_length
            {
                output_vec.copy_within(prev_pos..=prev_pos+2, position+i*channels);
            }
            position+=run_length*channels;
        }
        //temp_time+=headertime.elapsed().as_nanos();
                
    
    }
    //println!("temp_time:{}: ",temp_time);
    Ok(image)
}
