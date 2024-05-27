use std::{fs, path::PathBuf};

use serde_json::{from_str, to_string};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::{doc, Index, IndexWriter};

use crate::{error::Result, Link};

const TANTIVY_INDEX_DIR: &str = ".tantivy";
const TANTIVY_WRITER_SIZE: usize = 256_000_000; // 256 MB

pub struct CacheBuilder {
    root_dir: PathBuf,
}

/// Cache wraps an instance of the link cache. Each instance has a root
/// directory that contains data files in both a raw format and a
/// Tantivy index format.
pub struct Cache {
    index: Index,
    writer: IndexWriter,
}

impl CacheBuilder {
    pub fn new() -> Self {
        CacheBuilder {
            root_dir: Cache::default_root_dir(),
        }
    }

    pub fn with_root_dir(&mut self, dir: PathBuf) -> &mut Self {
        self.root_dir = dir;
        self
    }

    /// Initializes the index, its schema, and custom tokenization
    pub fn build(&self) -> Result<Cache> {
        // Ensure the index directory exists
        let index_path = self.root_dir.join(TANTIVY_INDEX_DIR);
        fs::create_dir_all(&index_path)?;

        // Schema
        let mut builder = Schema::builder();
        builder.add_text_field(
            "title",
            TextOptions::default().set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("ngram")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            ),
        );
        builder.add_text_field("subtitle", TEXT);
        builder.add_text_field("url", STRING);
        builder.add_text_field("struct_json", STORED);
        let schema = builder.build();

        // Open or create the index
        let index = Index::open_in_dir(&index_path)
            .or_else(|_| Index::create_in_dir(&index_path, schema))?;

        // Custom NGram Tokenizer for the Title field
        let tokenizer = TextAnalyzer::builder(NgramTokenizer::new(3, 3, false).unwrap())
            .filter(LowerCaser)
            .filter(StopWordFilter::remove(vec![
                "to".to_string(),
                "the".to_string(),
                "and".to_string(),
            ]))
            .build();
        index.tokenizers().register("ngram", tokenizer);

        // Prepare an index writer
        let writer = index.writer(TANTIVY_WRITER_SIZE)?;
        Ok(Cache { index, writer })
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Adds a new link to the index. The url field is used as the unique
    /// key. This function removes any existing link with the same url before
    /// saving a new one. The commit() function must be called after adding
    /// to persist the changes. Batch updates should call add() many times
    /// and commit() once.
    pub fn add(&mut self, link: Link) -> Result<()> {
        self.remove(&link)?;

        let json_str = to_string(&link)?;

        let writer = &self.writer;
        writer.add_document(doc!(
            self.index.schema().get_field("title")? => link.title,
            self.index.schema().get_field("subtitle")? => link.subtitle.unwrap_or_default(),
            self.index.schema().get_field("url")? => link.url,
            self.index.schema().get_field("struct_json")? => json_str,
        ))?;
        Ok(())
    }

    /// Removes a Link from the index. The url field is used as the unique key.
    /// The commit() function must be called after removing to persist the
    /// the change on disk.
    pub fn remove(&mut self, link: &Link) -> Result<()> {
        let field = self.index.schema().get_field("url")?;
        let term = Term::from_field_text(field, &link.url.to_string());
        let _ = self.writer.delete_term(term);
        Ok(())
    }

    /// Persists all pending index changes to disk. No change is reflected in
    /// the index until this is called.
    pub fn commit(&mut self) -> Result<()> {
        self.writer.commit()?;
        Ok(())
    }

    /// Searches the index for linkx matching the query
    pub fn search(&self, query: &str) -> Result<Vec<Link>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let title_field = self.index.schema().get_field("title").unwrap();
        let subtitle_field = self.index.schema().get_field("subtitle").unwrap();
        let mut parser = QueryParser::for_index(&self.index, vec![title_field, subtitle_field]);
        parser.set_field_fuzzy(title_field, true, 1, true);
        parser.set_field_boost(title_field, 2.0);
        let query = parser.parse_query_lenient(query).0;

        let top_docs = searcher.search(&query, &tantivy::collector::TopDocs::with_limit(20))?;

        let results = top_docs
            .into_iter()
            .map(|(score, doc_address)| {
                let doc: TantivyDocument = searcher.doc(doc_address).unwrap();
                let json_string = doc
                    .get_first(self.index.schema().get_field("struct_json").unwrap())
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_owned();
                let mut link: Link = from_str(json_string.as_str()).unwrap();
                link.score = Some(score);
                link
            })
            .collect();
        Ok(results)
    }

    /// Calculates the default root directory for a cache. Will use the
    /// .linkcache directory in the user's home directory, or /tmp/.linkcache
    /// as an alternate if we don't have a home directory.
    fn default_root_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".linkcache")
    }
}

/// Defines the Default implementaton for Cache.
impl Default for Cache {
    fn default() -> Self {
        CacheBuilder::new().build().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_cache_instance() -> Cache {
        let binding = tempdir().expect("Failed to create temp dir");
        let temp_dir = binding.path();
        let cache = CacheBuilder::new()
            .with_root_dir(temp_dir.to_path_buf())
            .build()
            .expect("Failed to build cache");
        cache
    }

    #[test]
    fn test_new_default_path() {
        let cache = CacheBuilder::new();
        assert!(cache.root_dir.ends_with(".linkcache"));
    }

    #[test]
    fn test_add_and_search_fuzzy() {
        let mut cache = test_cache_instance();
        assert!(cache
            .add(Link {
                title: "Visual Studio Code".to_string(),
                url: "https://code.visualstudio.com".to_string(),
                ..Default::default()
            })
            .is_ok());
        assert!(cache
            .add(Link {
                title: "Sublime Text".to_string(),
                url: "https://www.sublimetext.com".to_string(),
                ..Default::default()
            })
            .is_ok());
        assert!(cache.commit().is_ok());
        let results = cache.search("Viz Studio").expect("Search failed");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Visual Studio Code");
    }
}
