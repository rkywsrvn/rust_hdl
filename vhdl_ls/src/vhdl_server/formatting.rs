use crate::vhdl_server::{uri_to_file_name, VHDLServer};
use lsp_types::{DocumentFormattingParams, Position, Range, TextEdit};
use vhdl_lang::{Message, Source};

impl VHDLServer {
    pub fn document_formatting(
        &mut self,
        params: &DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        let source = self
            .project
            .get_source(&uri_to_file_name(&params.text_document.uri))?;
        match self.project.format_source(&source) {
            Ok(formatted) => Some(vec![TextEdit {
                range: Range::new(Position::new(0, 0), end_of(&source)),
                new_text: formatted,
            }]),
            Err(err) => {
                self.message(Message::log(err.to_string()));
                None
            }
        }
    }
}

/// The position after the last character of a source.
fn end_of(source: &Source) -> Position {
    let contents = source.contents();
    let num_lines = contents.num_lines() as u32;
    match contents.get_line(contents.num_lines().saturating_sub(1)) {
        Some(line) if line.ends_with('\n') => Position::new(num_lines, 0),
        Some(line) => Position::new(
            num_lines - 1,
            line.chars().map(char::len_utf16).sum::<usize>() as u32,
        ),
        None => Position::new(0, 0),
    }
}

#[cfg(test)]
mod tests {
    use crate::vhdl_server::tests::{
        expect_loaded_config_messages, initialize_server, setup_server, temp_root_uri,
        write_config, write_file,
    };
    use lsp_types::{
        DocumentFormattingParams, Position, Range, TextDocumentIdentifier, TextEdit, Url,
    };

    fn formatting_params(uri: Url) -> DocumentFormattingParams {
        DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(uri),
            options: Default::default(),
            work_done_progress_params: Default::default(),
        }
    }

    #[test]
    fn formats_whole_document() {
        let (mock, mut server) = setup_server();
        let (_tempdir, root_uri) = temp_root_uri();
        let file_uri = write_file(&root_uri, "file.vhd", "entity   ent is\nend   entity;\n");
        let config_uri = write_config(&root_uri, "[libraries]\nlib.files = ['file.vhd']\n");
        expect_loaded_config_messages(&mock, &config_uri);
        initialize_server(&mut server, root_uri);

        assert_eq!(
            server.document_formatting(&formatting_params(file_uri)),
            Some(vec![TextEdit {
                range: Range::new(Position::new(0, 0), Position::new(2, 0)),
                new_text: "entity ent is\nend entity;\n".to_owned(),
            }])
        );
    }

    #[test]
    fn does_not_format_ignored_regions() {
        let (mock, mut server) = setup_server();
        let (_tempdir, root_uri) = temp_root_uri();
        let file_uri = write_file(
            &root_uri,
            "file.vhd",
            "entity ent is\nend entity;\n-- vhdl_ls off\nfoo\n-- vhdl_ls on\n",
        );
        let config_uri = write_config(&root_uri, "[libraries]\nlib.files = ['file.vhd']\n");
        expect_loaded_config_messages(&mock, &config_uri);
        initialize_server(&mut server, root_uri);

        mock.expect_message_contains("Cannot format");
        assert_eq!(
            server.document_formatting(&formatting_params(file_uri)),
            None
        );
    }
}
