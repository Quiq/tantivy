extern crate neon;
extern crate tantivy;

use neon::prelude::*;
use tantivy::{
    schema::{TextOptions},
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

            let mut text_options = TextOptions::default();

            for handle in vec.iter() {
                //let option: String = handle.to_string(&mut cx)?.value();
                let option: String = handle.to_string(&mut cx)?.value();
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