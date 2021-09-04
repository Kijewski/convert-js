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
use std::env::var;
use std::fs::{OpenOptions, create_dir_all, remove_file};
use std::io::{Read, Write};
use std::path::Path;

use blake2::{Blake2b, Digest};
use fd_lock::RwLock;
use litrs::StringLit;
use proc_macro::{Literal, TokenStream, TokenTree};

/// TODO: doc
fn compile_error(msg: &str) -> TokenStream {
    format!("::core::compile_error!({:?})", msg)
        .parse()
        .expect("Could not parse error message")
}

fn token_stream(converted: String) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::string(&converted)))
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
    let js_code = js_code.value().as_bytes();

    let mut hash = Blake2b::new();
    hash.update(js_code);
    let hash = hash.finalize();
    let hash = base64::encode_config(&hash[0..32], base64::URL_SAFE_NO_PAD);
    let hash = format!("{}-{}.js", hash, js_code.len());
    let (a, b) = hash.split_at(4);
    let (b, c) = b.split_at(4);

    let tmpdir = match var("CARGO_TARGET_TMPDIR") {
        Ok(tmpdir) => tmpdir,
        _ => return compile_error("Environment variable CARGO_TARGET_TMPDIR is not set."),
    };
    let cachefile = Path::new(&tmpdir).join(format!("convert-js/{}/{}/{}", a, b, c));
    if let Err(err) = create_dir_all(cachefile.parent().unwrap()) {
        dbg!(err);
        return compile_error(&format!("Could not create tempdir."));
    }

    for _ in 0..10 {
        match OpenOptions::new().read(true).open(&cachefile) {
            Err(_) => match {
                OpenOptions::new().create(true).create_new(true).write(true).open(&cachefile)
             } {
                Err(err) => {
                    dbg!(err);
                    continue
                }
                Ok(file) => {
                    let mut file = RwLock::new(file);
                    let mut file = match file.write() {
                        Ok(file) => file,
                        Err(err) => {
                            dbg!(err);
                            let _ = remove_file(&cachefile);
                            continue;
                        }
                    };
                    let converted = match convert_js_impl::convert_js(js_code) {
                        Ok(converted) => converted,
                        Err(err) => {
                            dbg!(err);
                            return compile_error("Could not convert JavaScript code.");
                        }
                    };
                    match file.write_all(converted.as_bytes()) {
                        Ok(_) => return token_stream(converted),
                        Err(err) => {
                            dbg!(err);
                            let _ = remove_file(&cachefile);
                            return compile_error(&format!("Could not write cache file."));
                        }
                    }
                }
            },
            Ok(file) => {
                let mut converted = String::new();
                let mut file = RwLock::new(file);
                let mut file = match file.write() {
                    Err(err) => {
                        dbg!(err);
                        continue
                    }
                    Ok(file) => file,
                };
                match file.read_to_string(&mut converted) {
                    Err(err) => {
                        dbg!(err);
                        continue
                    }
                    Ok(_) => return token_stream(converted),
                }
            }
        }
    }

    compile_error("Could not write to cache file after 10 tries.")
}
