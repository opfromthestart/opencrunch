use quote::quote;
use proc_macro2::{TokenStream, TokenTree, Group};

/// Adds a field called strings that keeps track of all the 
#[proc_macro_attribute]
pub fn crunch_fill(_attr: proc_macro::TokenStream, s: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree : TokenStream = s.into();
    let mut vtree = tree.into_iter().collect::<Vec<_>>();

    let body = vtree.last().expect("Struct expected").clone();
    let on_comma = on_punct(',');

    let struct_pos = vtree.iter().position(|x| {
        match x {
            TokenTree::Ident(i) => i.to_string() == "struct",
            _ => false
        }
    }).unwrap();
    let struct_name = vtree.get(struct_pos+1).unwrap().clone();

    let (args, names) = match body {
        proc_macro2::TokenTree::Group(g) => {
            let s : Vec<_> = g.stream().into_iter().collect();
            let mut end_comma = false;
            let names : Vec<_> = s.split(on_comma).filter_map(|slice| match slice {
                [name, ..] => Some(name.clone()),
                [] => {end_comma = true; None}
            }).collect();
            let vals = names.len()+1; // another for the error
            let fill = if end_comma {
                quote!(
                    /// #names
                    strings: [String; #vals],
                )
            }
            else {
                quote!(
                    /// #names
                    ,strings: [String; #vals],
                )
            };
            let mut new_fields = g.stream();
            new_fields.extend(fill);
            (TokenTree::Group(Group::new(g.delimiter(), new_fields)), names)
        },
        _ => panic!("Not usable on unit structs."),
    };
    *(vtree.last_mut().unwrap()) = args;

    let vfill: TokenStream = names.iter().enumerate().flat_map(|(i,n)| {
        quote!(if let Ok(val) = self.strings[#i].parse() {
            self.#n = val;
        })
    }).collect();

    let vfill_impl = quote!(
        impl #struct_name {
            fn vfill(&mut self) {
                #vfill
            }
        }
    );

    let tree : TokenStream = vtree.into_iter().chain(vfill_impl).collect();
    tree.into_iter().collect::<TokenStream>().into()
}

fn on_punct(c: char) -> Box<dyn Fn(&TokenTree) -> bool> {
    Box::new(move |x| {
        if let TokenTree::Punct(p) = x {
            p.as_char() == c
        }
        else {
            false
        }
    })
}