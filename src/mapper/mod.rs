#[path = "ElectribeToInternalMapper.rs"]
pub mod electribetointernal;

#[path = "LiveToInternalMapper.rs"]
pub mod livetointernal;

#[path = "InternalToNoteMapper.rs"]
pub mod internaltonote;

#[path = "electribetointernal/MappingContext.rs"]
mod electribetointernalmappingcontext;

#[cfg(test)]
#[path = "ElectribeToInternalMapperTest.rs"]
mod electribetointernalmappertest;

#[path = "electribetointernal/StepTimeline.rs"]
mod electribesteptimeline;

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
