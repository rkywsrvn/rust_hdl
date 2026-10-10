use crate::vhdl_server::{from_lsp_pos, uri_to_file_name, VHDLServer};
use lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use vhdl_lang::OverloadedEnt;

impl VHDLServer {
    pub fn signature_help(&self, params: &SignatureHelpParams) -> Option<SignatureHelp> {
        let position = &params.text_document_position_params;
        let source = self
            .project
            .get_source(&uri_to_file_name(&position.text_document.uri))?;
        let help = self
            .project
            .signature_help(&source, from_lsp_pos(position.position))?;
        let signatures = help
            .signatures
            .iter()
            .map(|signature| {
                let mut information = signature_information(*signature);
                information.active_parameter =
                    help.active_parameter(*signature).map(|index| index as u32);
                information
            })
            .collect();
        Some(SignatureHelp {
            signatures,
            active_signature: Some(help.active_signature as u32),
            active_parameter: None,
        })
    }
}

/// Describes a subprogram as `name(a : integer; b : bit) return bit`.
fn signature_information(signature: OverloadedEnt<'_>) -> SignatureInformation {
    let mut label = signature.designator().to_string();
    let mut parameters = Vec::new();
    if signature.formals().len() > 0 {
        label.push('(');
        for (i, formal) in signature.formals().iter().enumerate() {
            if i > 0 {
                label.push_str("; ");
            }
            let start = utf16_len(&label);
            label.push_str(&format!(
                "{} : {}",
                formal.designator(),
                formal.type_mark().designator()
            ));
            parameters.push(ParameterInformation {
                label: ParameterLabel::LabelOffsets([start, utf16_len(&label)]),
                documentation: None,
            });
        }
        label.push(')');
    }
    if let Some(return_type) = signature.return_type() {
        label.push_str(&format!(" return {}", return_type.designator()));
    }
    SignatureInformation {
        label,
        documentation: None,
        parameters: Some(parameters),
        active_parameter: None,
    }
}

fn utf16_len(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
    use crate::vhdl_server::tests::{
        expect_loaded_config_messages, initialize_server, setup_server, std_lib_config,
        temp_root_uri, write_config, write_file,
    };
    use lsp_types::{
        ParameterInformation, ParameterLabel, Position, SignatureHelp, SignatureHelpParams,
        SignatureInformation, TextDocumentIdentifier, TextDocumentPositionParams,
    };

    #[test]
    fn describes_the_called_subprogram() {
        let (mock, mut server) = setup_server();
        let (_tempdir, root_uri) = temp_root_uri();
        let file_uri = write_file(
            &root_uri,
            "file.vhd",
            "\
package pkg is
    type t is (a, b);
    procedure proc(x : t; y : t);
end package;

package body pkg is
    procedure proc(x : t; y : t) is
    begin
        proc(x, y);
    end procedure;
end package body;
",
        );
        let config_uri = write_config(&root_uri, std_lib_config());
        expect_loaded_config_messages(&mock, &config_uri);
        initialize_server(&mut server, root_uri);

        let help = server.signature_help(&SignatureHelpParams {
            context: None,
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier::new(file_uri),
                position: Position::new(8, "        proc(x, ".len() as u32),
            },
            work_done_progress_params: Default::default(),
        });
        assert_eq!(
            help,
            Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: "proc(x : t; y : t)".to_owned(),
                    documentation: None,
                    parameters: Some(vec![
                        ParameterInformation {
                            label: ParameterLabel::LabelOffsets([5, 10]),
                            documentation: None,
                        },
                        ParameterInformation {
                            label: ParameterLabel::LabelOffsets([12, 17]),
                            documentation: None,
                        },
                    ]),
                    active_parameter: Some(1),
                }],
                active_signature: Some(0),
                active_parameter: None,
            })
        );
    }
}
