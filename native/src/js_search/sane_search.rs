use std::path::PathBuf;

use tantivy::{
    query::{QueryParser, Query},
    schema::{Schema, Field},
    DocAddress,
    Score,
    Index, IndexWriter,
    collector::{TopDocs},
    tokenizer::{Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, TextAnalyzer},
};

 
pub struct SaneSearch {
    pub index: Option<Index>,
    pub schema: Option<Schema>,
    pub index_writer: Option<IndexWriter>,
    pub default_search_fields: Option<Vec<Field>>
}



impl SaneSearch {

    pub fn simple_search(&self, query: &str) -> Result<Vec<String>, &str> {
        let (index, schema, default_search_fields): (&Index,_,_) = {
            let (index, schema, default_search_fields);

            if let Some(s) = &self.schema {
                schema = s;
            } else {
                return Err("NoSchemaError")
            }

            if let Some(i) = &self.index {
                index = i;
            } else {
                return Err("Cannot search without an index")
            }

            if let Some(fields) = &self.default_search_fields {
                default_search_fields = fields;
            } else {
                return Err("No default_search_fields")
            }

            (index, schema, default_search_fields)
        };

        let query_parser = QueryParser::for_index(&index, default_search_fields.to_vec());
        let query = query_parser.parse_query(query).unwrap();

        let reader = index.reader().map_err(|_| "Unable to acquire reader")?;
        let searcher = reader.searcher();

        // let mut top_collector = TopCollector::with_limit(10);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        let mut results: Vec<String> = vec![];
        for (_score, doc_address) in top_docs  {
            let retrieved_doc = searcher.doc(doc_address).unwrap();
                results.push(schema.to_json(&retrieved_doc));
        }
        
        Ok(results)
    }

    pub fn top_search(&self, query: &Box<dyn Query>, collector: &TopDocs) -> Vec<(Score, String)> {
        let index = self.index.as_ref().expect("Cannot search without an index");
        let schema = self.schema.as_ref().expect("Cannot search without a schema");
        let reader = index.reader().expect("Unable to acquire reader");
        let searcher = reader.searcher();

        let top_docs: Vec<(Score, DocAddress)> =  searcher.search(query, collector).unwrap();
        
        let mut results: Vec<(Score, String)> = vec![];
        for (score, doc_address) in top_docs  {
            let retrieved_doc = searcher.doc(doc_address).unwrap();

            results.push((score, schema.to_json(&retrieved_doc)));
        }

        results
    }

    #[allow(dead_code)]
    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = Some(schema);
    }

    #[allow(dead_code)]
    pub fn set_index(&mut self, index: Index) {
        self.index = Some(index);
    }

    pub fn set_default_fields(&mut self, fields: Vec<Field>) {
        self.default_search_fields = Some(fields);
    }

    pub fn create_index(&mut self, path: String) -> tantivy::Result<()> {
        let schema = self.schema.as_ref().expect("Cannot create a new index without a schema");
        let dir_path = PathBuf::from(path);
        //let dir = tantivy::directory::MmapDirectory::open(dir_path)?;
        
        let index = Index::create_in_dir(dir_path.as_path(), schema.clone())?; 

        // Attach custom tokenizers for different languages
        SaneSearch::register_custom_text_analyzers(&index);

        self.index = Some(index);
        Ok(())
    }

    pub fn open_index(&mut self, path: String) -> tantivy::Result<()> {
        let dir_path = PathBuf::from(path);
        let dir = tantivy::directory::MmapDirectory::open(dir_path)?;
        
        let index = Index::open(dir)?; 

        // Attach custom tokenizers for different languages
        SaneSearch::register_custom_text_analyzers(&index);

        self.schema = Some(index.schema());
        self.index = Some(index);
        Ok(())
    }

    pub fn create_index_writer(&mut self, heap_size: usize) -> tantivy::Result<()> {
        let index = self.index.as_ref().expect("Cannot create a new index writer without an index");

        if let Some(_writer) = self.index_writer.as_ref() {
            Ok(())
        } else {
            let writer = index.writer(heap_size)?;
            self.index_writer = Some(writer);
            Ok(())
        }
    }

    pub fn add_doc(&mut self, json: &str) -> tantivy::Result<()> {
        let index_writer = self.index_writer.as_mut().expect("Cannot add a document without an index writer");
        let schema = self.schema.as_ref().expect("Cannot add a document without a schema");
        let doc = schema.parse_document(json)?;
        index_writer.add_document(doc);
        Ok(())
    }

    pub fn commit_index_writer(&mut self) -> tantivy::Result<()> {
        let index_writer = self.index_writer.as_mut().expect("Cannot add a document without an index writer");
        index_writer.commit()?;
        Ok(())
    }

    #[deprecated]
    pub fn load_searchers(&self) -> tantivy::Result<()> {
        println!("This function does nothing now");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn consume_writer(&mut self) {
        let _index_writer = self.index_writer.take().expect("Cannot consume an index writer without an index writer");
    }
    
    // Build custom text analyzers to be attached to the index
    // For now sticking with French, German and Spanish for simplicity
    // https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
    // https://docs.rs/tantivy/latest/tantivy/tokenizer/struct.TextAnalyzer.html
    // https://docs.rs/tantivy/latest/tantivy/tokenizer/enum.Language.html
    fn register_custom_text_analyzers(index: &Index) {
        let de_stem = TextAnalyzer::from(SimpleTokenizer)
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(Language::German));
        index.tokenizers().register("de_stem", de_stem);

        let es_stem = TextAnalyzer::from(SimpleTokenizer)
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(Language::Spanish));
        index.tokenizers().register("es_stem", es_stem);

        let fr_stem = TextAnalyzer::from(SimpleTokenizer)
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(Language::French));
        index.tokenizers().register("fr_stem", fr_stem);
    }
}
