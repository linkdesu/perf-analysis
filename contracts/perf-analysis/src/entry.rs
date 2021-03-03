use super::error::Error;
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, debug, error::SysError, syscalls};
use core::result::Result;
use das_bloom_filter::BloomFilter;
use core::convert::{TryFrom, TryInto};

pub fn main() -> Result<(), Error> {
    bloom_test()?;
    Ok(())
}

fn bloom_test() -> Result<(), Error> {
    debug!("Define bloom filter bits.");
    let v_bloom_filter = load_data(|buf, offset| syscalls::load_witness(buf, offset, 0, Source::Input)).map_err(|e| Error::from(e))?;
    let first = v_bloom_filter.get(..4).unwrap();
    let u_first = u32::from_le_bytes(first.try_into().unwrap());
    let second = v_bloom_filter.get(4..8).unwrap();
    let u_second = u32::from_le_bytes(second.try_into().unwrap());
    let bloom_filter = v_bloom_filter.get(8..).unwrap();
    
    debug!("First 10 bytes: {:?}", bloom_filter.get(..10));

    debug!("Restore bloom filter from bits: {}", u_first);
    //let bf = BloomFilter::new_with_data(u_first as u64, u_second as u64, &bloom_filter);

    debug!("Check if string is contained in filter.");
    //bf.contains(b"das.bit");

    Ok(())
}

fn load_test() -> Result<(), Error> {
    load_witnesses()?;

    Ok(())
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
