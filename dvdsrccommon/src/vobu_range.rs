use crate::index::{IndexedVts, Vobu};

pub type VobuRange = (u32, u32);

#[derive(Debug)]
pub struct EVobu {
    pub i: usize,
    pub v: Vobu,
}
pub struct VobuRanger {}
impl VobuRanger {
    pub fn from(lst: &[VobuRange], vts: &IndexedVts) -> Vec<EVobu> {
        let mut voburan = Vec::new();
        for a in lst.iter().map(|e| (e.0 as usize, e.1 as usize)) {
            voburan.extend(
                vts.vobus[a.0..a.1 + 1]
                    .iter()
                    .enumerate()
                    .map(|(inner_idx, vob)| EVobu {
                        i: a.0 + inner_idx,
                        v: vob.clone(),
                    }),
            );
        }
        return voburan;
    }

    pub fn full(vts: &IndexedVts) -> Vec<EVobu> {
        return Self::from(&[(0, vts.vobus.len() as u32 - 1)], vts);
    }
}
