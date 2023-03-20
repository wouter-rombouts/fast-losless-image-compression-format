use std::io;

pub struct Bitreader<'a, R : io::Read>
{
    pub reader : &'a  mut R,
    pub bit_offset : u8,
    pub cache : u32
}
//TODO version for u8 and 32
impl<R: io::Read> Bitreader<'_,R>
{
    pub fn read_bits3bytes
    (
            &mut self
    )
    -> std::io::Result<[u8;3]>
    {
        //if we need to read more than what is available in the cache
        /*if self.bit_offset > 8
        {*/

        let mut buffer =[0,0,0];
        self.reader.read_exact(&mut buffer)?;
        self.cache= u32::from_be_bytes([0,buffer[0],buffer[1],buffer[2]])  + (self.cache<<24);
        //self.bit_offset-=24;

        //}
        //move offset, but don't change the cache
        let ret  =((self.cache<<(self.bit_offset-24))>>8).to_be_bytes();
        //self.bit_offset+=24;
        //println!("output: {}",(self.bit_offset));
        Ok([ret[1],ret[2],ret[3]])
    }
    //TODO hardcode amount of bits?
    pub fn read_bitsu8
    (
            &mut self,
       amount_of_bits : u8
    )
    -> std::io::Result<u8>
    {
        //if we need to read more than what is available in the cache
        if amount_of_bits > 32-self.bit_offset
        {

            let mut buffer =[0];
            self.reader.read_exact(&mut buffer)?;
            self.cache= buffer[0] as u32  + (self.cache<<8);
            
            self.bit_offset-=8;

        }
        //move offset, but don't change the cache
        let ret  =((self.cache<<self.bit_offset)>>(32-amount_of_bits)).try_into().unwrap();
        self.bit_offset+=amount_of_bits;
        //println!("output: {}",(self.bit_offset));
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_reader() {
        let testreader = [252;6];
        let mut myreader = super::Bitreader{reader:&mut &testreader[..],bit_offset:32,cache:0};
        let test = myreader.read_bitsu8(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bitsu8(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bitsu8(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bitsu8(2).unwrap();
        debug_assert_eq!(test,0b0000_0000);
    }    

    #[test]
    fn check_reader4() {
        let testreader = [252,215,250,249,248,247,246,245,244,243,242];
        let mut myreader = super::Bitreader{reader:&mut &testreader[..],bit_offset:32,cache:0};
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test,[252,215,250]);
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test,[249,248,247]);
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test,[246,245,244]);
    }
}