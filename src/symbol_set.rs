#[derive(PartialEq,Copy,Clone)]
pub struct RGBSet
{
    pub(crate) red:Option<u8>,
    pub(crate) green:Option<u8>,
    pub(crate) blue:Option<u8>
}
#[derive(PartialEq,Copy,Clone)]

pub struct DiffSet
{
    pub(crate) red_diff : Option<u8>,
    pub(crate) green_diff : Option<u8>,
    pub(crate) blue_diff : Option<u8>
}

#[derive(PartialEq,Copy,Clone)]
pub struct LumaSet
{
    pub(crate) back_ref : u8,
    pub(crate) red_diff : Option<u8>,
    pub(crate) green_diff : u8,
    pub(crate) blue_diff : Option<u8>
}

#[derive(PartialEq,Clone)]
pub struct RunSet
{
    pub(crate) runs : Vec<(u8,u8)>
}

#[derive(PartialEq,Clone)]
pub enum SymbolSet
{
    Rgbset(RGBSet),
    DiffSet(DiffSet),
    LumaSet(LumaSet),
    RunSet(RunSet)
}



pub struct SymbolSetGroup
{
    pub choice1 : Option<SymbolSet>,
    pub choice2 : Option<SymbolSet>,
    pub choice3 : Option<SymbolSet>
}
pub struct SymbolSetsMap//<'a>
{
    symbolsets:Vec<[Option<SymbolSet>;3]>,
    //preferred_symbolsets:Vec<&'a SymbolSetGroup>
}

/*impl SymbolSetsMap<'_>
{
    fn add_
}*/