
use std::{collections::BinaryHeap};
use std::io::{self, Write, Read};
use crate::bitreader::Bitreader;
use crate::bitwriter::Bitwriter;
pub struct EncodedOutput<'a,W:Write>
{ pub symbols : [usize;256],
  pub data_vec : &'a mut Vec<u8>,
  pub bitwriter : Bitwriter<'a,W>
}
impl<W:Write> EncodedOutput<'_,W>
{
    pub fn add_symbolu8( &mut self, symbol : u8 )
    {
        self.data_vec.push(symbol);
        self.symbols[symbol as usize]+=1;
    }

    pub fn add_symbolusize( &mut self, symbol : usize )
    {
        self.data_vec.push(symbol as u8);
        self.symbols[symbol]+=1;
    }

    pub fn to_encoded_output( &mut self )

    -> Result<(), io::Error>
    {
        

        
        //calculate amount of bits for each color value, based on (flattened) huffman tree.
        //initialize 1 so no joining the last level which contains a lot of values in the symbols_under_node
        let mut amount_of_bits_per_symbol : [u8;256]=[1;256];
        let mut flat_tree = BinaryHeap::<TreeNode>::new();

        for i in 0..self.symbols.len()
        {
            flat_tree.push(TreeNode{ occurrences_sum : self.symbols[i],
                symbols_under_node : vec![i as u8] });
        }

        while flat_tree.len() > 2
        {
            let first = flat_tree.pop().unwrap();
            let second = flat_tree.pop().unwrap();
            let newnodeslist = [first.symbols_under_node,second.symbols_under_node].concat();
            //store codes(=amounts of bits<8) in output,0-255
            for el in newnodeslist.iter()
            {
                amount_of_bits_per_symbol[*el as usize]+=1;
            }
            flat_tree.push(TreeNode{ occurrences_sum : first.occurrences_sum+second.occurrences_sum,
                                     symbols_under_node : newnodeslist})
        }
        //build binary tree with color values based on amount of bits, in numerical order( bottom-up)
        //write amount of bits to output
        let symbols_lookup = amount_of_bits_to_bcodes(&amount_of_bits_per_symbol);

        let mut sumtot=0;
        for i in 0..self.symbols.len()
        {
            sumtot+=self.symbols[i] * symbols_lookup[i].1 as usize;
        }

        dbg!(sumtot);
        /*for i in 0..symbols_lookup.len()
        {
            dbg!(symbols_lookup[i].0);
            dbg!(symbols_lookup[i].1);
        }*/

        //TODO write codes
        let max_aob=*amount_of_bits_per_symbol.iter().max().unwrap();
        self.bitwriter.write_8bits(5, max_aob)?;
        
        for el in amount_of_bits_per_symbol
        {
            self.bitwriter.write_8bits(max_aob.next_power_of_two().count_zeros() as u8, el)?;
        }

        for el in &mut *self.data_vec
        {
            /*dbg!(symbols_lookup[*el as usize].1);
            dbg!(symbols_lookup[*el as usize].0 as u32);
            dbg!(self.bitwriter.bit_offset);
            dbg!(self.bitwriter.cache);*/
            /*if *el==30
            {
                dbg!(*el);
                dbg!(symbols_lookup[*el as usize]);
                dbg!(amount_of_bits_per_symbol[30]);
            }*/
            self.bitwriter.write_24bits(symbols_lookup[*el as usize].1, symbols_lookup[*el as usize].0 as u32)?;
        }
        self.bitwriter.writer.write_all(&[(self.bitwriter.cache>>24).try_into().unwrap()])?;
        //TODO test in vacuum

        Ok(())
    }
}
#[derive(PartialEq,PartialOrd,Eq,Ord,Debug)]
pub struct LookupItem
{
    code : usize,
    symbol : u8,
    aob : u8
}
pub struct DecodeInput<'a,R:Read>
{
    pub bitreader : Bitreader<'a,R>,
    symbols_lookup : Vec<LookupItem>,

    max_aob : u8
}

impl<R:Read> DecodeInput<'_,R>
{

    pub fn new( bitreader : Bitreader<'_,R> )
    -> DecodeInput<'_,R>
    {
        DecodeInput{ bitreader, symbols_lookup:Vec::new(), max_aob:0}
    }

    pub fn read_header_into_tree( &mut self )
    -> Result<(), io::Error>
    {
        self.max_aob=self.bitreader.read_bitsu8(5)?;
        let max_aob_bits = self.max_aob.next_power_of_two().count_zeros() as u8;
        let mut list_of_aobs=[0u8;256];
        for i in 0..256
        {
            list_of_aobs[i]=self.bitreader.read_bitsu8(max_aob_bits)?;
        }
        //v1.iter().copied().zip(v2.iter().copied()).collect()
        self.symbols_lookup = amount_of_bits_to_bcodes(&list_of_aobs).iter().copied().zip(list_of_aobs.iter().copied()).map(|((a,b),c)|LookupItem{code:a,symbol:b,aob:c}).collect();
        //bitshift codes
        //add symbols and order by smallest aob,largest code value
        for i in 0..(self.symbols_lookup.len())
        {
            self.symbols_lookup[i].code<<=(self.max_aob-self.symbols_lookup[i].symbol) as usize;
            self.symbols_lookup[i].symbol=i as u8;
        }
        //new values: bitshifted code and the symbol
        //sorted by bitshifted symbols, which is unique, so not sorted by symbol as .0 is never equal
        self.symbols_lookup.sort_unstable_by(|a, b|b.cmp(a));
        Ok(())
    }

    pub fn read_next_symbol( &mut self )
    -> Result<u8, io::Error>
    {
        //TODO encoder leaf nodes have highest values?
        let newcode=self.bitreader.read_24bits_noclear(self.max_aob)as usize;
        //let ret:&LookupItem=&self.symbols_lookup[self.symbols_lookup.partition_point(|lookupitem| newcode < lookupitem.code)];
        //dbg!(self.bitreader.bit_offset);
        Ok(&self.symbols_lookup[self.symbols_lookup.partition_point(|lookupitem| newcode < lookupitem.code)]).map(|i| {self.bitreader.bit_offset+=i.aob;i.symbol})
        //self.bitreader.bit_offset+=ret.aob;
        //Ok(ret.symbol)
    }
}

pub struct TreeNode
{
    pub occurrences_sum : usize,
    //if list empty then leaf node
    pub symbols_under_node : Vec<u8>
}

impl PartialEq for TreeNode
{
    fn eq(&self, other: &Self) -> bool {
        self.occurrences_sum == other.occurrences_sum
    }
}
//empty => accepts defaults in PartialEq
impl Eq for TreeNode
{
    
}

impl PartialOrd for TreeNode
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }

}
impl Ord for TreeNode
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.occurrences_sum.cmp(&self.occurrences_sum)
    }
}


//function to be called in encoder+decoder
pub fn amount_of_bits_to_bcodes( amount_of_bits_per_symbol : &[u8;256])
//codes(+amount of bits, not needed can use same index as input to derive amount of bits )
-> [(usize, u8); 256]
{
    //build btree based on aob in the order they appear(0-255)
    //make sure minheap respects order of same aob, also order on actual color value?

    let mut final_symbol_lookup=[(0usize,0u8);256];
    let mut current_code=0;
    
    let mut symbols_ordered:Vec<(usize,&u8)> = amount_of_bits_per_symbol.iter().enumerate().collect()/**/;
    //sort by highest aob, and highest symmbol.
    symbols_ordered.sort_by(|a,b|(b.1,b.0).cmp(&(a.1,a.0)));
    let mut prev_aob=0;

    for (symbol,amount_of_bits) in symbols_ordered
    {
        if *amount_of_bits<prev_aob
        {
            current_code>>=prev_aob-*amount_of_bits;
        }
        //don't add in the first iteration of the loop
        if prev_aob>0
        {
            current_code+=1;
        }
        //invert result so smallest aob gives highest bitshifted values in decoder(=faster)
        final_symbol_lookup[symbol].0=(1<<(*amount_of_bits))-current_code-1;
        final_symbol_lookup[symbol].1=*amount_of_bits;

        //update current_code
        prev_aob=*amount_of_bits;
    }
    //use flattened tree and then added 0 or (1 bitshifted) to final codes for encoding
    //performance diff with normal huffman tree???
    //for decoding, build an actual huffman tree and use "match" to decide
    //2 to the power aob can be used as value to build a huffman tree,although different codes can be the result, but the aob is same => same compression level.
    final_symbol_lookup
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_basic_encoding()
    {
        //initialize
        let occurrences=[0;256];
        
        let mut output_vec : Vec<u8> = Vec::new();
        let mut encoder = super::EncodedOutput{ symbols : occurrences,
                                                data_vec : &mut Vec::<u8>::with_capacity(2560000),
                                                bitwriter : crate::bitwriter::Bitwriter::new(&mut output_vec) };
        //add data
        let now = std::time::Instant::now();
        for i in 0..=255
        {
            for _j in 0..i*10
            {
                encoder.add_symbolusize(i);
            }
        }
        //encode
        encoder.to_encoded_output().unwrap();
        let cache=encoder.bitwriter.cache.to_be_bytes();
        output_vec.extend_from_slice(&cache[..]);
        println!("encoder speed: {}", now.elapsed().as_millis());
        //read
        dbg!(output_vec.len());
        let mut binding = output_vec.as_slice();
        let mut decoder = super::DecodeInput::new(  crate::bitreader::Bitreader::new( &mut binding ));
        
        let now = std::time::Instant::now();
        decoder.read_header_into_tree().unwrap();
        //dbg!(decoder.symbols_lookup);
        //TODO: opti decoder speed
        for i in 0usize..=255
        {
            for _j in 0..i*10
            {
                /*dbg!(i);
                dbg!(_j);*/
            let res =decoder.read_next_symbol().unwrap();
            debug_assert_eq!(res,(i) as u8,"i:{}",i);
            }
        }
        println!("decoder speed: {}", now.elapsed().as_millis());
        
        //TODO put back leftover bits
        
        
    }    

}