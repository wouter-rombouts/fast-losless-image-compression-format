
use std::{collections::BinaryHeap, cmp::Ordering};
struct CodeListItem
{ symbols : [u8]
}

//occurs amount, amount of bits
pub fn occurs_to_list_of_codes( mut occurrences : [ (usize,u8);256 ], total_length : &usize )
-> ()
{

 //calc amount of bits for occur
 loop
 { 
 }

 
 
 //create list if not exists or create on beforehand 
 //calc code on base number(array number) + symbol offset(pos within inner array)
 //apply codes to data
 //convert to canonical and store codes
}


pub enum TreeNodeBranchOrLeaf
{
    Leaf(u8),
    Branch((Box<TreeNode>,Box<TreeNode>))

}
pub struct TreeNode
{
    pub occurrences_sum : usize,
     pub branch_or_leaf : TreeNodeBranchOrLeaf
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
        Some(self.cmp(other))/*
        match self.occurrences_sum.partial_cmp(&other.occurrences_sum) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.split1.partial_cmp(&other.split1)*/
    }

}
impl Ord for TreeNode
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.occurrences_sum.cmp(&self.occurrences_sum)
    }
}
pub struct HuffmanTree
{
    tree : BinaryHeap<TreeNode>
}
impl HuffmanTree
{
    pub fn new( occurrences : [usize;256]
    )
    ->HuffmanTree
    {
        let mut tree  = BinaryHeap::<TreeNode>::new();
        for (i,occurrence) in occurrences.iter().enumerate()
        {
            tree.push(TreeNode{ occurrences_sum : *occurrence,
                                 branch_or_leaf : TreeNodeBranchOrLeaf::Leaf(i as u8) });
        }
        HuffmanTree{tree}
    }

    pub fn list_of_nodes_to_tree( &mut self )
    {
        //get lowest occurences amount
        //taking 2 and add 1 will result in the lowest value always existing
        loop
        {
            let lowest = self.tree.pop().unwrap();
            let second_lowest = self.tree.pop();
            if let Some(second_value) = second_lowest
            {
                self.tree.push( TreeNode{ occurrences_sum : lowest.occurrences_sum+second_value.occurrences_sum,
                                           branch_or_leaf : TreeNodeBranchOrLeaf::Branch( ( Box::new(lowest), 
                                                                                            Box::new(second_value) ) ) } );
            }
            else
            {
                break;
            }
        }

    }
    
    fn node_to_amounts( &mut self, node : TreeNode, current_level : u8, symbol_and_amounts : &mut Vec::<(u8,u8)> )
    {
        
        match node.branch_or_leaf
        {
            TreeNodeBranchOrLeaf::Leaf(value) =>
            {
                symbol_and_amounts.push((value,current_level));
            },
            TreeNodeBranchOrLeaf::Branch((new_node1, new_node2)) =>
            {
                self.node_to_amounts(*new_node1,current_level+1,symbol_and_amounts);
                self.node_to_amounts(*new_node2,current_level+1,symbol_and_amounts);
            }
        }
    }

    pub fn tree_to_amounts( &mut self
    )
    -> Vec<(u8,u8)>
    {
        //amount of bits for this code, index is element
        let mut symbol_and_amounts = Vec::<(u8,u8)>::with_capacity(256);
        //huffman_codes=vec![0;256];
        //let mut temp_bits = Vec::<u8>::with_capacity(44);
        let root = self.tree.pop().unwrap();
        self.node_to_amounts(root,0,&mut symbol_and_amounts);
        //let tree_iter = self.tree.iter();


        symbol_and_amounts
    }
}

pub struct canonical_symbol_element
{
    amount : u8,
    symbol : u8
}

//amount_of_bits: symbol and amount of bits
pub fn amount_of_bits_to_codes(  amount_of_bits : &mut Vec<(u8,u8)>
)
//max bits and list of amount of bits and code
->(u8,[(u8,u64);256])
{
    let mut ret = [(0,0);256];
    //length and value
    //re-order based on length, numerical codes stay the same
    amount_of_bits.sort_by(|&a,&b|{
        a.1.cmp(&b.1)
    });
    //first zero's, for most common symbol,position is value
    ret[amount_of_bits[0].0 as usize]=(amount_of_bits[0].1,0);
    //
    let mut temp_num = 0u64;
    for i in 1..256 
    {   
            temp_num+=1;
            if amount_of_bits[i].1 != amount_of_bits[i-1].1
            {
                temp_num<<=1;
            }
        ret[amount_of_bits[i].0 as usize]=(amount_of_bits[i].1,temp_num);
    }
    (amount_of_bits[255].1,ret)
}

/*pub fn apply_huffman_codes_to_data( input_data : &Vec<u8>,
                                         codes : &[(u8,u64);256],
                                      data_out : &mut super::bitslice::BitVec
)
{
    //make list of actual huffman codes based on the amount of bits
    for byte in input_data
    {
        let mut amount_of_remaining_bits = codes[*byte as usize].0;
        let mut divup = amount_of_remaining_bits/8;
        if amount_of_remaining_bits%8>0{divup+=1;};
        let mut curr_bit = 56/*/amount_of_remaining_bits/8*8*/;
        //TODO process first byte, then loop over remaining bytes
        let significant_bytes=&codes[*byte as usize].1.to_be_bytes()[((8-divup) as usize)..8];
        for &byte_part in significant_bytes
        {
            
            match curr_bit.cmp(&amount_of_remaining_bits) {
                Ordering::Less => {
                    
                    data_out.insert_bits(amount_of_remaining_bits%8, byte_part);
                },
                Ordering::Greater => {

                },
                Ordering::Equal => {
                    data_out.insert_bits(8, byte_part);
                },
            }
            curr_bit-=8;
        }
    }
}*/