use bytes::{BufMut, Bytes, BytesMut};
use std::cmp::min;

pub fn get_chunks(buf: Bytes, len: usize) -> Vec<Bytes> {
    let mut chunks = vec![];
    let mut ptr = 0;

    if len == 0 {
        panic!("len should be greater than 0");
    }

    loop {
        let end = min(buf.len(), ptr + len);
        let chunk = buf.slice(ptr..end);
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

#[test]
fn test_get_chunks() {
    let data = Bytes::from_static(&[1, 2, 3, 4, 5]);
    let chunks = get_chunks(data, 2);

    assert_eq!(chunks[0], Bytes::from_static(&[1, 2]));
    assert_eq!(chunks[1], Bytes::from_static(&[3, 4]));
    assert_eq!(chunks[2], Bytes::from_static(&[5]));
}

#[test]
#[should_panic]
fn test_get_chunks_panic() {
    let data = Bytes::from_static(&[1, 2, 3, 4, 5]);
    get_chunks(data, 0);
}

#[test]
fn test_num_to_buf_le() {
    let buf = num_to_buf_le(0xf0ffffffffffffffffffffff, 12);

    assert_eq!(
        buf,
        Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0])
    );
}
