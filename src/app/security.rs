use core::str;

use aes::{
    cipher::{generic_array::GenericArray, typenum, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};

const BLOCK_SIZE: usize = 16;

pub fn encrypt(message: &str, key: &[u8; 16]) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut data = message.as_bytes().to_vec();

    let pad_len = BLOCK_SIZE - (data.len() % BLOCK_SIZE);
    data.extend(std::iter::repeat(pad_len as u8).take(pad_len));

    let mut blocks: Vec<GenericArray<u8, typenum::U16>> = data
        .chunks_exact(BLOCK_SIZE)
        .map(|chunk| *GenericArray::from_slice(chunk))
        .collect();

    cipher.encrypt_blocks(&mut blocks);

    blocks
        .iter()
        .flat_map(|block| block.iter().copied())
        .collect()
}

pub fn decrypt(msg: Vec<u8>, key: &[u8; 16]) -> String {
    let cipher = Aes128::new(GenericArray::from_slice(key));

    let mut blocks: Vec<GenericArray<u8, typenum::U16>> = msg
        .chunks_exact(16)
        .map(|chunk| *GenericArray::from_slice(chunk))
        .collect();
    cipher.decrypt_blocks(&mut blocks);
    let bytes: Vec<u8> = blocks
        .iter()
        .flat_map(|block| block.iter().copied())
        .collect();
     
    let len = bytes.len() - *bytes.last().unwrap() as usize;

    str::from_utf8(&bytes[..len]).unwrap().to_string()
}
