use std::io;

pub struct Bitreader<'a, R : io::Read>
{
    pub reader : &'a  mut R,
    pub bit_offset : u8,
    pub cache : u32,
    buffer : [u8;1]
}
//TODO version for u8 and 32
impl<R: io::Read> Bitreader<'_,R>
{
    pub fn new( reader : &mut R)
    ->Bitreader<'_,R>
    {
        Bitreader{ reader, bit_offset : 32, cache : 0,buffer:[0]}
    }

    pub fn read_bits3bytes
    (
            &mut self
    )
    -> std::io::Result<u32>
    {
        let mut buffer =[0,0,0];
        self.reader.read_exact(&mut buffer)?;
        self.cache= (self.cache<<24) + ((buffer[0] as u32) <<16) + ((buffer[1] as u32)<<8)+(buffer[2] as u32 ) ;
        Ok((self.cache<<(self.bit_offset-24))>>8)
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
        
        if amount_of_bits > 32 - self.bit_offset
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

    pub fn read_24bits
    (
            &mut self,
       amount_of_bits : u8
    )
    -> u32
    {
        //if we need to read more than what is available in the cache
        let aob_rev = 32-amount_of_bits;
        while self.bit_offset > aob_rev
        {
            self.reader.read_exact(&mut self.buffer).expect("error reading the io source");
            self.bit_offset-=8;
            self.cache= self.buffer[0] as u32  + (self.cache<<8);

        }
        //move offset
        self.bit_offset+=amount_of_bits;
        (self.cache<<(self.bit_offset-amount_of_bits))>>aob_rev
    }

    
    pub fn read_24bits_noclear
    (
            &mut self,
       amount_of_bits : u8
    )
    -> Result<usize, io::Error>
    {
        let aob_rev = 32-amount_of_bits;

        //if we need to read more than what is available in the cache
        while self.bit_offset > aob_rev
        {
            match self.reader.read(&mut self.buffer)
            {
                Err(e)=>{return Err(e);},
                Ok(_)=>{
                    self.bit_offset-=8;
                    self.cache=(self.cache<<8)+ self.buffer[0] as u32;
                }
            }
        }
        Ok(((self.cache<<self.bit_offset)>>aob_rev)as usize)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_reader() {
        let testreader = [252;6];
        let mut binding = &testreader[..];
        let mut myreader = super::Bitreader::new(&mut binding);
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
    fn check_reader3() {
        let testreader = [252,215,250,249,248,247,246,245,244,243,242];
        let mut binding = &testreader[..];
        let mut myreader = super::Bitreader::new(&mut binding);
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test.to_be_bytes()[1..=3],[252,215,250]);
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test.to_be_bytes()[1..=3],[249,248,247]);
        let test = myreader.read_bits3bytes().unwrap();
        debug_assert_eq!(test.to_be_bytes()[1..=3],[246,245,244]);
    }
    #[test]
    fn check_reader24() {
        let testreader = [252;6];
        let mut myreader = super::Bitreader{reader:&mut &testreader[..],bit_offset:32,cache:0, buffer: [0] };

        let test = myreader.read_24bits(9);
        debug_assert_eq!(test,0b111111001);
        let test = myreader.read_24bits_noclear(9).unwrap();
        debug_assert_eq!(test,0b111110011);

        let test = myreader.read_24bits(9);
        debug_assert_eq!(test,0b111110011);
        let test = myreader.read_24bits(9);
        debug_assert_eq!(test,0b111100111);
    }
}