use std::io;

pub struct Bitwriter<'a, W : io::Write>
{
    pub writer : &'a  mut W,
    pub bit_offset : u8,
    pub cache : u32
}

impl<W: io::Write> Bitwriter<'_,W>
{    
    pub fn write_bits_u8
    (
            &mut self,
       amount_of_bits : u8,
                value : u8
    )
    -> std::io::Result<()>
    {
        self.bit_offset+= amount_of_bits;
        self.cache+= (value as u32)<<( 32-self.bit_offset );
        //write cache to buffer
        if self.bit_offset >= 8
        {
            self.writer.write_all(&[(self.cache>>24) as u8])?;
            self.bit_offset -= 8;
            self.cache = self.cache<<8;
        }
        Ok(())
    }
    
    pub fn write_24bits
    (
            &mut self,
                value : u32
    )
    -> std::io::Result<()>
    {
        //self.bit_offset += 24;
        self.cache+= value<<( 8-self.bit_offset );
        //write cache to buffer

        self.writer.write_all(&(self.cache).to_be_bytes()[0..3])?;

        //self.bit_offset -= 24;
        self.cache = self.cache<<24;
        Ok(())
    }
}

mod tests {
    #[test]
    fn check_writer() {
        let mut my_output= Vec::new();
        let mut myreader = super::Bitwriter{writer:&mut my_output,bit_offset:0,cache:0};
        let test = myreader.write_bits_u8(2,3).unwrap();
        
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0000_0011);
        let test = myreader.write_bits_u8(2,3).unwrap();
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0000_1111);
        let test = myreader.write_bits_u8(2,3).unwrap();
        //debug_assert_eq!(*my_output.get(0).unwrap(),0b0011_1111);
        let test = myreader.write_bits_u8(2,0).unwrap();
        debug_assert_eq!(*my_output.get(0).unwrap(),0b1111_1100);
    }
}