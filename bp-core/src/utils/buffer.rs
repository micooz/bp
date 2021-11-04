use std::cmp::min;

use bytes::{BufMut, Bytes, BytesMut};

pub fn get_chunks(arr: Bytes, len: usize) -> Vec<Bytes> {
    let mut chunks = vec![];
    let mut ptr = 0;

    assert!(len != 0, "len should be greater than 0");

    loop {
        let end = min(arr.len(), ptr + len);
        let chunk = arr.slice(ptr..end);
        let chunk_len = chunk.len();

        chunks.push(chunk);

        if chunk_len < len {
            break;
        }

        ptr += len;
    }

    chunks
}

pub fn num_to_buf_be(num: u64, nbytes: usize) -> Bytes {
    let mut buf = BytesMut::with_capacity(nbytes);
    buf.put_uint(num, nbytes);
    buf.freeze()
}

pub fn num_to_buf_le(num: u128, nbytes: usize) -> Bytes {
    let mut buf = BytesMut::with_capacity(nbytes);
    buf.put_u128_le(num);
    buf.freeze().slice(0..12)
}
