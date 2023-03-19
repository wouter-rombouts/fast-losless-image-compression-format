use std::io;

pub struct Bitreader<'a, R : io::Read>
{
    pub reader : &'a  mut R,
    pub bit_offset : u8,
    pub cache : u16
}

impl<R: io::Read> Bitreader<'_,R>
{
    pub fn read_bits
    (
            &mut self,
       amount_of_bits : u8
    )
    -> std::io::Result<u8>
    {
        //if we need to read more than what is available in the cache
        if amount_of_bits > 16-self.bit_offset
        {
            let mut buffer =[0];
            self.reader.read_exact(&mut buffer)?;
            self.cache=buffer[0] as u16 + (self.cache<<8);
            
            self.bit_offset-=8;

        }
        //move offset, but don't change the cache
        let ret  =((self.cache<<self.bit_offset)>>(16-amount_of_bits)).try_into().unwrap();
        self.bit_offset+=amount_of_bits;
        //println!("output: {}",(self.bit_offset));
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_reader() {
        let testreader = [252;4];
        let mut myreader = super::Bitreader{reader:&mut &testreader[..],bit_offset:16,cache:0};
        let test = myreader.read_bits(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bits(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bits(2).unwrap();
        debug_assert_eq!(test,0b0000_0011);
        let test = myreader.read_bits(2).unwrap();
        debug_assert_eq!(test,0b0000_0000);
    }
}