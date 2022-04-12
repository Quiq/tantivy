extern crate neon;
extern crate tantivy;

use neon::prelude::*;
use tantivy::{
    schema::{IndexRecordOption, TextFieldIndexing, TextOptions},
};


mod sane_schema_builder;
use sane_schema_builder::SaneSchemaBuilder;

declare_types! {
    pub class JsSchemaBuilder for SaneSchemaBuilder {
        init(mut _cx) {
            Ok(SaneSchemaBuilder::new())
        }

        method addTextField(mut cx) {
            let field_name = cx.argument::<JsString>(0)?.value();

            let js_arr_handle: Handle<JsArray> = cx.argument(1)?;
            let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;

            // Allow the selection of tokenizers for different languages. 
            // Out of the box tokenizers: default, raw, en_stem
            // https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
            let mut tokenizer = cx.argument::<JsString>(2)?.value();
            tokenizer = if !tokenizer.is_empty() {
                tokenizer
            } else {
                "default".to_string()
            };

            println!("Using tokenizer {}", tokenizer);

            let mut text_options = TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer(&tokenizer)
                        .set_index_option(IndexRecordOption::Basic)
                );
            for handle in vec.iter() {
                //let option: String = handle.to_string(&mut cx)?.value();
                let option: String = handle.to_string(&mut cx)?.value();

                // https://docs.rs/tantivy/latest/tantivy/schema/index.html#constants
                let new_options = match option.as_ref() {
                    "TEXT" => tantivy::schema::TEXT,
                    "STORED" => tantivy::schema::STORED.into(),
                    "STRING" => tantivy::schema::STRING.into(),
                    _ => panic!("Unknown text option")
                };
                text_options = new_options | text_options.clone();
            }

            let mut this = cx.this();
            let field = {
                let guard = cx.lock();
                let mut borrowed_self  = this.borrow_mut(&guard);
                borrowed_self.add_text_field(&field_name, text_options)
            };

            Ok(cx.number(field.field_id()).upcast())
        }

        method addFacetField(mut cx) {
            let field_name = cx.argument::<JsString>(0)?.value();
            
            let mut this = cx.this();
            let field = {
                let guard = cx.lock();
                let mut borrowed_self = this.borrow_mut(&guard);
                borrowed_self.add_facet_field(&field_name)
            };   

            Ok(cx.number(field.field_id()).upcast())
        }
    }
}