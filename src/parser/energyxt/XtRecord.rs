/// One record of an energyXT record stream: a key and its payload.
#[derive(Debug, Clone, Copy)]
pub(super) struct XtRecord<'a> {
    pub(super) key: &'a str,
    pub(super) payload: &'a [u8],
}
