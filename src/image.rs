

pub(crate) const SUBBLOCK_HEIGHT_MAX: usize = 7;
pub(crate) const SUBBLOCK_WIDTH_MAX: usize = 9;
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: u8,
}

impl Image
{
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

        if index >= (self.height-self.height%SUBBLOCK_HEIGHT_MAX)*self.width&&index<self.width*self.height
        {
            //near end of image
            subblock_height=self.height%SUBBLOCK_HEIGHT_MAX;
        }
        else
        {
            subblock_height=SUBBLOCK_HEIGHT_MAX;
        }
        //get width blocks
        offset=index/(self.width*SUBBLOCK_HEIGHT_MAX)*(self.width*SUBBLOCK_HEIGHT_MAX);
        remainder=index-offset;

        if remainder >= subblock_height*(self.width-self.width%SUBBLOCK_WIDTH_MAX)&&index<self.width*self.height
        {
            subblock_width=self.width%SUBBLOCK_WIDTH_MAX
        }
        else
        {
            subblock_width=SUBBLOCK_WIDTH_MAX;
        }
        /*if index >11999200
        {
        dbg!(index);
        dbg!(subblock_width);
        dbg!(subblock_height);
            }*/
        offset+=remainder/(subblock_width*subblock_height)*subblock_width;
        remainder=remainder%(subblock_width*subblock_height);
        //add innner block
        let amount_of_subrows=remainder/subblock_width;
        offset+=remainder/subblock_width*self.width;
        remainder=remainder%subblock_width;
        //add last row
        offset+if amount_of_subrows%2==1{subblock_width-remainder-1}else{remainder}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_positions() {
        let image = crate::image::Image{width:4000,height:3000,channels:3};

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