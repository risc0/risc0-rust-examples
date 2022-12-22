#![no_main]
#![no_std]  // std support is experimental, but you can remove this to try it

risc0_zkvm::guest::entry!(main);

#[link(name = "chibicc")]
extern {
    fn string_to_bin(input: *const u8, output_buf_len: u32) -> u32;
}

pub fn call_compile(input: &[u8], output_buf_len: u32) -> u32 {
    unsafe {
        string_to_bin(input.as_ptr(), output_buf_len);
    }
    return 1;

}

pub fn main() {
    // TODO: Implement your guest code here
    call_compile("int main() { return 0; }".as_bytes(), 40000);
}
