use crate::index::Index;
use std::ffi::{c_char, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct SearchResultFFI {
    filename: *mut c_char,
    similarity: f32,
    snippets: *mut *mut c_char,
    snippets_count: usize,
}

#[repr(C)]
pub struct SearchResultsFFI {
    results: *mut SearchResultFFI,
    count: usize,
}

static mut GLOBAL_INDEX: Option<Index> = None;

#[no_mangle]
pub extern "C" fn initialize_search_engine(path: *const c_char) -> bool {
    println!("initialize_search_engine called");
    let c_str = unsafe {
        if path.is_null() {
            println!("path pointer is null");
            return false;
        }
        CStr::from_ptr(path)
    };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            println!("failed to convert path to string: {}", e);
            return false;
        }
    };

    println!("attempting to build Index with path : {}", path_str);
    match Index::new(path_str) {
        Ok(index) => {
            println!("successfully built index");
            unsafe {
                GLOBAL_INDEX = Some(index);
            }
            true
        }
        Err(e) => {
            println!("error building Index: {}", e);
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn perform_search(query: *const c_char) -> *mut SearchResultsFFI {
    let c_str = unsafe {
        if query.is_null() {
            return ptr::null_mut();
        }
        CStr::from_ptr(query)
    };
    let query_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let results = unsafe {
        match &mut GLOBAL_INDEX {
            Some(index) => index.search(query_str),
            None => return ptr::null_mut(),
        }
    };

    let mut ffi_results = Vec::with_capacity(results.len());

    for result in results {
        let filename = CString::new(result.filename).unwrap();

        let mut snippet_ptrs = Vec::with_capacity(result.snippets.len());
        let num_snippets = result.snippets.len();
        for snippet in result.snippets {
            let snippet_cstring = CString::new(snippet).unwrap();
            snippet_ptrs.push(snippet_cstring.into_raw());
        }

        let snippets_ptr = if !snippet_ptrs.is_empty() {
            let ptr = snippet_ptrs.as_ptr() as *mut *mut c_char;
            std::mem::forget(snippet_ptrs);
            ptr
        } else {
            ptr::null_mut()
        };

        ffi_results.push(SearchResultFFI {
            filename: filename.into_raw(),
            similarity: result.similarity,
            snippets: snippets_ptr,
            snippets_count: num_snippets,
        });
    }

    let count = ffi_results.len();
    let results_ptr = ffi_results.as_ptr() as *mut SearchResultFFI;
    std::mem::forget(ffi_results);

    let results_container = Box::new(SearchResultsFFI {
        results: results_ptr,
        count,
    });

    Box::into_raw(results_container)
}

#[no_mangle]
pub extern "C" fn free_search_results(results: *mut SearchResultsFFI) {
    if results.is_null() {
        return;
    }

    unsafe {
        let container = Box::from_raw(results);
        let results_slice = std::slice::from_raw_parts(container.results, container.count);
        for result in results_slice {
            if !result.filename.is_null() {
                let _ = CString::from_raw(result.filename);
            }

            if !result.snippets.is_null() {
                let snippets_slice =
                    std::slice::from_raw_parts_mut(result.snippets, result.snippets_count);
                for &mut snippet in snippets_slice {
                    if !snippet.is_null() {
                        let _ = CString::from_raw(snippet);
                    }
                }
                let _ = Vec::from_raw_parts(
                    result.snippets,
                    result.snippets_count,
                    result.snippets_count,
                );
            }
        }
        let _ = Vec::from_raw_parts(container.results, container.count, container.count);
    }
}
