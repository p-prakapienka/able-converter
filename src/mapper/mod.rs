#[path = "LiveToInternalMapper.rs"]
pub mod livetointernal;

#[path = "InternalToNoteMapper.rs"]
pub mod internaltonote;

#[cfg(test)]
#[path = "InternalToNoteMapperTest.rs"]
mod internaltonotemappertest;

#[path = "internaltonote/MappingContext.rs"]
mod internaltonotemappingcontext;

#[path = "livetointernal/MappingContext.rs"]
mod livetointernalmappingcontext;

#[cfg(test)]
#[path = "LiveToInternalMapperTest.rs"]
mod livetointernalmappertest;

#[path = "EnergyXtToInternalMapper.rs"]
pub mod energyxttointernal;

#[path = "energyxttointernal/MappingContext.rs"]
mod energyxttointernalmappingcontext;

#[cfg(test)]
#[path = "EnergyXtToInternalMapperTest.rs"]
mod energyxttointernalmappertest;

#[path = "ArrangementToSessionMapper.rs"]
pub mod arrangementtosession;

#[path = "arrangementtosession/LayoutSection.rs"]
mod arrangementtosessionlayoutsection;

#[path = "arrangementtosession/LayoutUnit.rs"]
mod arrangementtosessionlayoutunit;

#[path = "arrangementtosession/MappingContext.rs"]
mod arrangementtosessionmappingcontext;

#[cfg(test)]
#[path = "ArrangementToSessionMapperTest.rs"]
mod arrangementtosessionmappertest;
