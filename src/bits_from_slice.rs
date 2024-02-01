pub struct Bitfromslice<'a>{

    pub my_slice : &'a [u8] ,
    pub slice_offset : usize,
    pub bit_offset : u8,
    pub cache : u32
}
impl Bitfromslice<'_>{

    pub fn new( my_slice:& [u8],)
    ->Bitfromslice<'_>
    {
        Bitfromslice{ my_slice, bit_offset : 32, cache : 0,slice_offset:0}
    }
    pub fn read_24bits_noclear
    (
            &mut self,
    amount_of_bits : u8
    )
    -> usize
    {
        let aob_rev = 32-amount_of_bits;

        //if we need to read more than what is available in the cache
        while self.bit_offset > aob_rev
        {

            self.bit_offset-=8;
            self.cache=(self.cache<<8)+ self.my_slice[self.slice_offset] as u32;
            self.slice_offset+=1;
            
        }
        ((self.cache<<self.bit_offset)>>aob_rev)as usize
    }

    pub fn read_bitsu8
    (
            &mut self,
       amount_of_bits : u8
    )
    -> u8
    {
        //if we need to read more than what is available in the cache
        
        if amount_of_bits > 32 - self.bit_offset
        {
            self.cache= self.my_slice[self.slice_offset] as u32  + (self.cache<<8);
            self.bit_offset-=8;
            self.slice_offset+=1;

        }
        //move offset, but don't change the cache
        let ret  =((self.cache<<self.bit_offset)>>(32-amount_of_bits)).try_into().unwrap();
        self.bit_offset+=amount_of_bits;
        //println!("output: {}",(self.bit_offset));
        ret
    }
}