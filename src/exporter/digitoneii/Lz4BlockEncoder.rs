//! LZ4 block encoder for Digitone project images.
//!
//! The encoding is a valid linked-block chain, not a byte copy of Elektron's
//! compressor. Many encodings decode to the same image. Callers compare decoded
//! images, not compressed bytes.

const BLOCK_SIZE: usize = 32_768;
const MATCH_SEARCH_LIMIT: usize = 12;
const LAST_LITERALS: usize = 5;
const MIN_MATCH: usize = 4;
const MAX_OFFSET: usize = 0xFFFF;
const HASH_SIZE: usize = 1 << 16;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Lz4BlockEncoder;

impl Lz4BlockEncoder {
    pub(crate) fn encode(self, image: &[u8]) -> Vec<u8> {
        let mut chain = Vec::new();
        let mut table = vec![None; HASH_SIZE];
        let mut start = 0;
        while start < image.len() {
            let end = (start + BLOCK_SIZE).min(image.len());
            let block = self.encodeBlock(image, start, end, &mut table);
            chain.extend_from_slice(
                &u32::try_from(block.len())
                    .expect("block fits in u32")
                    .to_be_bytes(),
            );
            chain.extend_from_slice(&block);
            start = end;
        }
        chain.extend_from_slice(&0u32.to_be_bytes());
        chain
    }

    fn encodeBlock(
        self,
        image: &[u8],
        start: usize,
        end: usize,
        table: &mut [Option<usize>],
    ) -> Vec<u8> {
        let mut out = Vec::new();
        if end - start <= MATCH_SEARCH_LIMIT {
            self.writeLiterals(&mut out, image, start, end);
            return out;
        }
        let matchLimit = end - MATCH_SEARCH_LIMIT;
        let mut anchor = start;
        let mut at = start;
        while at < matchLimit {
            let hash = hash4(image, at);
            let candidate = table[hash];
            table[hash] = Some(at);
            let Some(matched) = self.matchAt(image, candidate, at, end) else {
                at += 1;
                continue;
            };
            self.writeSequence(&mut out, image, anchor, at, matched.offset, matched.length);
            at += matched.length;
            anchor = at;
            let mut indexed = start.max(at.saturating_sub(matched.length - 1));
            while indexed < at && indexed < matchLimit {
                table[hash4(image, indexed)] = Some(indexed);
                indexed += 1;
            }
        }
        self.writeLiterals(&mut out, image, anchor, end);
        out
    }

    fn matchAt(
        self,
        image: &[u8],
        candidate: Option<usize>,
        at: usize,
        end: usize,
    ) -> Option<Match> {
        let candidate = candidate?;
        if candidate >= at {
            return None;
        }
        let offset = at - candidate;
        if offset == 0 || offset > MAX_OFFSET || candidate + MIN_MATCH > image.len() {
            return None;
        }
        if image[candidate..candidate + MIN_MATCH] != image[at..at + MIN_MATCH] {
            return None;
        }
        let extendLimit = end - LAST_LITERALS;
        let mut length = MIN_MATCH;
        while at + length < extendLimit
            && candidate + length < image.len()
            && image[candidate + length] == image[at + length]
        {
            length += 1;
        }
        Some(Match { offset, length })
    }

    fn writeSequence(
        self,
        out: &mut Vec<u8>,
        image: &[u8],
        anchor: usize,
        at: usize,
        offset: usize,
        length: usize,
    ) {
        let literalLength = at - anchor;
        let matchExtra = length - MIN_MATCH;
        let literalNib = literalLength.min(15);
        let matchNib = matchExtra.min(15);
        out.push(u8::try_from((literalNib << 4) | matchNib).expect("nibbles fit in a byte"));
        if literalLength >= 15 {
            writeExtension(out, literalLength - 15);
        }
        out.extend_from_slice(&image[anchor..at]);
        out.push(u8::try_from(offset & 0xFF).expect("low offset byte"));
        out.push(u8::try_from(offset >> 8).expect("high offset byte"));
        if matchExtra >= 15 {
            writeExtension(out, matchExtra - 15);
        }
    }

    fn writeLiterals(self, out: &mut Vec<u8>, image: &[u8], anchor: usize, end: usize) {
        let literalLength = end - anchor;
        let literalNib = literalLength.min(15);
        out.push(u8::try_from(literalNib << 4).expect("nibble fits in a byte"));
        if literalLength >= 15 {
            writeExtension(out, literalLength - 15);
        }
        out.extend_from_slice(&image[anchor..end]);
    }
}

struct Match {
    offset: usize,
    length: usize,
}

fn hash4(image: &[u8], at: usize) -> usize {
    let word = u32::from_le_bytes([image[at], image[at + 1], image[at + 2], image[at + 3]]);
    usize::try_from(word.wrapping_mul(2_654_435_761) >> 16).expect("hash fits")
}

fn writeExtension(out: &mut Vec<u8>, mut value: usize) {
    while value >= 255 {
        out.push(255);
        value -= 255;
    }
    out.push(u8::try_from(value).expect("remaining extension fits in a byte"));
}
