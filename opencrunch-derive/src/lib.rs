use quote::quote;
use proc_macro2::{TokenStream, TokenTree, Group, Punct, Spacing};

/// Adds a field called strings that keeps track of all the inputs of the thing
#[proc_macro_attribute]
pub fn crunch_fill(_attr: proc_macro::TokenStream, s: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree : TokenStream = s.into();
    let mut vtree = tree.into_iter().collect::<Vec<_>>();

    let body = vtree.last().expect("Struct expected").clone();
    let on_comma = on_punct(',');

    let struct_pos = vtree.iter().position(|x| {
        match x {
            TokenTree::Ident(i) => *i == "struct",
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
            let nam : TokenStream = names.iter().flat_map(|n| [n.clone(), TokenTree::Punct(Punct::new(',', Spacing::Alone))]).collect();
            let namstr = format!("{nam} error,");
            let comm = quote!(#[doc = #namstr]);
            let fill = if end_comma {
                quote!(
                    strings: [String; #vals],
                )
            }
            else {
                quote!(
                    ,strings: [String; #vals],
                )
            };
            let mut new_fields = g.stream();
            new_fields.extend(comm);
            new_fields.extend(fill);
            //eprintln!("{new_fields}");
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
            /// Fills all fields of the struct from its strings.
            fn vfill(&mut self) {
                #vfill
            }
        }
    );

    let tree : TokenStream = vtree.into_iter().chain(vfill_impl).collect();
    tree.into_iter().collect::<TokenStream>().into()
}

#[proc_macro_attribute]
pub fn crunch_fill_eval(_attr: proc_macro::TokenStream, s: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree : TokenStream = s.into();
    let mut vtree = tree.into_iter().collect::<Vec<_>>();

    let body = vtree.last().expect("Struct expected").clone();
    let on_comma = on_punct(',');

    let struct_pos = vtree.iter().position(|x| {
        match x {
            TokenTree::Ident(i) => *i == "struct",
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
            let nam : TokenStream = names.iter().flat_map(|n| [n.clone(), TokenTree::Punct(Punct::new(',', Spacing::Alone))]).collect();
            let namstr = format!("{nam} error,");
            let comm = quote!(#[doc = #namstr]);
            let fill = if end_comma {
                quote!(
                    strings: [String; #vals],
                )
            }
            else {
                quote!(
                    ,strings: [String; #vals],
                )
            };
            let mut new_fields = g.stream();
            new_fields.extend(comm);
            new_fields.extend(fill);
            //eprintln!("{new_fields}");
            (TokenTree::Group(Group::new(g.delimiter(), new_fields)), names)
        },
        _ => panic!("Not usable on unit structs."),
    };
    *(vtree.last_mut().unwrap()) = args;

    let vfill: TokenStream = names.iter().enumerate().flat_map(|(i,n)| {
        quote!(if let Ok(val) = self.strings[#i].parse::<Constr<Expr>>() {
            if let Ok(val) = val.eval() {
                self.#n = val;
            }
        })
    }).collect();

    let vfill_impl = quote!(
        impl #struct_name {
            /// Fills all fields of the struct from its strings.
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