extern crate proc_macro;
extern crate proc_macro2;

use quote::quote;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use serializer::*;
use syn::{parse_str, TypePath};

#[proc_macro]
pub fn generate_struct(_: TokenStream) -> TokenStream {
    let root_dir = std::fs::read_dir("resources/shaders/reflected").unwrap();

    let mut result = quote! {
        struct ErrorStruct{}
    };

    for file in root_dir {
        let file = file.unwrap().path();
        let path = file.parent().unwrap();
        let name = file.file_stem().unwrap();

        let main_node = YamlNode::deserialize(
            path.to_str().unwrap(), 
            name.to_str().unwrap())
        .unwrap();

        for structs in &main_node.children {
            let struct_name = format!("{}_{}", &structs.name, &main_node.name);
            
            let mut fields_tokens = TokenStream2::default();
            if !structs.children.is_empty() {
                for field in &structs.children {
                    let field_name = &field.name;
                    let field_type = &field.node_type;

                    let field_name = parse_str::<Ident>(&field_name).unwrap();
                    let field_type = parse_str::<TypePath>(&field_type).unwrap();
                    
                    let field_tokens = quote! {
                        pub #field_name: #field_type,
                    };
                    
                    fields_tokens.extend::<TokenStream2>(field_tokens.into());
                }

                let struct_name = parse_str::<Ident>(&struct_name).unwrap();
                result.extend::<TokenStream2>(quote! {
                    #[derive(Debug)]
                    pub struct #struct_name {
                        #fields_tokens
                    }
                }.into());
            }
        }
    }

    result.into()
}