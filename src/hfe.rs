
use std::{collections::BinaryHeap};
use std::io::{self, Write, Read};
use crate::bitreader::Bitreader;
use crate::bits_from_slice;
use crate::bitwriter::Bitwriter;
use std::rc::Rc;

/*struct Occurrences{
    //pub output_type : u8,
    list_of_occurs : Box<[usize]>
}*/
pub struct EncodedOutput
{ pub symbol_occurs : Vec<usize>,
  pub data_vec : Vec<u16>
}
impl EncodedOutput
{
    pub fn new( amount_of_symbols : usize, data_size_estimate : usize)
    -> EncodedOutput
    {
        EncodedOutput{ symbol_occurs: vec![0;amount_of_symbols], data_vec: Vec::<u16>::with_capacity(data_size_estimate) }
    }

    pub fn end( &mut self)
    {
        //TODO write cache to data_vec before output?
    }

    pub fn add_symbolu8( &mut self, symbol : u8 )
    {
        self.data_vec.push(symbol as u16);
        self.symbol_occurs[symbol as usize]+=1;
    }    
    
    pub fn add_symbolu16( &mut self, symbol : u16 )
    {
        self.data_vec.push(symbol);
        self.symbol_occurs[symbol as usize]+=1;
    }

    
    pub fn add_symbolusize( &mut self, symbol : usize )
    {
        self.data_vec.push(symbol as u16);
        self.symbol_occurs[symbol]+=1;
    }

    /*pub fn add_output_type( &mut self, size : usize)
    {
        self.symbol_occurs.push(vec![0;size]);
    }*/
    pub fn to_encoded_output<'a,W:Write>( &mut self, bitwriter : &mut Bitwriter<'a, W> )

    -> Result<(), io::Error>
    {

        //TODO create a lookup table with codes for each output type
        //let mut list_of_bcodes : Vec<Vec<Bcode>> = Vec::new();

            //calculate amount of bits for each color value, based on (flattened) huffman tree.
            //initialize 1 so no joining the last level which contains a lot of values in the symbols_under_node
            let mut bcodes : Vec<Bcode>=vec![Bcode{ aob: 1, code: 0 };self.symbol_occurs.len()];
            let mut flat_tree = BinaryHeap::<TreeNode>::new();


            for i in 0..self.symbol_occurs.len()
            {
                flat_tree.push(TreeNode{ occurrences_sum : self.symbol_occurs[i],
                                        symbols_under_node : vec![i as u16]});
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
            for i in 0..self.symbol_occurs[occurs_i].len()
            {
                sumtot+=self.symbol_occurs[occurs_i][i] * bcodes[i].aob as usize;
            }

            dbg!(sumtot);*/
            let max_aob=bcodes.iter().max().unwrap().aob;
            bitwriter.write_8bits(5, max_aob)?;
            
            for el in bcodes.iter()
            {
                bitwriter.write_8bits(max_aob.next_power_of_two().count_zeros() as u8, el.aob)?;
            }
            //list_of_bcodes.push(bcodes);

        //decodingstream should know how many output types there are

        //TODO improve final output see png blog
        //TODO Huffman lookup table for double symbol(max), then overwrite if only 1 symbol.
        //enums possible?
        for el in &mut *self.data_vec
        {
            let el_bcode=bcodes[*el as usize];
            bitwriter.write_24bits(el_bcode.aob, el_bcode.code as u32)?;
        }
        //TODO move to code.rs at the very end
        bitwriter.writer.write_all(&[(bitwriter.cache>>24).try_into().unwrap()])?;
        Ok(())
    }
}


#[derive(Clone,Ord,Eq,PartialEq,PartialOrd,Copy)]
pub struct Bcode
{
    pub aob : u8,
    pub code : usize

}

#[derive(Clone)]
pub struct SymbolstreamLookup
{
    //does this need to be saved?
    pub max_aob : u8,
    //size is 2^max_aob
    pub symbol_lookup : Vec<SymbolLookupItem>,
    //index is value from symbol_lookup
    aob_lookup : Vec<SymbolLookupItem>
}

impl SymbolstreamLookup
{
    pub fn new( size : usize )
    ->SymbolstreamLookup
    {
        SymbolstreamLookup{max_aob:0, symbol_lookup : Vec::new(),aob_lookup:Vec::<SymbolLookupItem>::with_capacity(size)}
    }
}

#[derive(Clone,Copy)]
pub struct SymbolLookupItem
{
    pub symbol : u16,
    pub aob : u8   
}
#[derive(Clone,Copy)]
pub struct SymbolLookupItemUsize
{
    pub symbol : usize,
    pub aob : u8   
}
pub struct SymbolstreamLookupUsize
{
    //does this need to be saved?
    pub max_aob : u8,
    //size is 2^max_aob
    pub symbol_lookup : Vec<SymbolLookupItemUsize>,
    //index is value from symbol_lookup
    aob_lookup : Vec<SymbolLookupItemUsize>
}
impl SymbolstreamLookupUsize
{
    pub fn new( size : usize )
    ->SymbolstreamLookupUsize
    {
        SymbolstreamLookupUsize{max_aob:0, symbol_lookup : Vec::new(),aob_lookup:Vec::<SymbolLookupItemUsize>::with_capacity(size)}
    }
}
pub struct DecodeInput<'a>
{
    pub input_data : &'a [u8],
    //lookup : SymbolstreamLookupUsize
    //max_aob : u8
}

impl DecodeInput<'_>
{



    pub fn new( data : & [u8] )
    -> DecodeInput
    {
        DecodeInput{ input_data:data}
    }

    /*pub fn read_header_into_tree_and_output_usize( &mut self, size : usize )
    -> Result<(), io::Error>
    {
            let mut aob_vec=SymbolstreamLookupUsize::new(size);
            let mut bitreader = Bitreader::new(&mut &Rc::get_mut(&mut self.input_data).unwrap()[..]);
            let amount_of_symbols=aob_vec.aob_lookup.capacity();
            aob_vec.max_aob=bitreader.read_bitsu8(5)?;
            let max_aob_bits = aob_vec.max_aob.next_power_of_two().count_zeros() as u8;

            let mut bcodes : Vec<Bcode>=vec![Bcode{ aob: 0, code: 0 };amount_of_symbols];
            aob_vec.aob_lookup=Vec::with_capacity(amount_of_symbols);
            for i in 0..amount_of_symbols
            {
                bcodes[i].aob=bitreader.read_bitsu8(max_aob_bits)?;
                aob_vec.aob_lookup.push(SymbolLookupItemUsize{ symbol: i as usize, aob: bcodes[i].aob });
            }
            
            //bitshift codes
            //add symbols and order by smallest aob,largest code value
            amount_of_bits_to_bcodes(&mut bcodes);
            aob_vec.symbol_lookup=vec![SymbolLookupItemUsize{ symbol: 0, aob: 0 };(1<<aob_vec.max_aob as usize)];

            for (i,code_symbol) in bcodes.iter().enumerate()
            {
                let code_shifted=code_symbol.code<<(aob_vec.max_aob-code_symbol.aob);
                let code_shifted_plus1=(code_symbol.code+1)<<(aob_vec.max_aob-code_symbol.aob);
                for sl_index in code_shifted..code_shifted_plus1
                {
                    
                    aob_vec.symbol_lookup[sl_index]=aob_vec.aob_lookup[i];
                }
            }
        Ok(())
    }*/

    pub fn read_header_into_tree( &mut self, size : usize )
    -> Result<Vec<u16>, io::Error>
    {
        let mut output : Vec<u16>=Vec::with_capacity(self.input_data.len());
        let mut aob_vec=SymbolstreamLookup::new(size);
        let mut input = bits_from_slice::Bitfromslice::new(self.input_data);
        //let mut binding = &Rc::get_mut(&mut self.input_data).unwrap()[..];
        //let mut bitreader = Bitreader::new(&mut binding);
        let amount_of_symbols=aob_vec.aob_lookup.capacity();
        aob_vec.max_aob=input.read_bitsu8(5);
        let max_aob_bits = aob_vec.max_aob.next_power_of_two().count_zeros() as u8;

        let mut bcodes : Vec<Bcode>=vec![Bcode{ aob: 0, code: 0 };amount_of_symbols];
        aob_vec.aob_lookup=Vec::with_capacity(amount_of_symbols);
        for i in 0..amount_of_symbols
        {
            bcodes[i].aob=input.read_bitsu8(max_aob_bits);
            aob_vec.aob_lookup.push(SymbolLookupItem{ symbol: i as u16, aob: bcodes[i].aob });
        }
        
        //bitshift codes
        //add symbols and order by smallest aob,largest code value
        amount_of_bits_to_bcodes(&mut bcodes);
        aob_vec.symbol_lookup=vec![SymbolLookupItem{ symbol: 0, aob: 0 };(1<<aob_vec.max_aob as usize)];

        for (i,code_symbol) in bcodes.iter().enumerate()
        {
            let code_shifted=code_symbol.code<<(aob_vec.max_aob-code_symbol.aob);
            let code_shifted_plus1=(code_symbol.code+1)<<(aob_vec.max_aob-code_symbol.aob);
            for sl_index in code_shifted..code_shifted_plus1
            {
                
                aob_vec.symbol_lookup[sl_index]=aob_vec.aob_lookup[i];
            }
        }
        //TODO read symbols into output

        while input.slice_offset<input.my_slice.len()
        {
            
        
                    let lookup = aob_vec.symbol_lookup[input.read_24bits_noclear(aob_vec.max_aob)];
                    input.bit_offset+=lookup.aob;
                    output.push(lookup.symbol);
        }
        //TODO clear and output cache
        while input.bit_offset<32
        {
            let lookup = aob_vec.symbol_lookup[((input.cache<<input.bit_offset)>>(32-aob_vec.max_aob))as usize];
            input.bit_offset+=lookup.aob;
            output.push(lookup.symbol);
        }
        //input.cache
        Ok(output)
    }
    /*pub fn read_next_symbol_usize( &mut self, lookup : &SymbolstreamLookupUsize )
    -> Result<usize, io::Error>
    {
        match self.bitreader.read_24bits_noclear(lookup.max_aob)
        {
            Ok(newcode)=>
            {
                let lookup = lookup.symbol_lookup[newcode];
                self.bitreader.bit_offset+=lookup.aob;
                Ok(lookup.symbol)
            },
            Err(e)=>
            {
                Err(e)
            }
        }
    }*/
    /*pub fn read_next_symbol( &mut self, lookup : &SymbolstreamLookup )
    -> Result<u16, io::Error>
    {
        match self.bitreader.read_24bits_noclear(lookup.max_aob)
        {
            Ok(newcode)=>
            {
                let lookup = lookup.symbol_lookup[newcode];
                self.bitreader.bit_offset+=lookup.aob;
                Ok(lookup.symbol)
            },
            Err(e)=>
            {
                Err(e)
            }
        }
    }*/
}
#[derive(Eq)]
pub struct TreeNode
{
    pub occurrences_sum : usize,
    //if list empty then leaf nodes
    pub symbols_under_node : Vec<u16>
}

impl PartialEq for TreeNode
{
    fn eq(&self, other: &Self) -> bool {
        self.occurrences_sum == other.occurrences_sum
    }
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
        
        let mut output_vec : Vec<u8> = Vec::new();
        let mut encoder = super::EncodedOutput::new(256,SIZE_ARR*1000);
        //encoder.add_output_type(SIZE_ARR);
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
        let mut mywriter=crate::bitwriter::Bitwriter::new(&mut output_vec);
        encoder.to_encoded_output(&mut mywriter).unwrap();
        //let cache=mywriter.cache.to_be_bytes();
        //output_vec.extend_from_slice(&cache[..]);
        println!("encoder speed: {}", now.elapsed().as_millis());
        //read
        dbg!(output_vec.len());
        //let mut binding = output_vec.as_slice();
        let mut decoder = super::DecodeInput::new(  &output_vec );
        
        let now = std::time::Instant::now();
        //let mut symbol_lookup = crate::hfe::SymbolstreamLookup::new(SIZE_ARR);
        let output = decoder.read_header_into_tree(SIZE_ARR).unwrap();
        //dbg!(decoder.symbols_lookup);
        //TODO: opti decoder speed
        println!("decoder speed: {}", now.elapsed().as_millis());
        let mut amount_processed=0;
        for i in 0usize..SIZE_ARR
        {
            for j in 0..i*10
            {
                //let res =decoder.read_next_symbol(&symbol_lookup);
                debug_assert_eq!(output[amount_processed],(i) as u16,"i:{},j:{}",i,j);
                amount_processed+=1;
            }
        }
        
        //TODO put back leftover bits
        
        
    }    

}