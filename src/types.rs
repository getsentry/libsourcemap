#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IndexItem {
    pub line: u32,
    pub col: u32,
    pub name_id: u32,
    pub src_id: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct StringMarker {
    pub pos: u32,
    pub len: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct MapHead {
    pub index_size: u32,
    pub names_start: u32,
    pub names_count: u32,
    pub sources_start: u32,
    pub sources_count: u32,
}
