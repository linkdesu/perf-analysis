use super::error::Error;
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, debug, error::SysError, syscalls};
use core::result::Result;
use das_bloom_filter::BloomFilter;

pub fn main() -> Result<(), Error> {
    bloom_test();
    Ok(())
}

fn bloom_test() {
    debug!("Define bloom filter bits.");
    let bloom_filter = [];

    debug!("Restore bloom filter from bits.");
    let mut bf = BloomFilter::new_with_data(239627, 17, &bloom_filter);

    debug!("Check if string is contained in filter.");
    bf.contains(b"das.bit");
}

fn load_test() {
    load_witnesses()?;
}

fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
    let mut buf = [0u8; 1000];
    match syscall(&mut buf, 0) {
        Ok(len) => Ok(buf[..len].to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            let mut data = Vec::with_capacity(actual_size);
            data.resize(actual_size, 0);
            let loaded_len = buf.len();
            data[..loaded_len].copy_from_slice(&buf);
            let len = syscall(&mut data[loaded_len..], loaded_len)?;
            debug_assert_eq!(len + loaded_len, actual_size);
            Ok(data)
        }
        Err(err) => Err(err),
    }
}

pub fn load_witnesses() -> Result<Vec<Vec<u8>>, Error> {
    let mut i = 0;
    let mut witnesses = Vec::new();
    loop {
        let data;
        let ret = load_data(|buf, offset| syscalls::load_witness(buf, offset, i, Source::Input));
        match ret {
            Ok(_data) => {
                i += 1;
                data = _data;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(Error::from(e)),
        }

        witnesses.push(data);
    }

    Ok(witnesses)
}
