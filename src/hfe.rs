
use std::{collections::BinaryHeap};
use std::io::{self, Write, Read};
use crate::bitreader::Bitreader;
use crate::bitwriter::Bitwriter;
pub struct EncodedOutput<'a,W:Write>
{ pub symbol_occurs : Box<[usize]>,
  pub data_vec : &'a mut Vec<u8>,
  pub bitwriter : Bitwriter<'a,W>
}
impl<W:Write> EncodedOutput<'_,W>
{
    pub fn add_symbolu8( &mut self, symbol : u8 )
    {
        self.data_vec.push(symbol);
        self.symbol_occurs[symbol as usize]+=1;
    }

    pub fn add_symbolusize( &mut self, symbol : usize )
    {
        self.data_vec.push(symbol as u8);
        self.symbol_occurs[symbol]+=1;
    }

    pub fn to_encoded_output( &mut self )

    -> Result<(), io::Error>
    {
        

        
        //calculate amount of bits for each color value, based on (flattened) huffman tree.
        //initialize 1 so no joining the last level which contains a lot of values in the symbols_under_node
        let mut bcodes : Vec<Bcode>=vec![Bcode{ aob: 1, code: 0 };self.symbol_occurs.len()];
        let mut flat_tree = BinaryHeap::<TreeNode>::new();


        for i in 0..self.symbol_occurs.len()
        {
            flat_tree.push(TreeNode{ occurrences_sum : self.symbol_occurs[i],
                                     symbols_under_node : vec![i as u8]});
        }

        while flat_tree.len() > 2
        {
            let first = flat_tree.pop().unwrap();
            let second = flat_tree.pop().unwrap();
            let treenode=TreeNode{ occurrences_sum : first.occurrences_sum + second.occurrences_sum,
                                   symbols_under_node : [first.symbols_under_node, second.symbols_under_node].concat()};
            //store codes(=amounts of bits<8) in output,0-255
            for el in treenode.symbols_under_node.iter()
            {
                bcodes[*el as usize].aob+=1;
            }
            flat_tree.push(treenode);
        }
        //build binary tree with color values based on amount of bits, in numerical order( bottom-up)
        //write amount of bits to output
        amount_of_bits_to_bcodes(&mut bcodes);
        /*#[cfg(debug_assertions)]
        let mut sumtot=0;
        #[cfg(debug_assertions)]
        for i in 0..self.symbol_occurs.len()
        {
            sumtot+=self.symbol_occurs[i] * symbols_lookup[i].1 as usize;
        }

        dbg!(sumtot);*/

        //TODO write codes
        let max_aob=bcodes.iter().max().unwrap().aob;
        self.bitwriter.write_8bits(5, max_aob)?;
        
        for el in bcodes.iter()
        {
            self.bitwriter.write_8bits(max_aob.next_power_of_two().count_zeros() as u8, el.aob)?;
        }

        for el in &mut *self.data_vec
        {
            self.bitwriter.write_24bits(bcodes[*el as usize].aob, bcodes[*el as usize].code as u32)?;
        }
        self.bitwriter.writer.write_all(&[(self.bitwriter.cache>>24).try_into().unwrap()])?;
        //TODO test in vacuum

        Ok(())
    }
}


#[derive(Clone,Ord,Eq,PartialEq,PartialOrd,Copy)]
pub struct Bcode
{
    pub aob : u8,
    pub code : usize

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

    pub fn read_header_into_tree( &mut self, amount_of_symbols : usize )
    -> Result<(), io::Error>
    {
        self.max_aob=self.bitreader.read_bitsu8(5)?;
        let max_aob_bits = self.max_aob.next_power_of_two().count_zeros() as u8;

        let mut bcodes : Vec<Bcode>=vec![Bcode{ aob: 0, code: 0 };amount_of_symbols];
        for i in 0..amount_of_symbols
        {
            bcodes[i].aob=self.bitreader.read_bitsu8(max_aob_bits)?;
        }
        
        //bitshift codes
        //add symbols and order by smallest aob,largest code value
        amount_of_bits_to_bcodes(&mut bcodes);

        self.symbols_lookup=Vec::with_capacity(amount_of_symbols);
        for (i,code_symbol) in bcodes.iter().enumerate()
        {
            self.symbols_lookup.push(LookupItem{ code: code_symbol.code<<(self.max_aob-code_symbol.aob) as usize, symbol: i as u8, aob: code_symbol.aob });
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
pub fn amount_of_bits_to_bcodes( codes : &mut Vec<Bcode>)
//codes(+amount of bits, not needed can use same index as input to derive amount of bits )
{
    //build btree based on aob in the order they appear(0-255)
    //make sure minheap respects order of same aob, also order on actual color value?

    //let mut final_symbol_lookup=vec![(0usize,0u8);amount_of_bits_per_symbol.len()];
    let mut current_code=0;
    
    let mut symbols_ordered:Vec<(usize,Bcode)>=Vec::with_capacity(codes.len());
    for i in 0..codes.len()
    {
        symbols_ordered.push((i,codes[i]));
    }
    //sort by highest aob, and highest symmbol.
    symbols_ordered.sort_unstable_by(|a,b|(b.1.aob,b.0).cmp(&(a.1.aob,a.0)));
    let mut prev_aob=0;

    for (symbol,bcode) in symbols_ordered.iter()
    {
        if bcode.aob<prev_aob
        {
            current_code>>=prev_aob-bcode.aob;
        }
        //don't add in the first iteration of the loop
        if prev_aob>0
        {
            current_code+=1;
        }
        //invert result so smallest aob gives highest bitshifted values in decoder(=faster)
        codes[*symbol].code=(1<<(bcode.aob))-current_code-1;
        //codes[symbol].aob=bcode.aob;

        //update current_code
        prev_aob=bcode.aob;
    }
    //use flattened tree and then added 0 or (1 bitshifted) to final codes for encoding
    //performance diff with normal huffman tree???
    //for decoding, build an actual huffman tree and use "match" to decide
    //2 to the power aob can be used as value to build a huffman tree,although different codes can be the result, but the aob is same => same compression level.
    
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_basic_encoding()
    {
        //initialize
        const SIZE_ARR:usize=256;
        let occurrences=Box::new([0usize;SIZE_ARR]);
        
        let mut output_vec : Vec<u8> = Vec::new();
        let mut encoder = super::EncodedOutput{ symbol_occurs : occurrences,
                                                data_vec : &mut Vec::<u8>::with_capacity(SIZE_ARR*1000),
                                                bitwriter : crate::bitwriter::Bitwriter::new(&mut output_vec) };
        //add data
        let now = std::time::Instant::now();
        for i in 0..SIZE_ARR
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
        decoder.read_header_into_tree(SIZE_ARR).unwrap();
        //dbg!(decoder.symbols_lookup);
        //TODO: opti decoder speed
        for i in 0usize..SIZE_ARR
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