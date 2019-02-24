//! Safe wrappers around interpreter intrinsics.

use crate::types::*;

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

mod eth {
    extern "C" {
        /// Direct/classic call. Corresponds to "CALL" opcode in EVM
        pub fn ccall(
            gas: i64,
            address: *const u8,
            val_ptr: *const u8,
            input_ptr: *const u8,
            input_len: u32,
            result_ptr: *mut u8,
            result_len: u32,
        ) -> i32;

        /// Delegate call. Corresponds to "CALLCODE" opcode in EVM
        pub fn dcall(
            gas: i64,
            address: *const u8,
            input_ptr: *const u8,
            input_len: u32,
            result_ptr: *mut u8,
            result_len: u32,
        ) -> i32;

        /// Static call. Corresponds to "STACICCALL" opcode in EVM
        pub fn scall(
            gas: i64,
            address: *const u8,
            input_ptr: *const u8,
            input_len: u32,
            result_ptr: *mut u8,
            result_len: u32,
        ) -> i32;

        // blockchain functions
        pub fn address(dest: *mut u8);
        pub fn balance(address: *const u8, dest: *mut u8);
        pub fn blockhash(number: i64, dest: *mut u8);
        pub fn blocknumber() -> i64;
        pub fn coinbase(dest: *mut u8);
        pub fn create(
            endowment: *const u8,
            code_ptr: *const u8,
            code_len: u32,
            result_ptr: *mut u8,
        ) -> i32;
        pub fn create2(
            endowment: *const u8,
            salt: *const u8,
            code_ptr: *const u8,
            code_len: u32,
            result_ptr: *mut u8,
        ) -> i32;
        pub fn difficulty(dest: *mut u8);
        pub fn elog(topic_ptr: *const u8, topic_count: u32, data_ptr: *const u8, data_len: u32);
        pub fn fetch_input(dst: *mut u8);
        pub fn gasleft() -> i64;
        pub fn gaslimit(dest: *mut u8);
        pub fn input_length() -> u32;
        pub fn origin(dest: *mut u8);
        pub fn ret(ptr: *const u8, len: u32) -> !;
        pub fn sender(dest: *mut u8);
        pub fn suicide(refund: *const u8) -> !;
        pub fn timestamp() -> i64;
        pub fn value(dest: *mut u8);
    }
}

extern "C" {
    // oasis platform functions
    fn storage_read(key: *const u8, dst: *mut u8);
    fn storage_write(key: *const u8, src: *const u8);
}

/// Halt execution and register account for deletion.
///
/// Value of the current account will be tranfered to `refund` address.
pub fn suicide(refund: &Address) -> ! {
    unsafe {
        eth::suicide(refund.as_ptr());
    }
}

/// Get balance of the given account.
///
/// If an account is not registered in the chain yet,
/// it is considered as an account with `balance = 0`.
pub fn balance(address: &Address) -> U256 {
    unsafe { fetch_u256(|x| eth::balance(address.as_ptr(), x)) }
}

/// Create a new account with the given code
///
/// # Errors
///
/// Returns [`Error`] in case contract constructor failed.
///
/// [`Error`]: struct.Error.html
pub fn create(endowment: U256, code: &[u8]) -> Result<Address, Error> {
    let mut endowment_arr = [0u8; 32];
    endowment.to_big_endian(&mut endowment_arr);
    let mut result = Address::zero();
    unsafe {
        if eth::create(
            endowment_arr.as_ptr(),
            code.as_ptr(),
            code.len() as u32,
            (&mut result).as_mut_ptr(),
        ) == 0
        {
            Ok(result)
        } else {
            Err(Error)
        }
    }
}

/// Create a new account with the given code and salt.
///
/// # Errors
///
/// Returns [`Error`] in case contract constructor failed.
///
/// [`Error`]: struct.Error.html
pub fn create2(endowment: U256, salt: H256, code: &[u8]) -> Result<Address, Error> {
    let mut endowment_arr = [0u8; 32];
    endowment.to_big_endian(&mut endowment_arr);
    let mut result = Address::zero();
    unsafe {
        if eth::create2(
            endowment_arr.as_ptr(),
            salt.as_ptr(),
            code.as_ptr(),
            code.len() as u32,
            (&mut result).as_mut_ptr(),
        ) == 0
        {
            Ok(result)
        } else {
            Err(Error)
        }
    }
}

///	Message-call into an account
///
///	# Arguments:
///	* `gas`- a gas limit for a call. A call execution will halt if call exceed this amount
/// * `address` - an address of contract to send a call
/// * `value` - a value in Wei to send with a call
/// * `input` - a data to send with a call
/// * `result` - a mutable reference to be filled with a result data
///
///	# Returns:
///
/// Call is succeed if it returns `Result::Ok(())`
/// If call returns `Result::Err(Error)` it means tha call was failed due to execution halting
pub fn call(
    gas: u64,
    address: &Address,
    value: U256,
    input: &[u8],
    result: &mut [u8],
) -> Result<(), Error> {
    let mut value_arr = [0u8; 32];
    value.to_big_endian(&mut value_arr);
    unsafe {
        if eth::ccall(
            gas as i64,
            address.as_ptr(),
            value_arr.as_ptr(),
            input.as_ptr(),
            input.len() as u32,
            result.as_mut_ptr(),
            result.len() as u32,
        ) == 0
        {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/// Like [`call`], but with code at the given `address`
///
/// Effectively this function is like calling current account but with
/// different code (i.e. like `DELEGATECALL` EVM instruction).
///
/// [`call`]: fn.call.html
pub fn call_code(
    gas: u64,
    address: &Address,
    input: &[u8],
    result: &mut [u8],
) -> Result<(), Error> {
    unsafe {
        if eth::dcall(
            gas as i64,
            address.as_ptr(),
            input.as_ptr(),
            input.len() as u32,
            result.as_mut_ptr(),
            result.len() as u32,
        ) == 0
        {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/// Like [`call`], but this call and any of it's subcalls are disallowed to modify any storage.
///
/// It will return an error in this case.
///
/// [`call`]: fn.call.html
pub fn static_call(
    gas: u64,
    address: &Address,
    input: &[u8],
    result: &mut [u8],
) -> Result<(), Error> {
    unsafe {
        if eth::scall(
            gas as i64,
            address.as_ptr(),
            input.as_ptr(),
            input.len() as u32,
            result.as_mut_ptr(),
            result.len() as u32,
        ) == 0
        {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/// Returns hash of the given block or H256::zero()
///
/// Only works for 256 most recent blocks excluding current
/// Returns H256::zero() in case of failure
pub fn block_hash(block_number: u64) -> H256 {
    let mut res = H256::zero();
    unsafe { eth::blockhash(block_number as i64, res.as_mut_ptr()) }
    res
}

/// Get the current block’s beneficiary address (the current miner account address)
pub fn coinbase() -> Address {
    unsafe { fetch_address(|x| eth::coinbase(x)) }
}

/// Get the block's timestamp
///
/// It can be viewed as an output of Unix's `time()` function at
/// current block's inception.
pub fn timestamp() -> u64 {
    unsafe { eth::timestamp() as u64 }
}

/// Get the block's number
///
/// This value represents number of ancestor blocks.
/// The genesis block has a number of zero.
pub fn block_number() -> u64 {
    unsafe { eth::blocknumber() as u64 }
}

/// Get the block's difficulty.
pub fn difficulty() -> U256 {
    unsafe { fetch_u256(|x| eth::difficulty(x)) }
}

/// Get the block's gas limit.
pub fn gas_limit() -> U256 {
    unsafe { fetch_u256(|x| eth::gaslimit(x)) }
}

/// Get amount of gas left.
pub fn gas_left() -> u64 {
    unsafe { eth::gasleft() as u64 }
}

/// Get caller address
///
/// This is the address of the account that is directly responsible for this execution.
/// Use [`origin`] to get an address of external account - an original initiator of a transaction
pub fn sender() -> Address {
    unsafe { fetch_address(|x| eth::sender(x)) }
}

/// Get execution origination address
///
/// This is the sender of original transaction.
/// It could be only external account, not a contract
pub fn origin() -> Address {
    unsafe { fetch_address(|x| eth::origin(x)) }
}

/// Get deposited value by the instruction/transaction responsible for this execution.
pub fn value() -> U256 {
    unsafe { fetch_u256(|x| eth::value(x)) }
}

/// Get address of currently executing account
pub fn address() -> Address {
    unsafe { fetch_address(|x| eth::address(x)) }
}

/// Creates log entry with given topics and data.
///
/// There could be only up to 4 topics.
///
/// # Panics
///
/// If `topics` contains more than 4 elements then this function will trap.
pub fn log(topics: &[H256], data: &[u8]) {
    unsafe {
        eth::elog(
            topics.as_ptr() as *const u8,
            topics.len() as u32,
            data.as_ptr(),
            data.len() as u32,
        );
    }
}

/// Allocates and requests [`call`] arguments (input)
///
/// Input data comes either with external transaction or from [`call`] input value.
pub fn input() -> Vec<u8> {
    let len = unsafe { eth::input_length() };

    match len {
        0 => Vec::new(),
        non_zero => {
            let mut data = Vec::with_capacity(non_zero as usize);
            unsafe {
                data.set_len(non_zero as usize);
                eth::fetch_input(data.as_mut_ptr());
            }
            data
        }
    }
}

/// Sets a [`call`] return value
///
/// Pass return data to the runtime. Runtime SHOULD trap the execution.
///
pub fn ret(data: &[u8]) -> ! {
    unsafe {
        eth::ret(data.as_ptr(), data.len() as u32);
    }
}

/// Performs read from the storage.
pub fn read(key: &H256) -> [u8; 32] {
    let mut dst = [0u8; 32];
    unsafe {
        storage_read(key.as_ptr(), dst.as_mut_ptr());
    }
    dst
}

/// Performs write to the storage
pub fn write(key: &H256, val: &[u8; 32]) {
    unsafe {
        storage_write(key.as_ptr(), val.as_ptr());
    }
}

unsafe fn fetch_address<F: Fn(*mut u8)>(f: F) -> Address {
    let mut res = Address::zero();
    f(res.as_mut_ptr());
    res
}

unsafe fn fetch_u256<F: Fn(*mut u8)>(f: F) -> U256 {
    let mut res = [0u8; 32];
    f(res.as_mut_ptr());
    U256::from_big_endian(&res)
}