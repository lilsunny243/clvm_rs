use core::slice;
use std::ptr;

use clvmr::{
    allocator::Allocator,
    chia_dialect::{ChiaDialect, NO_NEG_DIV},
    dialect::Dialect,
    node::Node,
    serialize::{node_from_bytes, node_to_bytes},
};

#[repr(C)]
pub struct Ret {
    cost: u64,
    ret: *const u8,
    ret_len: usize,
    err: *const u8,
    err_len: usize,
}

#[no_mangle]
pub extern "C" fn eval_op(
    op_code_buf: *mut u8,
    op_code_len: usize,
    args_buf: *mut u8,
    args_len: usize,
    block_height: u32,
) -> Ret {
    let mut a = Allocator::new();
    let mut flags: u32 = 0;
    if block_height >= 2300000 {
        flags = NO_NEG_DIV;
    }

    let args = unsafe { slice::from_raw_parts_mut(args_buf, args_len) };
    let op_code = unsafe { slice::from_raw_parts_mut(op_code_buf, op_code_len) };

    let op_node = a.new_atom(op_code).unwrap();
    let args_node = node_from_bytes(&mut a, args).unwrap();

    let d = ChiaDialect::new(flags);
    match d.op(&mut a, op_node, args_node, 18446744073709551615) {
        Err(err) => {
            let err_len = err.1.len();
            let mut err_vec = err.1.as_bytes().to_vec();
            let err_vec_ptr = err_vec.as_mut_ptr();

            std::mem::forget(err_vec);

            return Ret {
                cost: 0,
                ret: ptr::null_mut(),
                ret_len: 0,
                err: err_vec_ptr,
                err_len: err_len,
            };
        }
        Ok(r) => {
            let ret_blob = node_to_bytes(&Node::new(&a, r.1)).unwrap();
            let ret_len = ret_blob.len();

            let mut ret_vec = ret_blob.to_vec();
            let ret_vec_ptr = ret_vec.as_mut_ptr();

            std::mem::forget(ret_vec);

            return Ret {
                cost: r.0,
                ret: ret_vec_ptr,
                ret_len: ret_len,
                err: ptr::null_mut(),
                err_len: 0,
            };
        }
    }
}
