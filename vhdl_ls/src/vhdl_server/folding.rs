use crate::vhdl_server::{uri_to_file_name, VHDLServer};
use lsp_types::{FoldingRange, FoldingRangeParams};
use vhdl_lang::{EntHierarchy, Token};

impl VHDLServer {
    pub fn folding_range(&self, params: &FoldingRangeParams) -> Option<Vec<FoldingRange>> {
        let source = self
            .project
            .get_source(&uri_to_file_name(&params.text_document.uri))?;
        let library_name = self
            .project
            .library_mapping_of(&source)
            .into_iter()
            .next()?;
        let mut ranges = Vec::new();
        for (hierarchy, tokens) in self.project.document_symbols(&library_name, &source) {
            collect_folding_ranges(hierarchy, tokens, &mut ranges);
        }
        ranges.sort_by_key(|range| (range.start_line, range.end_line));
        ranges.dedup_by_key(|range| (range.start_line, range.end_line));
        Some(ranges)
    }
}

fn collect_folding_ranges(
    EntHierarchy { ent, children }: EntHierarchy,
    tokens: &Vec<Token>,
    ranges: &mut Vec<FoldingRange>,
) {
    let range = ent.src_span.pos(tokens).range();
    if range.end.line > range.start.line {
        ranges.push(FoldingRange {
            start_line: range.start.line,
            end_line: range.end.line,
            ..Default::default()
        });
    }
    for child in children {
        collect_folding_ranges(child, tokens, ranges);
    }
}

#[cfg(test)]
mod tests {
    use crate::vhdl_server::tests::{
        expect_loaded_config_messages, initialize_server, setup_server, temp_root_uri,
        write_config, write_file,
    };
    use lsp_types::{FoldingRange, FoldingRangeParams, TextDocumentIdentifier};

    #[test]
    fn folds_multi_line_declarations_and_statements() {
        let (mock, mut server) = setup_server();
        let (_tempdir, root_uri) = temp_root_uri();
        let file_uri = write_file(
            &root_uri,
            "file.vhd",
            "\
entity ent is
end entity;

architecture arch of ent is
begin
    process
    begin
        wait;
    end process;
end architecture;
",
        );
        let config_uri = write_config(&root_uri, "[libraries]\nlib.files = ['file.vhd']\n");
        expect_loaded_config_messages(&mock, &config_uri);
        initialize_server(&mut server, root_uri);

        let ranges = server
            .folding_range(&FoldingRangeParams {
                text_document: TextDocumentIdentifier::new(file_uri),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .unwrap();
        let fold = |start_line, end_line| FoldingRange {
            start_line,
            end_line,
            ..Default::default()
        };
        assert_eq!(ranges, vec![fold(0, 1), fold(3, 9), fold(5, 8)]);
    }
}
