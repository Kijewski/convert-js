// Copyright (c) 2021 René Kijewski <rene.[SURNAME]@fu-berlin.de>
// All rights reserved.
//
// This software and the accompanying materials are made available under
// the terms of the ISC License which is available in the project root as LICENSE-ISC, AND/OR
// the terms of the MIT License which is available at in the project root as LICENSE-MIT, AND/OR
// the terms of the Apache License, Version 2.0 which is available in the project root as LICENSE-APACHE.
//
// You have to accept AT LEAST one of the aforementioned licenses to use, copy, modify, and/or distribute this software.
// At your will you may redistribute the software under the terms of only one, two, or all three of the aforementioned licenses.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! TODO: doc

use std::convert::TryFrom;

use litrs::StringLit;
use proc_macro::{Literal, TokenStream, TokenTree};

/// TODO: doc
fn compile_error(msg: &str) -> TokenStream {
    format!("::core::compile_error!({:?})", msg)
        .parse()
        .expect("Could not parse error message")
}

/// TODO: doc
#[proc_macro]
pub fn convert_js(arg: TokenStream) -> TokenStream {
    let mut arg_iter = arg.into_iter();
    let lit = match (arg_iter.next(), arg_iter.next()) {
        (Some(TokenTree::Literal(lit)), None) => lit,
        _ => return compile_error("Expected exactly one string as argument"),
    };
    let js_code = match StringLit::try_from(lit) {
        Ok(js_code) => js_code,
        _ => return compile_error("Expected exactly one string as argument"),
    };
    match convert_js_impl::convert_js(js_code.value()) {
        Ok(converted) => TokenStream::from(TokenTree::Literal(Literal::string(&converted))),
        Err(err) => compile_error(&format!("{:?}", err)),
    }
}
