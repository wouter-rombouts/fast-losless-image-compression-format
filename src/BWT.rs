
pub fn bwt_transform( input_data : &[u8])
-> (Vec<u8>,usize)
{
    //start with array of indices of the input data
    let mut transformed_data : Vec<usize>=(0..input_data.len()).collect();
    //TODO use unique symbols similar to inverse transform
    //sort data by symbol, and if equal get next symbol in the input data to see the order, return index of this
    //TODO optimize: merge same symbol subsets, as it can be taxing to performance
    let mut ret_index=0;
    transformed_data.sort_by(|a, b| {
        let mut i =0;
        while i<input_data.len() && input_data[(a+i)%input_data.len()]==input_data[(b+i)%input_data.len()]
        {

            i+=1;
        }
        input_data[(a+i)%input_data.len()].partial_cmp(&input_data[(b+i)%input_data.len()]).unwrap()
    });
    //TODO add length-1 to get the last symbol of this rotation that starts with the current symbol in the sorted list.
    let mut return_data : Vec<u8>=vec![0;input_data.len()];
    for i in 0..return_data.len()
    {
        return_data[i]=input_data[if transformed_data[i]==0{ret_index=i;input_data.len()}else{transformed_data[i]}-1];
    }
    
    //TODO return index of unique symbol
    (return_data,ret_index)
}
//let input be the occurrences table instead of raw data
pub fn bwt_inverse_transform( transformed_data : &[u8], offset : u8 )
-> Vec<u8>
{
    let mut original_data=vec![0;transformed_data.len()];
    //get occurrences for each symbol
    //let mut orig_data_occurences : Vec<(u8,usize)>=(0..u8::MAX).map(|i| (i,0)).collect();
    let mut transf_data_occurences : Vec<usize>=vec![0;256];
    for el in transformed_data
    {
        transf_data_occurences[*el as usize]+=1;
    }
    let mut ordered_data_occurences=transf_data_occurences.clone();
    ordered_data_occurences.sort_unstable();
    //TODO  subtract occurrences when found.
    /*let mut ordered_data=transformed_data.to_vec();
    ordered_data.sort();*/
    let next_pos=offset;

    for i in (0..transformed_data.len())
    
    {
        original_data[transformed_data.len()-i-1]=transf_data_occurences[next_pos as usize] as u8;
        //next_pos=ordered_data
    }
    original_data
}

pub fn bwt_basic_transform( input_data : &[u8])
-> Vec<u8>
{   
    let mut positions_cache=vec![0;input_data.len()];
    let mut temp_pos_cache_list=vec![vec![0;input_data.len()];input_data.len()];
    //initial list
    for i in 0..input_data.len()
    {
        temp_pos_cache_list[0][i]=input_data[i];
    }
    //rotations
    for i in 1..input_data.len()
    {
        for j in 0..input_data.len()
        {
            temp_pos_cache_list[i][j]=temp_pos_cache_list[0][(j+i)%input_data.len()];
        } 
    }

    //sort by sum of colors?
    temp_pos_cache_list.sort();
    //take last column
    for i in 0..input_data.len()
    {
        positions_cache[i]=temp_pos_cache_list[i][input_data.len()-1];
    }
    positions_cache
    //output index of correct begin
    
}

pub fn rotate_list( mylist : &mut [u8], amount : usize ){
    let mut temp_bytes : Vec<u8> = vec![0;mylist.len()];
    temp_bytes.copy_from_slice(mylist);
    for i in 0..mylist.len(){
        mylist[i]=temp_bytes[(i+mylist.len()-amount)%mylist.len()];
    }

}

mod tests {
    #[test]
    fn bwt_transform_check() {
        let data =b"morecharactersinasentenceandsomemoreandthanandonemoremoreoreo";
        dbg!(data);
        let mut transformed_data=super::bwt_transform(data);
        dbg!(&transformed_data);
        
        let basic_transformed_data=super::bwt_basic_transform(data);
        dbg!(&basic_transformed_data);
        debug_assert!(transformed_data.0==basic_transformed_data);
        super::rotate_list( &mut transformed_data.0, transformed_data.1 );
        transformed_data=super::bwt_transform(&transformed_data.0);
        dbg!(&transformed_data);

        
    }
    use itertools::Itertools;
    #[test]
    fn bwt_bytes_check() {
        let mut data=[[0;8];256];
        let mut outputddata=vec![vec![0u8;8];256];
        for i in 0..256
        {
            for j in 0..8
            {
                data[i][j]=((i&(1<<j))>>j) as u8;
            }
        }
        for i in 0..256
        {
            //dbg!(data[i]);
            let mut transformed_data=super::bwt_transform(&data[i]);
            println!("{},{},{},{},{},{},{},{},offset:{}",transformed_data.0[0],transformed_data.0[1],transformed_data.0[2],transformed_data.0[3],transformed_data.0[4],transformed_data.0[5],transformed_data.0[6],transformed_data.0[7],transformed_data.1);
            outputddata[i]=transformed_data.0;
            //dbg!(&transformed_data.0);
        }
        let uniques=(outputddata).iter().unique().sorted();
        let cnt=uniques.clone().count();
        for el in uniques
        {
            println!("{},{},{},{},{},{},{},{}",el[0],el[1],el[2],el[3],el[4],el[5],el[6],el[7]);

            let cnt_nr=outputddata.iter().filter(|&x|x == el).count();
            dbg!(cnt_nr);
        }
        dbg!(cnt);
        
        /*let basic_transformed_data=super::bwt_basic_transform(data);
        dbg!(&basic_transformed_data);
        debug_assert!(transformed_data.0==basic_transformed_data);
        super::rotate_list( &mut transformed_data.0, transformed_data.1 );
        transformed_data=super::bwt_transform(&transformed_data.0);
        dbg!(&transformed_data);*/

        
    }
}