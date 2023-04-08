
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

        //
        //

        if index >= (self.height-self.height%7)*self.width
        {
            //near end of image
            subblock_height=self.height%7;
        }
        else
        {
            subblock_height=7;
        }
        //get width blocks
        offset=index/(self.width*7)*(self.width*7);
        remainder=index-offset;

        if remainder >= subblock_height*(self.width-self.width%9)
        {
            subblock_width=self.width%9
        }
        else
        {
            subblock_width=9;
        }
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

        debug_assert_eq!(image.calc_pos_from(126),18);
        debug_assert_eq!(image.calc_pos_from(9),4008);

        debug_assert_eq!(image.calc_pos_from(28000),28000);
        debug_assert_eq!(image.calc_pos_from(27999),27999);
        debug_assert_eq!(image.calc_pos_from(27996),27996);
        debug_assert_eq!(image.calc_pos_from(27992),23999);
        debug_assert_eq!(image.calc_pos_from(27995),23996);
        debug_assert_eq!(image.calc_pos_from(11999996),11999999);
    }    

}