

pub(crate) const SUBBLOCK_HEIGHT_MAX: usize = 5;
pub(crate) const SUBBLOCK_WIDTH_MAX: usize = 5;
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: u8,
    pub height_subblock_leftover: usize,
    pub width_subblock_leftover: usize,
    image_size:usize,
    pub width_block_size:usize,
    width_minus_leftover:usize,
    height_minus_leftover_times_width:usize,
    pub subblocks_in_width:usize,
    pub subblocks_in_height:usize

}

impl Image
{
    pub fn new
    (
        width:usize,
        height:usize,
        channels:u8
    )
    -> Self
    {
        Image{
            width,
            height,
            channels,
            height_subblock_leftover:height%SUBBLOCK_HEIGHT_MAX,
            width_subblock_leftover:width%SUBBLOCK_WIDTH_MAX,
            image_size:width*height,
            width_block_size:width*SUBBLOCK_HEIGHT_MAX,
            width_minus_leftover:width-width%SUBBLOCK_WIDTH_MAX,
            height_minus_leftover_times_width:(height-height%SUBBLOCK_HEIGHT_MAX)*width,
            subblocks_in_width: width/SUBBLOCK_WIDTH_MAX,
            subblocks_in_height: height/SUBBLOCK_HEIGHT_MAX
        }
    }

    pub fn calc_pos_from
    (
        &self,
        index : usize
    )
    -> usize
    {
        //take all main width spanning blocks
        let subblock_height;
        let subblock_width;
        let mut remainder;
        let mut offset;

        if index >= self.height_minus_leftover_times_width&&index<self.image_size
        {
            //near end of image
            subblock_height=self.height_subblock_leftover;
        }
        else
        {
            subblock_height=SUBBLOCK_HEIGHT_MAX;
        }
        //get width blocks
        offset=index-index%(self.width_block_size);
        remainder=index-offset;

        if remainder >= subblock_height*(self.width_minus_leftover)&&index<self.image_size
        {
            subblock_width=self.width_subblock_leftover
        }
        else
        {
            subblock_width=SUBBLOCK_WIDTH_MAX;
        }
        offset+=remainder/(subblock_width*subblock_height)*subblock_width;
        remainder=remainder%(subblock_width*subblock_height);
        //add innner block
        let amount_of_subrows_mod2=(remainder/subblock_width)&1;
        offset+=remainder/subblock_width*self.width;
        remainder=remainder%subblock_width;
        //add last row
        //offset+(amount_of_subrows_mod2)*(subblock_width-remainder-1)+remainder-remainder*(amount_of_subrows_mod2)
        offset+if amount_of_subrows_mod2==1{subblock_width-remainder-1}else{remainder}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_positions() {
        //let image = crate::image::Image{width:4000,height:3000,channels:3};

        /*debug_assert_eq!(image.calc_pos_from(126),18);
        debug_assert_eq!(image.calc_pos_from(9),4008);

        debug_assert_eq!(image.calc_pos_from(28000),28000);
        debug_assert_eq!(image.calc_pos_from(27999),27999);
        debug_assert_eq!(image.calc_pos_from(27996),27996);
        debug_assert_eq!(image.calc_pos_from(27992),23999);
        debug_assert_eq!(image.calc_pos_from(27995),23996);
        debug_assert_eq!(image.calc_pos_from(11999996),11999999);
        debug_assert_eq!(image.calc_pos_from(140353),156047);
        debug_assert_eq!(image.calc_pos_from(140672),468288/3);
        debug_assert_eq!(image.calc_pos_from(140671),468285/3);*/
        
    }    

}