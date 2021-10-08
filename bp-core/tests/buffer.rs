use bp_core::utils::buffer::*;
use bytes::Bytes;

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
fn test_num_to_buf_be() {
    let buf = num_to_buf_be(0x010203, 4);

    assert_eq!(buf, Bytes::from_static(&[0x00, 0x01, 0x02, 0x03]));
}

#[test]
fn test_num_to_buf_le() {
    let buf = num_to_buf_le(0xf0ffffffffffffffffffffff, 12);

    assert_eq!(
        buf,
        Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0])
    );
}
