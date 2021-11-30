extern crate proc_macro;

use proc_macro::{TokenStream};
use syn::{FnArg, ItemFn, parse_macro_input};
use quote::{quote};

#[proc_macro_attribute]
pub fn circuit(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let circuit_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let body = &input_fn.block;
    let attrs = &input_fn.attrs;
    let inputs = &input_fn.sig.inputs;

    for i in inputs {
        if let FnArg::Receiver(_) = i {
            return TokenStream::from(quote! {
                compile_error!("circuits must not take a reference to self");
            });
        }
    }

    TokenStream::from(quote!{
        #(#attrs)*
        #vis fn #circuit_name() -> sunscreen_frontend_types::Context {
            use std::cell::Cell;
            use std::mem::transmute;
            use sunscreen_frontend_types::CURRENT_CTX;

            let mut context = Context::new();

            CURRENT_CTX.with(|ctx| {
                fn internal(#inputs) {
                    #body
                }

                // Transmute away the lifetime to 'static. So long as we are careful with internal()
                // panicing, this is safe because we set the context back to none before the funtion
                // returns.
                ctx.swap(&Cell::new(Some(unsafe { transmute(&context) })));
                
                let panic_res = std::panic::catch_unwind(|| {
                    internal()
                });
                
                ctx.swap(&Cell::new(None));

                if let Err(err) = panic_res {
                    std::panic::resume_unwind(err);
                }
            });

            context
        } 
    })
}
