use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Patch)]
pub fn patch_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if let syn::Data::Struct(data) = input.data {
        let mut field_lookup_table: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut seq_access_table: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut rkyv_table: Vec<proc_macro2::TokenStream> = Vec::new();

        for field in data.fields {
            let len = field_lookup_table.len() as u8;
            let ident = field.ident.unwrap();
            let ty = field.ty;

            field_lookup_table.push(quote! {
                #len => { self.#ident = <#ty as serde::Deserialize<'de>>::deserialize(deserializer)?; }
            });

            seq_access_table.push(quote! {
                #len => { self.#ident = seq.next_element()?.unwrap(); }
            });

            rkyv_table.push(quote! {
                #len => {
                    self.#ident = unsafe { rkyv::util::archived_value::<#ty>(bytes, position) }.deserialize(deser)?;
                    // *position += std::mem::size_of::<<#ty as rkyv::Archive>::Archived>();
                }
            });
        }

        let unreachable_clause = if field_lookup_table.len() < 256 {
            Some(quote! {
                _ => unreachable!()
            })
        } else {
            None
        };

        let ret = quote! {
            impl #impl_generics vhs_diff::Patch for #name #ty_generics #where_clause {
                #[inline(always)]
                fn do_patch_command<'de, D>(&mut self, field_index: u8, deserializer: D) -> Result<(), D::Error>
                where
                    D: serde::Deserializer<'de> {
                        match field_index {
                            #(#field_lookup_table),*
                            #unreachable_clause
                        };

                        Ok(())
                    }

                fn do_patch_from_seq<'de, A>(&mut self, field_index: u8, seq: &mut A) -> Result<(), A::Error> where A: serde::de::SeqAccess<'de> {
                    match field_index {
                        #(#seq_access_table),*
                        #unreachable_clause
                    };

                    Ok(())
                }

                unsafe fn do_patch_from_byteslice<D>(
                    &mut self,
                    field_index: u8,
                    deser: &mut D,
                    position: usize,
                    bytes: &[u8],
                ) -> Result<(), D::Error>
                where
                    D: rkyv::Fallible + ?Sized {
                        use rkyv::Deserialize;

                        match field_index {
                            #(#rkyv_table),*
                            #unreachable_clause
                        };

                        Ok(())
                    }
            }
        };

        TokenStream::from(ret)
    } else {
        panic!("can't derive diff for enums");
    }
}

#[proc_macro_derive(Diff)]
pub fn diff_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if let syn::Data::Struct(data) = input.data {
        let mut field_lookup_table: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut rkyv_table: Vec<proc_macro2::TokenStream> = Vec::new();

        for field in data.fields {
            let len = field_lookup_table.len() as u8;
            let ident = field.ident.unwrap();
            field_lookup_table.push(quote! {
                if self.#ident != rhs.#ident {
                    res_vec.push(vhs_diff::OwnedDiffCommand {
                        index: #len,
                        value: Box::new(rhs.#ident)
                    });
                }
            });

            rkyv_table.push(quote! {
                if self.#ident != rhs.#ident {
                    positions.push(vhs_diff::FieldAndPosition {
                        index: #len,
                        position: ser.serialize_value(&rhs.#ident).unwrap() as u32
                    });
                }
            })
        }

        let vec_capacity = field_lookup_table.len();

        let ret = quote! {
            impl #impl_generics vhs_diff::Diff for #name #ty_generics #where_clause {
                fn diff(&self, rhs: Self) -> vhs_diff::OwnedPatch {
                    let mut res_vec: Vec<vhs_diff::OwnedDiffCommand> = Vec::with_capacity(#vec_capacity);

                    #(#field_lookup_table)*

                    vhs_diff::OwnedPatch(res_vec)
                }

                fn diff_rkyv(&self, rhs: Self) -> vhs_diff::ArchivablePatch {
                    use rkyv::ser::{serializers::AllocSerializer, Serializer};
                    let mut positions: Vec<vhs_diff::FieldAndPosition> = Vec::with_capacity(#vec_capacity);
                    let mut ser = AllocSerializer::<1024>::default();

                    #(#rkyv_table)*

                    vhs_diff::ArchivablePatch {
                        field_positions: positions,
                        patch_bytes: ser.into_serializer().into_inner().to_vec()
                    }
                }
            }
        };

        TokenStream::from(ret)
    } else {
        panic!("can't derive diff for enums");
    }
}
