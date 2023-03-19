use std::io;

pub struct Bitwriter<'a, W : io::Write>
{
    pub writer : &'a  mut W,
    pub bit_offset : u8,
    pub cache : u16
}

impl<W: io::Write> Bitwriter<'_,W>
{    
    pub fn write_bits
    (
            &mut self,
       amount_of_bits : u8,
                value : u8
    )
    -> std::io::Result<()>
    {
        self.bit_offset += amount_of_bits;
        self.cache = self.cache | ((value as u16)<<( 16-self.bit_offset ));
        //write cache to buffer
        if self.bit_offset >= 8
        {
            self.writer.write_all(&[(self.cache>>8) as u8])?;
                //*byte = (self.cache>>8) as u8;

            self.bit_offset -= 8;
            self.cache = self.cache<<8;
        }
        Ok(())
    }
}

mod tests {
    #[test]
    fn check_writer() {
        let mut my_output= Vec::new();
        let mut myreader = super::Bitwriter{writer:&mut my_output,bit_offset:0,cache:0};
        let test = myreader.write_bits(2,3).unwrap();
        
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0000_0011);
        let test = myreader.write_bits(2,3).unwrap();
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0000_1111);
        let test = myreader.write_bits(2,3).unwrap();
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0011_1111);
        let test = myreader.write_bits(2,0).unwrap();
        debug_assert_eq!(*my_output.get(0).unwrap(),0b1111_1100);
    }
}